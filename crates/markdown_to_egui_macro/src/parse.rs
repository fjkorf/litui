//! Markdown event loop: pulldown-cmark events to `ParsedMarkdown`.
//!
//! This module walks the pulldown-cmark 0.9 event stream and accumulates
//! [`Fragment`]s (styled text, inline code, links, widgets) which are flushed
//! into generated `TokenStream` code at block boundaries.
//!
//! Key design choices:
//! - **Index-based iteration** (`while event_idx < events.len()`) instead of
//!   `for event in events`, enabling one-event lookahead for widget `{config}`
//!   consumption.
//! - **Fragment accumulation** -- inline content is buffered as [`Fragment`]
//!   values, then flushed at flush points into `ui.horizontal_wrapped(|ui| { ... })`
//!   calls.
//! - **Flush points**: `End(Paragraph)`, `End(Item)` (tight list fallback),
//!   `Start(List)` (parent item before nesting), `End(Heading)`, `End(CodeBlock)`,
//!   `End(TableCell)`.
//! - **Table-aware widget emission**: widgets inside table cells are pushed to
//!   `fragments` as `Fragment::Widget` so they render inside the `egui::Grid`
//!   closure; outside tables they go directly to `code_body`.
//!
//! See `knowledge/pulldown-cmark-0.9.md` for the event model and
//! `knowledge/proc-macro-architecture.md` for the fragment accumulation pattern.

use pulldown_cmark::{Event, Tag};
use quote::quote;

use crate::frontmatter::{
    Frontmatter, WidgetDef, detect_style_suffix, parse_hex_color, parse_selectors, resolve_classes,
    style_def_to_label_tokens,
};

/// A styled text fragment accumulated within a paragraph at compile time.
///
/// Fragments are collected between block boundaries (e.g., within a paragraph
/// or table cell) and flushed together into a `ui.horizontal_wrapped(...)` call.
///
/// - [`Styled`](Fragment::Styled) -- plain text with bold/italic/strikethrough flags,
///   created from `Text` events and style tag boundaries.
/// - [`InlineCode`](Fragment::InlineCode) -- backtick-delimited inline code spans.
/// - [`Link`](Fragment::Link) -- hyperlinks with display text, URL, and style flags.
/// - [`Widget`](Fragment::Widget) -- pre-generated widget `TokenStream`, used so
///   widget directives inside table cells render within the `egui::Grid` closure.
// ── Fragment enum ────────────────────────────────────────────────

pub(crate) enum Fragment {
    Styled {
        text: String,
        bold: bool,
        italic: bool,
        strikethrough: bool,
    },
    InlineCode(String),
    Link {
        text: String,
        url: String,
        bold: bool,
        italic: bool,
        strikethrough: bool,
    },
    /// Pre-generated widget code (used so widgets work inside table cells).
    Widget(proc_macro2::TokenStream),
    /// A foreach row field reference — rendered as `__row.field_name`.
    ForeachField(String),
}

// ── Widget type system ───────────────────────────────────────────

/// The Rust type of a widget's state field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WidgetType {
    F64,
    Bool,
    U32,
    Usize,
    String,
    ByteArray4,
    VecString,
}

impl WidgetType {
    pub fn to_tokens(self) -> proc_macro2::TokenStream {
        match self {
            Self::F64 => quote! { f64 },
            Self::Bool => quote! { bool },
            Self::U32 => quote! { u32 },
            Self::Usize => quote! { usize },
            Self::String => quote! { String },
            Self::ByteArray4 => quote! { [u8; 4] },
            Self::VecString => quote! { Vec<String> },
        }
    }

    pub fn default_tokens(self) -> proc_macro2::TokenStream {
        match self {
            Self::F64 => quote! { 0.0 },
            Self::Bool => quote! { false },
            Self::U32 => quote! { 0 },
            Self::Usize => quote! { 0 },
            Self::String => quote! { String::new() },
            Self::ByteArray4 => quote! { [255, 255, 255, 255] },
            Self::VecString => quote! { Vec::new() },
        }
    }
}

/// A widget field discovered during parsing. Collected into a generated
/// state struct (`MdFormState` or `AppState`).
#[derive(Clone)]
pub(crate) enum WidgetField {
    /// Standard stateful widget (slider, checkbox, textedit, etc.)
    Stateful { name: String, ty: WidgetType },
    /// Foreach collection — generates a row struct + `Vec<RowStruct>`
    Foreach {
        name: String,
        row_fields: Vec<String>,
    },
}

impl WidgetField {
    pub fn name(&self) -> &str {
        match self {
            Self::Stateful { name, .. } | Self::Foreach { name, .. } => name,
        }
    }

    pub fn ty(&self) -> Option<WidgetType> {
        match self {
            Self::Stateful { ty, .. } => Some(*ty),
            Self::Foreach { .. } => None,
        }
    }
}

// ── Block directive stack ────────────────────────────────────────

/// The type of block directive opened by `:::`.
enum BlockDirective {
    Foreach {
        row_fields: Vec<String>,
    },
    If,
    Style,
    Frame {
        style: Option<crate::frontmatter::StyleDef>,
    },
    Horizontal,
    Columns {
        count: usize,
        current_col: usize,
        column_bodies: Vec<Vec<proc_macro2::TokenStream>>,
    },
}

/// A stack frame for a `:::` block directive.
struct BlockFrame {
    directive: BlockDirective,
    field_name: String,
    saved_code_body: Vec<proc_macro2::TokenStream>,
}

// ── ParsedMarkdown output ────────────────────────────────────────

/// Structured output from parsing a single markdown file.
pub(crate) struct ParsedMarkdown {
    pub(crate) code_body: Vec<proc_macro2::TokenStream>,
    pub(crate) widget_fields: Vec<WidgetField>,
    /// True if the generated code references `state` (e.g., display widgets).
    pub(crate) references_state: bool,
    /// Field names referenced by display widgets (for validation against AppState).
    pub(crate) display_refs: Vec<String>,
    /// Generated style lookup function tokens (when dynamic styling is used).
    pub(crate) style_table: Option<proc_macro2::TokenStream>,
}

// ── Helpers ──────────────────────────────────────────────────────

fn get_widget_def(attrs: &str, frontmatter: &Frontmatter) -> WidgetDef {
    if attrs.is_empty() {
        WidgetDef::default()
    } else {
        frontmatter.widgets.get(attrs).cloned().unwrap_or_default()
    }
}

/// Scan text for `::class(text)` inline styled span patterns.
/// Splits into alternating Styled and Widget (styled_label_rich) fragments.
/// Returns true if any spans were found and processed.
fn parse_inline_styled_spans(
    text: &str,
    fragments: &mut Vec<Fragment>,
    frontmatter: &Frontmatter,
    bold: bool,
    italic: bool,
    strikethrough: bool,
) -> bool {
    let mut found = false;
    let mut remaining = text;

    while let Some(dcolon) = remaining.find("::") {
        // Check for ::identifier[ pattern
        let after_colons = &remaining[dcolon + 2..];
        let ident_end = after_colons
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .unwrap_or(after_colons.len());
        if ident_end == 0 {
            break; // no identifier after ::
        }
        let class_name = &after_colons[..ident_end];
        let after_ident = &after_colons[ident_end..];

        if !after_ident.starts_with('(') {
            break; // no [ after identifier
        }

        // Find matching ]
        if let Some(close_bracket) = after_ident.find(')') {
            let span_text = &after_ident[1..close_bracket];

            // Emit text before the span
            let before = &remaining[..dcolon];
            if !before.is_empty() {
                fragments.push(Fragment::Styled {
                    text: before.to_owned(),
                    bold,
                    italic,
                    strikethrough,
                });
            }

            // Resolve style and emit styled fragment
            let style = frontmatter.styles.get(class_name).unwrap_or_else(|| {
                panic!("Undefined style class '::{}' in inline span", class_name)
            });
            let tokens = style_def_to_label_tokens(span_text, style, bold, italic, strikethrough);
            fragments.push(Fragment::Widget(tokens));

            remaining = &after_ident[close_bracket + 1..];
            found = true;
        } else {
            break; // no closing ]
        }
    }

    // Emit any remaining text
    if found && !remaining.is_empty() {
        fragments.push(Fragment::Styled {
            text: remaining.to_owned(),
            bold,
            italic,
            strikethrough,
        });
    }

    found
}

pub(crate) fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

/// Parse markdown content into a [`ParsedMarkdown`] using the pulldown-cmark event loop.
///
/// This is the core of the macro: it walks every pulldown-cmark event, tracks
/// inline style state (bold/italic/strikethrough), list nesting, blockquote depth,
/// table structure, and widget directives. Text fragments are accumulated and
/// flushed into `TokenStream` code at block boundaries.
///
/// The `frontmatter` parameter provides style and widget definitions for
/// `::key` suffix and `::key(text)` inline span resolution during code generation.
pub(crate) fn markdown_to_egui(content: &str, frontmatter: &Frontmatter) -> ParsedMarkdown {
    use pulldown_cmark::{HeadingLevel, Options};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    let events: Vec<Event<'_>> = pulldown_cmark::Parser::new_ext(content, options).collect();

    // Compile-time state
    let mut bold = false;
    let mut italic = false;
    let mut strikethrough = false;

    let mut heading_level: Option<HeadingLevel> = None;
    let mut in_code_block = false;
    let mut in_link: Option<String> = None;
    let mut in_image: Option<String> = None;
    let mut blockquote_depth: usize = 0;

    // List tracking: None = unordered, Some(next_number) = ordered
    let mut list_stack: Vec<Option<usize>> = Vec::new();

    // Table accumulation state
    let mut in_table = false;
    let mut _in_table_head = false;
    let mut table_num_columns: usize = 0;
    let mut table_header_cells: Vec<Vec<Fragment>> = Vec::new();
    let mut table_rows: Vec<Vec<Vec<Fragment>>> = Vec::new();
    let mut table_current_row: Vec<Vec<Fragment>> = Vec::new();
    let mut table_count: usize = 0;

    let mut widget_fields: Vec<WidgetField> = Vec::new();
    let mut references_state = false;
    let mut display_refs: Vec<String> = Vec::new();

    let mut pending_text = String::new();
    let mut fragments: Vec<Fragment> = Vec::new();
    let mut code_body: Vec<proc_macro2::TokenStream> = Vec::new();

    // Block directive stack (::: foreach, ::: if, ::: style)
    let mut block_stack: Vec<BlockFrame> = Vec::new();
    let mut needs_style_table = false;

    // Resolve spacing from frontmatter (compile-time constants)
    let sp = frontmatter.spacing.as_ref();
    let sp_paragraph = sp.and_then(|s| s.paragraph).unwrap_or(8.0_f32);
    let sp_table = sp.and_then(|s| s.table).unwrap_or(8.0_f32);
    let sp_h1 = sp.and_then(|s| s.heading_h1).unwrap_or(16.0_f32);
    let sp_h2 = sp.and_then(|s| s.heading_h2).unwrap_or(12.0_f32);
    let sp_h3 = sp.and_then(|s| s.heading_h3).unwrap_or(8.0_f32);
    let sp_h4 = sp.and_then(|s| s.heading_h4).unwrap_or(4.0_f32);
    let sp_item = sp.and_then(|s| s.item);

    fn flush_pending(
        pending_text: &mut String,
        fragments: &mut Vec<Fragment>,
        frontmatter: &Frontmatter,
        bold: bool,
        italic: bool,
        strikethrough: bool,
    ) {
        if pending_text.is_empty() {
            return;
        }
        let text = std::mem::take(pending_text);
        // Check for ::class(text) inline styled spans
        if text.contains("::") && text.contains('(') {
            if parse_inline_styled_spans(&text, fragments, frontmatter, bold, italic, strikethrough)
            {
                return;
            }
        }
        fragments.push(Fragment::Styled {
            text,
            bold,
            italic,
            strikethrough,
        });
    }

    fn fragment_to_tokens(f: &Fragment) -> proc_macro2::TokenStream {
        match f {
            Fragment::Styled {
                text,
                bold,
                italic,
                strikethrough,
            } => {
                quote! { styled_label(ui, #text, #bold, #italic, #strikethrough); }
            }
            Fragment::InlineCode(text) => {
                quote! { inline_code(ui, #text); }
            }
            Fragment::Link {
                text,
                url,
                bold,
                italic,
                strikethrough,
            } => {
                quote! { styled_hyperlink(ui, #text, #url, #bold, #italic, #strikethrough); }
            }
            Fragment::Widget(tokens) => tokens.clone(),
            Fragment::ForeachField(name) => {
                let field = syn::Ident::new(name, proc_macro2::Span::call_site());
                quote! { ui.label(format!("{}", __row.#field)); }
            }
        }
    }

    /// Emit accumulated fragments as an inline-wrapped paragraph.
    /// If the last fragment ends with `::key`, apply the frontmatter style to all fragments.
    /// Returns `Some(field_name)` when a runtime `::$field` suffix was found,
    /// signalling the caller to wrap the emitted code in a style override block.
    fn emit_paragraph(
        fragments: &mut Vec<Fragment>,
        code_body: &mut Vec<proc_macro2::TokenStream>,
        blockquote_depth: usize,
        frontmatter: &Frontmatter,
        paragraph_spacing: f32,
    ) -> Option<String> {
        if fragments.is_empty() {
            return None;
        }

        // Check if the last fragment's text ends with ::key
        let style_key = {
            let mut found = None;
            if let Some(Fragment::Styled { text, .. }) = fragments.last() {
                let (trimmed, key) = detect_style_suffix(text);
                if let Some(k) = key {
                    found = Some((trimmed.to_owned(), k.to_owned()));
                }
            }
            found
        };

        // Separate runtime ($-prefixed) from compile-time style keys
        let mut runtime_style_field: Option<String> = None;
        let resolved_style = if let Some((trimmed_text, ref key)) = style_key {
            if key.starts_with('$') {
                // Runtime style — strip $ and return the field name to caller
                let field_name = key[1..].to_owned();
                // Trim the last fragment
                if let Some(Fragment::Styled { text, .. }) = fragments.last_mut() {
                    *text = trimmed_text;
                    if text.is_empty() {
                        fragments.pop();
                    }
                }
                runtime_style_field = Some(field_name);
                None
            } else {
                let style = frontmatter
                    .styles
                    .get(key.as_str())
                    .unwrap_or_else(|| {
                        let hint = if frontmatter.widgets.contains_key(key) {
                            format!(" '{key}' is a widget config, not a style. Attach it to a widget directive like [slider](field){{{key}}}, not to a paragraph.")
                        } else {
                            String::new()
                        };
                        panic!("Undefined style key '{key}' in frontmatter.{hint}")
                    });
                if let Some(Fragment::Styled { text, .. }) = fragments.last_mut() {
                    *text = trimmed_text;
                    if text.is_empty() {
                        fragments.pop();
                    }
                }
                Some(style.clone())
            }
        } else {
            None
        };

        if fragments.is_empty() {
            return None;
        }

        let calls: Vec<proc_macro2::TokenStream> = if let Some(ref style) = resolved_style {
            // Apply frontmatter style to all fragments
            fragments
                .iter()
                .map(|f| match f {
                    Fragment::Styled {
                        text,
                        bold,
                        italic,
                        strikethrough,
                    } => {
                        let mut merged = style.clone();
                        if *bold {
                            merged.bold = Some(true);
                        }
                        if *italic {
                            merged.italic = Some(true);
                        }
                        if *strikethrough {
                            merged.strikethrough = Some(true);
                        }
                        style_def_to_label_tokens(text, &merged, *bold, *italic, *strikethrough)
                    }
                    other => fragment_to_tokens(other),
                })
                .collect()
        } else {
            fragments.iter().map(fragment_to_tokens).collect()
        };

        if blockquote_depth > 0 {
            let depth = blockquote_depth;
            // Extract color from resolved style for quote bar coloring
            let bar_color_tokens = if let Some(ref style) = resolved_style {
                if let Some(ref hex) = style.color {
                    let [r, g, b] = parse_hex_color(hex).expect("Invalid color in frontmatter");
                    quote! { Some([#r, #g, #b]) }
                } else {
                    quote! { None }
                }
            } else {
                quote! { None }
            };
            code_body.push(quote! {
                ui.horizontal_wrapped(|ui| {
                    emit_quote_bars_colored(ui, #depth, #bar_color_tokens);
                    #(#calls)*
                });
            });
        } else {
            code_body.push(quote! {
                ui.horizontal_wrapped(|ui| {
                    #(#calls)*
                });
            });
        }
        code_body.push(quote! { ui.add_space(#paragraph_spacing); });
        fragments.clear();
        runtime_style_field
    }

    /// Emit accumulated fragments as a list item (bullet or numbered).
    /// Each item is a separate top-level `ui.horizontal_wrapped(...)`.
    /// If the last fragment ends with `::key`, the style's color is applied to the
    /// bullet/number prefix and all text fragments get the full style treatment.
    /// Returns `Some(field_name)` when a runtime `::$field` suffix was found.
    fn emit_list_item(
        fragments: &mut Vec<Fragment>,
        code_body: &mut Vec<proc_macro2::TokenStream>,
        list_stack: &mut [Option<usize>],
        blockquote_depth: usize,
        frontmatter: &Frontmatter,
    ) -> Option<String> {
        if fragments.is_empty() {
            return None;
        }

        // Check if the last fragment's text ends with ::key
        let style_key = {
            let mut found = None;
            if let Some(Fragment::Styled { text, .. }) = fragments.last() {
                let (trimmed, key) = detect_style_suffix(text);
                if let Some(k) = key {
                    found = Some((trimmed.to_owned(), k.to_owned()));
                }
            }
            found
        };

        let mut runtime_style_field: Option<String> = None;
        let resolved_style = if let Some((trimmed_text, ref key)) = style_key {
            if key.starts_with('$') {
                let field_name = key[1..].to_owned();
                if let Some(Fragment::Styled { text, .. }) = fragments.last_mut() {
                    *text = trimmed_text;
                    if text.is_empty() {
                        fragments.pop();
                    }
                }
                runtime_style_field = Some(field_name);
                None
            } else {
                let style = frontmatter
                    .styles
                    .get(key.as_str())
                    .unwrap_or_else(|| {
                        let hint = if frontmatter.widgets.contains_key(key) {
                            format!(" '{key}' is a widget config, not a style. Attach it to a widget directive like [slider](field){{{key}}}, not to a paragraph.")
                        } else {
                            String::new()
                        };
                        panic!("Undefined style key '{key}' in frontmatter.{hint}")
                    });
                if let Some(Fragment::Styled { text, .. }) = fragments.last_mut() {
                    *text = trimmed_text;
                    if text.is_empty() {
                        fragments.pop();
                    }
                }
                Some(style.clone())
            }
        } else {
            None
        };

        if fragments.is_empty() {
            return None;
        }

        let calls: Vec<proc_macro2::TokenStream> = if let Some(ref style) = resolved_style {
            fragments
                .iter()
                .map(|f| match f {
                    Fragment::Styled {
                        text,
                        bold,
                        italic,
                        strikethrough,
                    } => {
                        let mut merged = style.clone();
                        if *bold {
                            merged.bold = Some(true);
                        }
                        if *italic {
                            merged.italic = Some(true);
                        }
                        if *strikethrough {
                            merged.strikethrough = Some(true);
                        }
                        style_def_to_label_tokens(text, &merged, *bold, *italic, *strikethrough)
                    }
                    other => fragment_to_tokens(other),
                })
                .collect()
        } else {
            fragments.iter().map(fragment_to_tokens).collect()
        };

        let depth = list_stack.len();

        // Extract color for prefix coloring
        let prefix_color_tokens = if let Some(ref style) = resolved_style {
            if let Some(ref hex) = style.color {
                let [r, g, b] = parse_hex_color(hex).expect("Invalid color in frontmatter");
                quote! { Some([#r, #g, #b]) }
            } else {
                quote! { None }
            }
        } else {
            quote! { None }
        };

        // Build the prefix + content as one horizontal row
        let prefix = match list_stack.last_mut() {
            Some(Some(n)) => {
                let num_str = n.to_string();
                *n += 1;
                quote! { emit_numbered_prefix_colored(ui, #depth, #num_str, #prefix_color_tokens); }
            }
            Some(None) => {
                quote! { emit_bullet_prefix_colored(ui, #depth, #prefix_color_tokens); }
            }
            None => quote! {},
        };

        if blockquote_depth > 0 {
            let bq = blockquote_depth;
            code_body.push(quote! {
                ui.horizontal_wrapped(|ui| {
                    emit_quote_bars(ui, #bq);
                    #prefix
                    #(#calls)*
                });
            });
        } else {
            code_body.push(quote! {
                ui.horizontal_wrapped(|ui| {
                    #prefix
                    #(#calls)*
                });
            });
        }
        fragments.clear();
        runtime_style_field
    }

    fn cells_to_tokens(cells: &[Vec<Fragment>]) -> Vec<proc_macro2::TokenStream> {
        cells
            .iter()
            .map(|cell| {
                let calls: Vec<proc_macro2::TokenStream> =
                    cell.iter().map(fragment_to_tokens).collect();
                if calls.len() == 1 {
                    calls.into_iter().next().unwrap_or_default()
                } else {
                    quote! { ui.horizontal_wrapped(|ui| { #(#calls)* }); }
                }
            })
            .collect()
    }

    fn emit_table(
        header_cells: &[Vec<Fragment>],
        rows: &[Vec<Vec<Fragment>>],
        num_columns: usize,
        table_id: usize,
        in_foreach: bool,
        code_body: &mut Vec<proc_macro2::TokenStream>,
        table_spacing: f32,
    ) {
        let id_str = format!("md_table_{table_id}");
        let ncols = num_columns;

        // Header: render cells as bold
        let header_tokens: Vec<proc_macro2::TokenStream> = header_cells
            .iter()
            .map(|cell| {
                // For header cells, render each fragment but force bold
                let calls: Vec<proc_macro2::TokenStream> = cell
                    .iter()
                    .map(|f| match f {
                        Fragment::Styled {
                            text,
                            italic,
                            strikethrough,
                            ..
                        } => {
                            // Force bold for headers
                            let i = *italic;
                            let s = *strikethrough;
                            quote! { styled_label(ui, #text, true, #i, #s); }
                        }
                        other => fragment_to_tokens(other),
                    })
                    .collect();
                if calls.len() == 1 {
                    calls.into_iter().next().unwrap_or_default()
                } else {
                    quote! { ui.horizontal_wrapped(|ui| { #(#calls)* }); }
                }
            })
            .collect();

        // Body rows
        let row_tokens: Vec<proc_macro2::TokenStream> = rows
            .iter()
            .map(|row| {
                let cell_tokens = cells_to_tokens(row);
                quote! {
                    #(#cell_tokens)*
                    ui.end_row();
                }
            })
            .collect();

        if in_foreach {
            // Inside a foreach loop: use a runtime ID that includes the row index
            // to avoid Grid ID collisions across iterations.
            // __row is the loop variable from the enclosing for loop.
            let base_id = id_str;
            code_body.push(quote! {
                ui.push_id(format!("{}_{:p}", #base_id, __row as *const _), |ui| {
                    egui::Grid::new(format!("{}_{:p}", #base_id, __row as *const _))
                        .num_columns(#ncols)
                        .striped(true)
                        .show(ui, |ui| {
                            #(#header_tokens)*
                            ui.end_row();
                            #(#row_tokens)*
                        });
                });
            });
        } else {
            code_body.push(quote! {
                egui::Grid::new(#id_str)
                    .num_columns(#ncols)
                    .striped(true)
                    .show(ui, |ui| {
                        #(#header_tokens)*
                        ui.end_row();
                        #(#row_tokens)*
                    });
            });
        }
        code_body.push(quote! { ui.add_space(#table_spacing); });
    }

    /// Parse text containing `{field}` references into alternating Styled and ForeachField
    /// fragments. Used inside foreach blocks for field substitution.
    fn parse_foreach_text(
        text: &str,
        fragments: &mut Vec<Fragment>,
        row_fields: &mut Vec<String>,
        bold: bool,
        italic: bool,
        strikethrough: bool,
    ) {
        let mut remaining = text;
        while let Some(open) = remaining.find('{') {
            if let Some(close) = remaining[open..].find('}') {
                let close = open + close;
                let field_name = remaining[open + 1..close].trim();
                if !field_name.is_empty()
                    && field_name.chars().all(|c| c.is_alphanumeric() || c == '_')
                {
                    // Emit text before the field ref
                    let before = &remaining[..open];
                    if !before.is_empty() {
                        fragments.push(Fragment::Styled {
                            text: before.to_owned(),
                            bold,
                            italic,
                            strikethrough,
                        });
                    }
                    // Emit the field reference
                    if !row_fields.contains(&field_name.to_owned()) {
                        row_fields.push(field_name.to_owned());
                    }
                    fragments.push(Fragment::ForeachField(field_name.to_owned()));
                    remaining = &remaining[close + 1..];
                    continue;
                }
            }
            // No valid field ref — emit rest as text
            break;
        }
        if !remaining.is_empty() {
            fragments.push(Fragment::Styled {
                text: remaining.to_owned(),
                bold,
                italic,
                strikethrough,
            });
        }
    }

    // Known widget names that intercept link syntax.
    // To add a new widget: add its name here, then add a match arm in the
    // `End(Link)` handler below.
    const WIDGET_NAMES: &[&str] = &[
        "button",
        "progress",
        "spinner",
        "slider",
        "double_slider",
        "checkbox",
        "textedit",
        "textarea",
        "password",
        "dragvalue",
        "display",
        "radio",
        "combobox",
        "color",
        "toggle",
        "selectable",
        "select",
        "log",
    ];

    fn is_widget_name(name: &str) -> bool {
        WIDGET_NAMES.contains(&name)
    }

    // --- Main event loop (index-based for lookahead) ---

    let mut skip_next = false;
    let mut event_idx = 0;
    while event_idx < events.len() {
        if skip_next {
            skip_next = false;
            event_idx += 1;
            continue;
        }
        let event = &events[event_idx];
        match event {
            Event::Start(tag) => match tag {
                Tag::Table(alignments) => {
                    in_table = true;
                    table_num_columns = alignments.len();
                    table_header_cells.clear();
                    table_rows.clear();
                    table_current_row.clear();
                }
                Tag::TableHead => {
                    _in_table_head = true;
                    table_current_row.clear();
                }
                Tag::TableRow => {
                    table_current_row.clear();
                }
                Tag::TableCell => {
                    // Start collecting fragments for this cell
                    pending_text.clear();
                    fragments.clear();
                }
                Tag::Paragraph if in_table => {
                    // Tables don't emit paragraphs
                }
                Tag::Paragraph => {}
                Tag::Heading(level, _, _) => {
                    heading_level = Some(*level);
                    pending_text.clear();
                }
                Tag::BlockQuote => {
                    blockquote_depth += 1;
                }
                Tag::List(start) => {
                    // Flush any pending content from the parent item before nesting.
                    // In tight lists, Text appears directly in Item (no Paragraph wrapper),
                    // so pending_text may contain the parent item's text when a sub-list starts.
                    flush_pending(
                        &mut pending_text,
                        &mut fragments,
                        frontmatter,
                        bold,
                        italic,
                        strikethrough,
                    );
                    if !fragments.is_empty() && !list_stack.is_empty() {
                        // Ignore runtime style signal in this flush context
                        let _ = emit_list_item(
                            &mut fragments,
                            &mut code_body,
                            &mut list_stack,
                            blockquote_depth,
                            frontmatter,
                        );
                    }
                    list_stack.push(start.map(|n| n as usize));
                }
                Tag::Item => {}
                Tag::Emphasis => {
                    flush_pending(
                        &mut pending_text,
                        &mut fragments,
                        frontmatter,
                        bold,
                        italic,
                        strikethrough,
                    );
                    italic = true;
                }
                Tag::Strong => {
                    flush_pending(
                        &mut pending_text,
                        &mut fragments,
                        frontmatter,
                        bold,
                        italic,
                        strikethrough,
                    );
                    bold = true;
                }
                Tag::Strikethrough => {
                    flush_pending(
                        &mut pending_text,
                        &mut fragments,
                        frontmatter,
                        bold,
                        italic,
                        strikethrough,
                    );
                    strikethrough = true;
                }
                Tag::CodeBlock(_info) => {
                    in_code_block = true;
                    pending_text.clear();
                }
                Tag::Link(_link_type, dest, _title) => {
                    flush_pending(
                        &mut pending_text,
                        &mut fragments,
                        frontmatter,
                        bold,
                        italic,
                        strikethrough,
                    );
                    in_link = Some(dest.to_string());
                    pending_text.clear();
                }
                Tag::Image(_link_type, dest, _title) => {
                    flush_pending(
                        &mut pending_text,
                        &mut fragments,
                        frontmatter,
                        bold,
                        italic,
                        strikethrough,
                    );
                    in_image = Some(dest.to_string());
                    pending_text.clear();
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                Tag::Table(_) => {
                    let in_foreach = matches!(
                        block_stack.last().map(|f| &f.directive),
                        Some(BlockDirective::Foreach { .. })
                    );
                    emit_table(
                        &table_header_cells,
                        &table_rows,
                        table_num_columns,
                        table_count,
                        in_foreach,
                        &mut code_body,
                        sp_table,
                    );
                    table_count += 1;
                    in_table = false;
                }
                Tag::TableHead => {
                    table_header_cells = std::mem::take(&mut table_current_row);
                    _in_table_head = false;
                }
                Tag::TableRow => {
                    table_rows.push(std::mem::take(&mut table_current_row));
                }
                Tag::TableCell => {
                    flush_pending(
                        &mut pending_text,
                        &mut fragments,
                        frontmatter,
                        bold,
                        italic,
                        strikethrough,
                    );
                    table_current_row.push(std::mem::take(&mut fragments));
                }
                Tag::Paragraph if in_table => {}
                Tag::Paragraph => {
                    flush_pending(
                        &mut pending_text,
                        &mut fragments,
                        frontmatter,
                        bold,
                        italic,
                        strikethrough,
                    );
                    let runtime_field = if !list_stack.is_empty() {
                        // Inside a list item — emit as a list item row
                        emit_list_item(
                            &mut fragments,
                            &mut code_body,
                            &mut list_stack,
                            blockquote_depth,
                            frontmatter,
                        )
                    } else {
                        // Standalone paragraph
                        emit_paragraph(
                            &mut fragments,
                            &mut code_body,
                            blockquote_depth,
                            frontmatter,
                            sp_paragraph,
                        )
                    };
                    // Wrap emitted code in runtime style override if ::$field was found
                    if let Some(ref field_name) = runtime_field {
                        needs_style_table = true;
                        references_state = true;
                        let already = widget_fields.iter().any(|f| f.name() == *field_name);
                        if !already {
                            widget_fields.push(WidgetField::Stateful {
                                name: field_name.clone(),
                                ty: WidgetType::String,
                            });
                        }
                        let field_ident =
                            syn::Ident::new(field_name, proc_macro2::Span::call_site());
                        // Pop the emitted entries and wrap in style override
                        let emitted: Vec<_> = code_body
                            .drain(code_body.len().saturating_sub(2)..)
                            .collect();
                        code_body.push(quote! {
                            {
                                let __style_color = __resolve_style_color(&state.#field_ident);
                                if let Some(__c) = __style_color {
                                    ui.visuals_mut().override_text_color = Some(__c);
                                }
                                #(#emitted)*
                            }
                        });
                    }
                }
                Tag::Heading(level, _, _) => {
                    let raw_text = std::mem::take(&mut pending_text);
                    let (text, style_key) = detect_style_suffix(&raw_text);
                    let text = text.to_owned();

                    if let Some(key) = style_key {
                        if let Some(style) = frontmatter.styles.get(key) {
                            // Custom-styled heading: emit styled_label_rich with
                            // heading defaults merged with frontmatter overrides
                            let default_size = match level {
                                HeadingLevel::H1 => 28.0_f32,
                                HeadingLevel::H2 => 22.0_f32,
                                HeadingLevel::H3 => 18.0_f32,
                                _ => 14.0_f32,
                            };
                            let mut merged = style.clone();
                            if merged.bold.is_none() {
                                merged.bold = Some(true);
                            }
                            if merged.size.is_none() {
                                merged.size = Some(default_size);
                            }
                            // Add top spacing before styled headings
                            let heading_space = match level {
                                HeadingLevel::H1 => sp_h1,
                                HeadingLevel::H2 => sp_h2,
                                HeadingLevel::H3 => sp_h3,
                                _ => sp_h4,
                            };
                            code_body.push(quote! { ui.add_space(#heading_space); });
                            let tokens =
                                style_def_to_label_tokens(&text, &merged, true, false, false);
                            code_body.push(tokens);
                        } else {
                            {
                                let hint = if frontmatter.widgets.contains_key(key) {
                                    format!(
                                        " '{key}' is a widget config, not a style. Attach it to a widget directive like [slider](field){{{key}}}, not to a paragraph."
                                    )
                                } else {
                                    String::new()
                                };
                                panic!("Undefined style key '{key}' in frontmatter.{hint}")
                            };
                        }
                    } else {
                        // Add top spacing before headings for visual breathing room
                        let heading_space = match level {
                            HeadingLevel::H1 => sp_h1,
                            HeadingLevel::H2 => sp_h2,
                            HeadingLevel::H3 => sp_h3,
                            _ => sp_h4,
                        };
                        code_body.push(quote! { ui.add_space(#heading_space); });
                        code_body.push(match level {
                            HeadingLevel::H1 => quote! { h1(ui, #text); },
                            HeadingLevel::H2 => quote! { h2(ui, #text); },
                            HeadingLevel::H3 => quote! { h3(ui, #text); },
                            _ => quote! { body(ui, #text); },
                        });
                    }
                    heading_level = None;
                }
                Tag::BlockQuote => {
                    blockquote_depth = blockquote_depth.saturating_sub(1);
                }
                Tag::List(_) => {
                    list_stack.pop();
                }
                Tag::Item => {
                    // Flush any leftover content (edge case: item without paragraph)
                    flush_pending(
                        &mut pending_text,
                        &mut fragments,
                        frontmatter,
                        bold,
                        italic,
                        strikethrough,
                    );
                    if !fragments.is_empty() {
                        let runtime_field = emit_list_item(
                            &mut fragments,
                            &mut code_body,
                            &mut list_stack,
                            blockquote_depth,
                            frontmatter,
                        );
                        if let Some(ref field_name) = runtime_field {
                            needs_style_table = true;
                            references_state = true;
                            let already = widget_fields.iter().any(|f| f.name() == *field_name);
                            if !already {
                                widget_fields.push(WidgetField::Stateful {
                                    name: field_name.clone(),
                                    ty: WidgetType::String,
                                });
                            }
                            let field_ident =
                                syn::Ident::new(field_name, proc_macro2::Span::call_site());
                            let emitted = code_body.pop();
                            if let Some(emitted) = emitted {
                                code_body.push(quote! {
                                    {
                                        let __style_color = __resolve_style_color(&state.#field_ident);
                                        if let Some(__c) = __style_color {
                                            ui.visuals_mut().override_text_color = Some(__c);
                                        }
                                        #emitted
                                    }
                                });
                            }
                        }
                    }
                }
                Tag::Emphasis => {
                    flush_pending(
                        &mut pending_text,
                        &mut fragments,
                        frontmatter,
                        bold,
                        italic,
                        strikethrough,
                    );
                    italic = false;
                }
                Tag::Strong => {
                    flush_pending(
                        &mut pending_text,
                        &mut fragments,
                        frontmatter,
                        bold,
                        italic,
                        strikethrough,
                    );
                    bold = false;
                }
                Tag::Strikethrough => {
                    flush_pending(
                        &mut pending_text,
                        &mut fragments,
                        frontmatter,
                        bold,
                        italic,
                        strikethrough,
                    );
                    strikethrough = false;
                }
                Tag::CodeBlock(_info) => {
                    let code_text = std::mem::take(&mut pending_text);
                    code_body.push(quote! { code(ui, #code_text); });
                    in_code_block = false;
                }
                Tag::Link(_link_type, dest, _title) => {
                    let raw_link_text = std::mem::take(&mut pending_text);
                    let url = dest.to_string();
                    let selector = parse_selectors(&raw_link_text);

                    if is_widget_name(&selector.base_name) {
                        // Widget directive detected — intercept link as widget
                        in_link = None;

                        // Lookahead for {attrs} in the next Text event
                        let mut widget_attrs = String::new();
                        if let Some(Event::Text(next_text)) = events.get(event_idx + 1) {
                            let t = next_text.as_ref().trim();
                            if t.starts_with('{') && t.ends_with('}') {
                                widget_attrs = t[1..t.len() - 1].to_string();
                                skip_next = true; // consume the {attrs} event
                            }
                        }

                        // Resolve class-based styles from selectors
                        let class_style = resolve_classes(&selector.classes, frontmatter);

                        // {key} is now widget config only — use .class for styling
                        let style = class_style;

                        // Emit widget code
                        // For display widgets (button): convert underscores to spaces
                        // For stateful widgets: use raw URL as the field name
                        let content = url.clone();
                        let display_content = url.replace('_', " ");
                        let link_text = &selector.base_name;
                        let widget_code = match link_text.as_str() {
                            "button" => {
                                // Build the button expression (plain or styled)
                                let button_expr = if let Some(s) = &style {
                                    let bold_val = s.bold.unwrap_or(false);
                                    let italic_val = s.italic.unwrap_or(false);
                                    let strike_val = s.strikethrough.unwrap_or(false);
                                    let color_tokens = match &s.color {
                                        Some(hex) => {
                                            let [r, g, b] =
                                                parse_hex_color(hex).expect("Invalid button color");
                                            quote! { .color(egui::Color32::from_rgb(#r, #g, #b)) }
                                        }
                                        None => quote! {},
                                    };
                                    let size_tokens = match s.size {
                                        Some(sz) => quote! { .size(#sz) },
                                        None => quote! {},
                                    };
                                    quote! {{
                                        let mut rt = egui::RichText::new(#display_content);
                                        if #bold_val { rt = rt.strong(); }
                                        if #italic_val { rt = rt.italics(); }
                                        if #strike_val { rt = rt.strikethrough(); }
                                        rt = rt #color_tokens #size_tokens;
                                        ui.button(rt)
                                    }}
                                } else {
                                    quote! { ui.button(#display_content) }
                                };

                                // Stateful button: {config} present → click counter + optional advanced responses
                                if !widget_attrs.is_empty() {
                                    let wdef = frontmatter
                                        .widgets
                                        .get(&widget_attrs)
                                        .cloned()
                                        .unwrap_or_default();

                                    let count_name = format!("{widget_attrs}_count");
                                    widget_fields.push(WidgetField::Stateful {
                                        name: count_name.clone(),
                                        ty: WidgetType::U32,
                                    });
                                    let count_field = syn::Ident::new(
                                        &count_name,
                                        proc_macro2::Span::call_site(),
                                    );

                                    // Optional hover tracking
                                    let hover_code = if wdef.track_hover.unwrap_or(false) {
                                        let hover_name = format!("{widget_attrs}_hovered");
                                        widget_fields.push(WidgetField::Stateful {
                                            name: hover_name.clone(),
                                            ty: WidgetType::Bool,
                                        });
                                        let hover_field = syn::Ident::new(
                                            &hover_name,
                                            proc_macro2::Span::call_site(),
                                        );
                                        quote! {
                                            state.#hover_field = __btn_resp.hovered();
                                        }
                                    } else {
                                        quote! {}
                                    };

                                    // Optional secondary click tracking
                                    let secondary_code = if wdef.track_secondary.unwrap_or(false) {
                                        let sec_name = format!("{widget_attrs}_secondary_count");
                                        widget_fields.push(WidgetField::Stateful {
                                            name: sec_name.clone(),
                                            ty: WidgetType::U32,
                                        });
                                        let sec_field = syn::Ident::new(
                                            &sec_name,
                                            proc_macro2::Span::call_site(),
                                        );
                                        quote! {
                                            if __btn_resp.secondary_clicked() {
                                                state.#sec_field += 1;
                                            }
                                        }
                                    } else {
                                        quote! {}
                                    };

                                    quote! {
                                        {
                                            let __btn_resp = #button_expr;
                                            if __btn_resp.clicked() {
                                                state.#count_field += 1;
                                            }
                                            #hover_code
                                            #secondary_code
                                        }
                                    }
                                } else {
                                    // Stateless button: suppress #[must_use] warning
                                    quote! { let _ = #button_expr; }
                                }
                            }
                            "progress" => {
                                if let Ok(val) = content.parse::<f32>() {
                                    // Stateless: literal value (backwards compatible)
                                    quote! {
                                        ui.add(egui::ProgressBar::new(#val).show_percentage());
                                    }
                                } else {
                                    // Stateful: reads f64 from AppState
                                    widget_fields.push(WidgetField::Stateful {
                                        name: content.clone(),
                                        ty: WidgetType::F64,
                                    });
                                    let field =
                                        syn::Ident::new(&content, proc_macro2::Span::call_site());
                                    let wdef = get_widget_def(&widget_attrs, frontmatter);
                                    let fill_tokens = if let Some(ref hex) = wdef.fill {
                                        let [r, g, b] = parse_hex_color(hex)
                                            .expect("Invalid fill color in widget config");
                                        quote! { .fill(egui::Color32::from_rgb(#r, #g, #b)) }
                                    } else {
                                        quote! {}
                                    };
                                    quote! {
                                        ui.add(
                                            egui::ProgressBar::new(state.#field as f32)
                                                .show_percentage()
                                                #fill_tokens
                                        );
                                    }
                                }
                            }
                            "spinner" => {
                                quote! { ui.spinner(); }
                            }
                            "slider" => {
                                widget_fields.push(WidgetField::Stateful {
                                    name: content.clone(),
                                    ty: WidgetType::F64,
                                });
                                let field =
                                    syn::Ident::new(&content, proc_macro2::Span::call_site());
                                let wdef = if !widget_attrs.is_empty() {
                                    frontmatter
                                        .widgets
                                        .get(&widget_attrs)
                                        .cloned()
                                        .unwrap_or_default()
                                } else {
                                    WidgetDef::default()
                                };
                                let min_val = wdef.min.unwrap_or(0.0);
                                let max_val = wdef.max.unwrap_or(1.0);
                                let label = wdef.label.unwrap_or_default();
                                let suffix = wdef.suffix.unwrap_or_default();
                                let prefix = wdef.prefix.unwrap_or_default();
                                quote! {
                                    ui.add(
                                        egui::Slider::new(&mut state.#field, #min_val..=#max_val)
                                            .text(#label)
                                            .suffix(#suffix)
                                            .prefix(#prefix)
                                    );
                                }
                            }
                            "double_slider" => {
                                let low_name = format!("{content}_low");
                                let high_name = format!("{content}_high");
                                widget_fields.push(WidgetField::Stateful {
                                    name: low_name.clone(),
                                    ty: WidgetType::F64,
                                });
                                widget_fields.push(WidgetField::Stateful {
                                    name: high_name.clone(),
                                    ty: WidgetType::F64,
                                });
                                let low_field =
                                    syn::Ident::new(&low_name, proc_macro2::Span::call_site());
                                let high_field =
                                    syn::Ident::new(&high_name, proc_macro2::Span::call_site());
                                let wdef = if !widget_attrs.is_empty() {
                                    frontmatter
                                        .widgets
                                        .get(&widget_attrs)
                                        .cloned()
                                        .unwrap_or_default()
                                } else {
                                    WidgetDef::default()
                                };
                                let min_val = wdef.min.unwrap_or(0.0);
                                let max_val = wdef.max.unwrap_or(1.0);
                                quote! {
                                    ui.add(
                                        egui_double_slider::DoubleSlider::new(
                                            &mut state.#low_field,
                                            &mut state.#high_field,
                                            #min_val..=#max_val,
                                        )
                                    );
                                }
                            }
                            "checkbox" => {
                                widget_fields.push(WidgetField::Stateful {
                                    name: content.clone(),
                                    ty: WidgetType::Bool,
                                });
                                let field =
                                    syn::Ident::new(&content, proc_macro2::Span::call_site());
                                let label = if !widget_attrs.is_empty() {
                                    frontmatter
                                        .widgets
                                        .get(&widget_attrs)
                                        .and_then(|w| w.label.clone())
                                        .unwrap_or(content.clone())
                                } else {
                                    content.clone()
                                };
                                quote! {
                                    ui.checkbox(&mut state.#field, #label);
                                }
                            }
                            "textedit" => {
                                widget_fields.push(WidgetField::Stateful {
                                    name: content.clone(),
                                    ty: WidgetType::String,
                                });
                                let field =
                                    syn::Ident::new(&content, proc_macro2::Span::call_site());
                                let hint = if !widget_attrs.is_empty() {
                                    frontmatter
                                        .widgets
                                        .get(&widget_attrs)
                                        .and_then(|w| w.hint.clone())
                                        .unwrap_or_default()
                                } else {
                                    String::new()
                                };
                                if hint.is_empty() {
                                    quote! {
                                        ui.text_edit_singleline(&mut state.#field);
                                    }
                                } else {
                                    quote! {
                                        ui.add(egui::TextEdit::singleline(&mut state.#field).hint_text(#hint));
                                    }
                                }
                            }
                            "dragvalue" => {
                                widget_fields.push(WidgetField::Stateful {
                                    name: content.clone(),
                                    ty: WidgetType::F64,
                                });
                                let field =
                                    syn::Ident::new(&content, proc_macro2::Span::call_site());
                                let wdef = if !widget_attrs.is_empty() {
                                    frontmatter
                                        .widgets
                                        .get(&widget_attrs)
                                        .cloned()
                                        .unwrap_or_default()
                                } else {
                                    WidgetDef::default()
                                };
                                let speed = wdef.speed.unwrap_or(0.1);
                                quote! {
                                    ui.add(egui::DragValue::new(&mut state.#field).speed(#speed));
                                }
                            }
                            "display" => {
                                references_state = true;
                                display_refs.push(content.clone());
                                // Self-declare the field as String if no input widget
                                // has already declared it. This allows display-only
                                // fields populated from code (ECS systems, etc.).
                                let already_declared =
                                    widget_fields.iter().any(|f| f.name() == content);
                                if !already_declared {
                                    widget_fields.push(WidgetField::Stateful {
                                        name: content.clone(),
                                        ty: WidgetType::String,
                                    });
                                }
                                let field =
                                    syn::Ident::new(&content, proc_macro2::Span::call_site());
                                let wdef = if !widget_attrs.is_empty() {
                                    frontmatter
                                        .widgets
                                        .get(&widget_attrs)
                                        .cloned()
                                        .unwrap_or_default()
                                } else {
                                    WidgetDef::default()
                                };
                                let fmt = wdef.format.as_deref().unwrap_or("{}");
                                quote! {
                                    ui.label(format!(#fmt, state.#field));
                                }
                            }
                            "radio" => {
                                widget_fields.push(WidgetField::Stateful {
                                    name: content.clone(),
                                    ty: WidgetType::Usize,
                                });
                                let field =
                                    syn::Ident::new(&content, proc_macro2::Span::call_site());
                                let wdef = if !widget_attrs.is_empty() {
                                    frontmatter
                                        .widgets
                                        .get(&widget_attrs)
                                        .cloned()
                                        .unwrap_or_default()
                                } else {
                                    WidgetDef::default()
                                };
                                let options = wdef
                                    .options
                                    .unwrap_or_else(|| vec!["Option A".into(), "Option B".into()]);
                                let radio_calls: Vec<proc_macro2::TokenStream> = options
                                    .iter()
                                    .enumerate()
                                    .map(|(i, opt)| {
                                        quote! {
                                            ui.radio_value(&mut state.#field, #i, #opt);
                                        }
                                    })
                                    .collect();
                                quote! {
                                    #(#radio_calls)*
                                }
                            }
                            "combobox" => {
                                widget_fields.push(WidgetField::Stateful {
                                    name: content.clone(),
                                    ty: WidgetType::Usize,
                                });
                                let field =
                                    syn::Ident::new(&content, proc_macro2::Span::call_site());
                                let wdef = if !widget_attrs.is_empty() {
                                    frontmatter
                                        .widgets
                                        .get(&widget_attrs)
                                        .cloned()
                                        .unwrap_or_default()
                                } else {
                                    WidgetDef::default()
                                };
                                let options = wdef
                                    .options
                                    .unwrap_or_else(|| vec!["Option A".into(), "Option B".into()]);
                                let label = wdef.label.unwrap_or_else(|| display_content.clone());
                                let num_options = options.len();
                                quote! {
                                    {
                                        const OPTIONS: &[&str] = &[#(#options),*];
                                        egui::ComboBox::from_label(#label)
                                            .selected_text(OPTIONS[state.#field])
                                            .show_index(ui, &mut state.#field, #num_options, |i| OPTIONS[i]);
                                    }
                                }
                            }
                            "color" => {
                                widget_fields.push(WidgetField::Stateful {
                                    name: content.clone(),
                                    ty: WidgetType::ByteArray4,
                                });
                                let field =
                                    syn::Ident::new(&content, proc_macro2::Span::call_site());
                                quote! {
                                    {
                                        let mut __c = egui::Color32::from_rgba_unmultiplied(
                                            state.#field[0], state.#field[1],
                                            state.#field[2], state.#field[3],
                                        );
                                        egui::color_picker::color_edit_button_srgba(
                                            ui, &mut __c, egui::color_picker::Alpha::Opaque,
                                        );
                                        state.#field = [__c.r(), __c.g(), __c.b(), __c.a()];
                                    }
                                }
                            }
                            "textarea" => {
                                widget_fields.push(WidgetField::Stateful {
                                    name: content.clone(),
                                    ty: WidgetType::String,
                                });
                                let field =
                                    syn::Ident::new(&content, proc_macro2::Span::call_site());
                                let wdef = if !widget_attrs.is_empty() {
                                    frontmatter
                                        .widgets
                                        .get(&widget_attrs)
                                        .cloned()
                                        .unwrap_or_default()
                                } else {
                                    WidgetDef::default()
                                };
                                let hint = wdef.hint.unwrap_or_default();
                                let rows = wdef.rows.unwrap_or(4);
                                quote! {
                                    ui.add(
                                        egui::TextEdit::multiline(&mut state.#field)
                                            .hint_text(#hint)
                                            .desired_rows(#rows)
                                    );
                                }
                            }
                            "password" => {
                                widget_fields.push(WidgetField::Stateful {
                                    name: content.clone(),
                                    ty: WidgetType::String,
                                });
                                let field =
                                    syn::Ident::new(&content, proc_macro2::Span::call_site());
                                let hint = if !widget_attrs.is_empty() {
                                    frontmatter
                                        .widgets
                                        .get(&widget_attrs)
                                        .and_then(|w| w.hint.clone())
                                        .unwrap_or_default()
                                } else {
                                    String::new()
                                };
                                quote! {
                                    ui.add(
                                        egui::TextEdit::singleline(&mut state.#field)
                                            .password(true)
                                            .hint_text(#hint)
                                    );
                                }
                            }
                            "toggle" => {
                                widget_fields.push(WidgetField::Stateful {
                                    name: content.clone(),
                                    ty: WidgetType::Bool,
                                });
                                let field =
                                    syn::Ident::new(&content, proc_macro2::Span::call_site());
                                let label = if !widget_attrs.is_empty() {
                                    frontmatter
                                        .widgets
                                        .get(&widget_attrs)
                                        .and_then(|w| w.label.clone())
                                        .unwrap_or_default()
                                } else {
                                    String::new()
                                };
                                if label.is_empty() {
                                    quote! {
                                        toggle_switch(ui, &mut state.#field);
                                    }
                                } else {
                                    quote! {
                                        ui.horizontal(|ui| {
                                            toggle_switch(ui, &mut state.#field);
                                            ui.label(#label);
                                        });
                                    }
                                }
                            }
                            "selectable" => {
                                widget_fields.push(WidgetField::Stateful {
                                    name: content.clone(),
                                    ty: WidgetType::Usize,
                                });
                                let field =
                                    syn::Ident::new(&content, proc_macro2::Span::call_site());
                                let wdef = if !widget_attrs.is_empty() {
                                    frontmatter
                                        .widgets
                                        .get(&widget_attrs)
                                        .cloned()
                                        .unwrap_or_default()
                                } else {
                                    WidgetDef::default()
                                };
                                let options = wdef
                                    .options
                                    .unwrap_or_else(|| vec!["Option A".into(), "Option B".into()]);
                                let selectable_calls: Vec<proc_macro2::TokenStream> = options
                                    .iter()
                                    .enumerate()
                                    .map(|(i, opt)| {
                                        quote! {
                                            ui.selectable_value(&mut state.#field, #i, #opt);
                                        }
                                    })
                                    .collect();
                                quote! {
                                    ui.horizontal(|ui| {
                                        #(#selectable_calls)*
                                    });
                                }
                            }
                            "select" => {
                                let index_name = content.clone();
                                widget_fields.push(WidgetField::Stateful {
                                    name: index_name.clone(),
                                    ty: WidgetType::Usize,
                                });
                                if !widget_attrs.is_empty() {
                                    widget_fields.push(WidgetField::Stateful {
                                        name: widget_attrs.clone(),
                                        ty: WidgetType::VecString,
                                    });
                                }
                                let index_field =
                                    syn::Ident::new(&index_name, proc_macro2::Span::call_site());
                                let list_field =
                                    syn::Ident::new(&widget_attrs, proc_macro2::Span::call_site());
                                let wdef = if !widget_attrs.is_empty() {
                                    frontmatter
                                        .widgets
                                        .get(&widget_attrs)
                                        .cloned()
                                        .unwrap_or_default()
                                } else {
                                    WidgetDef::default()
                                };
                                let max_h = wdef.max_height.unwrap_or(200.0) as f32;
                                quote! {
                                    egui::ScrollArea::vertical()
                                        .max_height(#max_h)
                                        .show(ui, |ui| {
                                            for (__i, __label) in state.#list_field.iter().enumerate() {
                                                if ui.selectable_label(
                                                    state.#index_field == __i, __label
                                                ).clicked() {
                                                    state.#index_field = __i;
                                                }
                                            }
                                        });
                                }
                            }
                            "log" => {
                                widget_fields.push(WidgetField::Stateful {
                                    name: content.clone(),
                                    ty: WidgetType::VecString,
                                });
                                let field =
                                    syn::Ident::new(&content, proc_macro2::Span::call_site());
                                let wdef = get_widget_def(&widget_attrs, frontmatter);
                                let max_h = wdef.max_height.unwrap_or(200.0) as f32;
                                quote! {
                                    egui::ScrollArea::vertical()
                                        .max_height(#max_h)
                                        .stick_to_bottom(true)
                                        .show(ui, |ui| {
                                            for __msg in &state.#field {
                                                ui.label(__msg.as_str());
                                            }
                                        });
                                }
                            }
                            _ => {
                                // Unknown widget — fall back to label with widget name
                                quote! {
                                    ui.label(format!("[{}]({})", #link_text, #content));
                                }
                            }
                        };
                        // Wrap with push_id if selector has an ID
                        let widget_code = if let Some(ref id_str) = selector.id {
                            quote! {
                                ui.push_id(#id_str, |ui| { #widget_code });
                            }
                        } else {
                            widget_code
                        };
                        // Table-aware emission: widgets inside table cells go to
                        // fragments so they render inside the Grid closure.
                        if in_table {
                            fragments.push(Fragment::Widget(widget_code));
                        } else {
                            code_body.push(widget_code);
                        }
                    } else {
                        let class_style = resolve_classes(&selector.classes, frontmatter);

                        if selector.base_name.is_empty() && class_style.is_some() {
                            // Styled inline text span via class selector: [.premium](Hello World)
                            let display_content = url.replace('_', " ");
                            let s = class_style.unwrap();
                            let tokens = style_def_to_label_tokens(
                                &display_content,
                                &s,
                                bold,
                                italic,
                                strikethrough,
                            );
                            fragments.push(Fragment::Widget(tokens));
                        } else {
                            // Normal hyperlink — apply class bold/italic/strikethrough if present
                            let display_text = selector.base_name;
                            let (link_bold, link_italic, link_strike) = if let Some(s) = class_style
                            {
                                (
                                    s.bold.unwrap_or(bold),
                                    s.italic.unwrap_or(italic),
                                    s.strikethrough.unwrap_or(strikethrough),
                                )
                            } else {
                                (bold, italic, strikethrough)
                            };
                            fragments.push(Fragment::Link {
                                text: display_text,
                                url,
                                bold: link_bold,
                                italic: link_italic,
                                strikethrough: link_strike,
                            });
                        }
                        in_link = None;
                    }
                }
                Tag::Image(_link_type, _dest, _title) => {
                    let alt = std::mem::take(&mut pending_text);
                    let url = in_image.take().unwrap_or_default();
                    // Resolve relative paths against CARGO_MANIFEST_DIR at compile time
                    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
                    let abs_url = if url.starts_with("http://")
                        || url.starts_with("https://")
                        || url.starts_with("file://")
                    {
                        url
                    } else {
                        format!("file://{manifest_dir}/{url}")
                    };
                    let image_code = if alt.is_empty() {
                        quote! {
                            ui.add(egui::Image::new(#abs_url));
                        }
                    } else {
                        quote! {
                            ui.add(egui::Image::new(#abs_url).alt_text(#alt));
                        }
                    };
                    if in_table {
                        fragments.push(Fragment::Widget(image_code));
                    } else {
                        code_body.push(image_code);
                    }
                }
                _ => {}
            },
            Event::Text(text) => {
                let text_str: &str = text.as_ref();
                let trimmed = text_str.trim();

                // Detect ::: block fence directives
                if trimmed.starts_with(":::") {
                    let rest = trimmed[3..].trim();
                    if rest == "next" {
                        // Column separator: save current column, advance to next
                        if let Some(frame) = block_stack.last_mut() {
                            if let BlockDirective::Columns {
                                count,
                                current_col,
                                column_bodies,
                            } = &mut frame.directive
                            {
                                column_bodies[*current_col] = std::mem::take(&mut code_body);
                                *current_col += 1;
                                if *current_col >= *count {
                                    panic!(
                                        "::: next used too many times — only {} columns defined",
                                        count
                                    );
                                }
                            } else {
                                panic!("::: next can only be used inside ::: columns");
                            }
                        } else {
                            panic!("::: next outside of any block directive");
                        }
                        event_idx += 1;
                        continue;
                    }
                    if rest.is_empty() || rest.starts_with('/') {
                        // Close the innermost block
                        if let Some(frame) = block_stack.pop() {
                            let body = std::mem::take(&mut code_body);
                            code_body = frame.saved_code_body;

                            match frame.directive {
                                BlockDirective::Foreach { row_fields } => {
                                    if row_fields.is_empty() {
                                        panic!(
                                            "[foreach]({}) body contains no {{field}} references. \
                                             Ensure blank lines surround the table or list inside the \
                                             foreach block — CommonMark requires paragraph separation \
                                             for block-level elements like tables.",
                                            frame.field_name
                                        );
                                    }
                                    widget_fields.push(WidgetField::Foreach {
                                        name: frame.field_name.clone(),
                                        row_fields,
                                    });
                                    let field_ident = syn::Ident::new(
                                        &frame.field_name,
                                        proc_macro2::Span::call_site(),
                                    );
                                    code_body.push(quote! {
                                        for __row in &state.#field_ident {
                                            #(#body)*
                                        }
                                    });
                                }
                                BlockDirective::If => {
                                    let field_ident = syn::Ident::new(
                                        &frame.field_name,
                                        proc_macro2::Span::call_site(),
                                    );
                                    code_body.push(quote! {
                                        if state.#field_ident {
                                            #(#body)*
                                        }
                                    });
                                }
                                BlockDirective::Style => {
                                    needs_style_table = true;
                                    let field_ident = syn::Ident::new(
                                        &frame.field_name,
                                        proc_macro2::Span::call_site(),
                                    );
                                    code_body.push(quote! {
                                        {
                                            let __style_color = __resolve_style_color(&state.#field_ident);
                                            if let Some(__c) = __style_color {
                                                ui.visuals_mut().override_text_color = Some(__c);
                                            }
                                            #(#body)*
                                        }
                                    });
                                }
                                BlockDirective::Horizontal => {
                                    code_body.push(quote! {
                                        ui.horizontal(|ui| {
                                            #(#body)*
                                        });
                                    });
                                }
                                BlockDirective::Columns {
                                    count,
                                    current_col,
                                    mut column_bodies,
                                } => {
                                    // Save the last column's body
                                    column_bodies[current_col] = body;
                                    let col_count = count;
                                    let col_tokens: Vec<proc_macro2::TokenStream> = column_bodies
                                        .iter()
                                        .enumerate()
                                        .map(|(i, col_body)| {
                                            quote! {
                                                cols[#i].vertical(|ui| {
                                                    #(#col_body)*
                                                });
                                            }
                                        })
                                        .collect();
                                    code_body.push(quote! {
                                        ui.columns(#col_count, |cols| {
                                            #(#col_tokens)*
                                        });
                                    });
                                }
                                BlockDirective::Frame { style } => {
                                    let padding =
                                        style.as_ref().and_then(|s| s.inner_margin).unwrap_or(8.0);
                                    let outer =
                                        style.as_ref().and_then(|s| s.outer_margin).unwrap_or(0.0);
                                    let stroke_w =
                                        style.as_ref().and_then(|s| s.stroke).unwrap_or(0.0);
                                    let stroke_c = style
                                        .as_ref()
                                        .and_then(|s| s.stroke_color.as_ref())
                                        .map(|hex| {
                                            let [r, g, b] =
                                                parse_hex_color(hex).expect("Invalid stroke color");
                                            quote! { egui::Color32::from_rgb(#r, #g, #b) }
                                        })
                                        .unwrap_or_else(|| quote! { egui::Color32::TRANSPARENT });
                                    let radius =
                                        style.as_ref().and_then(|s| s.corner_radius).unwrap_or(0.0);
                                    let bg = style
                                        .as_ref()
                                        .and_then(|s| s.background.as_ref())
                                        .map(|hex| {
                                            let [r, g, b] = parse_hex_color(hex)
                                                .expect("Invalid background color");
                                            quote! { egui::Color32::from_rgb(#r, #g, #b) }
                                        })
                                        .unwrap_or_else(|| quote! { egui::Color32::TRANSPARENT });
                                    code_body.push(quote! {
                                        egui::Frame::default()
                                            .inner_margin(#padding)
                                            .outer_margin(#outer)
                                            .fill(#bg)
                                            .stroke(egui::Stroke::new(#stroke_w, #stroke_c))
                                            .corner_radius(#radius)
                                            .show(ui, |ui| {
                                                #(#body)*
                                            });
                                    });
                                }
                            }
                        }
                    } else {
                        // Open a new block: "foreach items", "if has_orb", "style hp_style"
                        let (directive_name, arg) = rest.split_once(' ').unwrap_or((rest, ""));
                        let field_name = arg.to_string();

                        match directive_name {
                            "foreach" => {
                                block_stack.push(BlockFrame {
                                    directive: BlockDirective::Foreach {
                                        row_fields: Vec::new(),
                                    },
                                    field_name,
                                    saved_code_body: std::mem::take(&mut code_body),
                                });
                            }
                            "if" => {
                                let already = widget_fields.iter().any(|f| f.name() == field_name);
                                if !already {
                                    widget_fields.push(WidgetField::Stateful {
                                        name: field_name.clone(),
                                        ty: WidgetType::Bool,
                                    });
                                }
                                block_stack.push(BlockFrame {
                                    directive: BlockDirective::If,
                                    field_name,
                                    saved_code_body: std::mem::take(&mut code_body),
                                });
                            }
                            "style" => {
                                needs_style_table = true;
                                let already = widget_fields.iter().any(|f| f.name() == field_name);
                                if !already {
                                    widget_fields.push(WidgetField::Stateful {
                                        name: field_name.clone(),
                                        ty: WidgetType::String,
                                    });
                                }
                                references_state = true;
                                block_stack.push(BlockFrame {
                                    directive: BlockDirective::Style,
                                    field_name,
                                    saved_code_body: std::mem::take(&mut code_body),
                                });
                            }
                            "frame" => {
                                let style = if !field_name.is_empty() {
                                    frontmatter.styles.get(&field_name).cloned()
                                } else {
                                    None
                                };
                                block_stack.push(BlockFrame {
                                    directive: BlockDirective::Frame { style },
                                    field_name,
                                    saved_code_body: std::mem::take(&mut code_body),
                                });
                            }
                            "horizontal" => {
                                block_stack.push(BlockFrame {
                                    directive: BlockDirective::Horizontal,
                                    field_name: String::new(),
                                    saved_code_body: std::mem::take(&mut code_body),
                                });
                            }
                            "columns" => {
                                let count: usize = field_name.parse().unwrap_or_else(|_| {
                                    panic!("::: columns requires a number, got '{field_name}'")
                                });
                                block_stack.push(BlockFrame {
                                    directive: BlockDirective::Columns {
                                        count,
                                        current_col: 0,
                                        column_bodies: vec![Vec::new(); count],
                                    },
                                    field_name: String::new(),
                                    saved_code_body: std::mem::take(&mut code_body),
                                });
                            }
                            _ => {
                                panic!(
                                    "Unknown block directive ':::{directive_name}'. \
                                     Valid: foreach, if, style, frame, horizontal, columns"
                                );
                            }
                        }
                    }
                    // Don't accumulate ::: as text — skip to next event
                    event_idx += 1;
                    continue;
                }

                // Inside foreach: parse {field} references
                let in_foreach = matches!(
                    block_stack.last().map(|f| &f.directive),
                    Some(BlockDirective::Foreach { .. })
                );
                if in_foreach && !in_code_block && heading_level.is_none() && in_link.is_none() {
                    if text_str.contains('{') {
                        flush_pending(
                            &mut pending_text,
                            &mut fragments,
                            frontmatter,
                            bold,
                            italic,
                            strikethrough,
                        );
                        if let Some(BlockFrame {
                            directive: BlockDirective::Foreach { row_fields },
                            ..
                        }) = block_stack.last_mut()
                        {
                            parse_foreach_text(
                                text_str,
                                &mut fragments,
                                row_fields,
                                bold,
                                italic,
                                strikethrough,
                            );
                        }
                    } else {
                        pending_text.push_str(text_str);
                    }
                } else {
                    pending_text.push_str(text_str);
                }
            }
            Event::Code(code_text) => {
                flush_pending(
                    &mut pending_text,
                    &mut fragments,
                    frontmatter,
                    bold,
                    italic,
                    strikethrough,
                );
                fragments.push(Fragment::InlineCode(code_text.to_string()));
            }
            Event::SoftBreak => {
                pending_text.push(' ');
            }
            Event::HardBreak => {
                flush_pending(
                    &mut pending_text,
                    &mut fragments,
                    frontmatter,
                    bold,
                    italic,
                    strikethrough,
                );
                let _ = emit_paragraph(
                    &mut fragments,
                    &mut code_body,
                    blockquote_depth,
                    frontmatter,
                    sp_paragraph,
                );
            }
            Event::Rule => {
                code_body.push(quote! { separator(ui); });
            }
            Event::Html(_) | Event::FootnoteReference(_) | Event::TaskListMarker(_) => {}
        }
        event_idx += 1;
    }

    // Generate style lookup table if dynamic styling is used
    let style_table = if needs_style_table {
        let arms: Vec<proc_macro2::TokenStream> = frontmatter
            .styles
            .iter()
            .filter_map(|(name, style)| {
                style.color.as_ref().map(|hex| {
                    let [r, g, b] = parse_hex_color(hex).expect("Invalid color in frontmatter");
                    quote! { #name => Some(egui::Color32::from_rgb(#r, #g, #b)), }
                })
            })
            .collect();
        Some(quote! {
            fn __resolve_style_color(name: &str) -> Option<egui::Color32> {
                match name {
                    #(#arms)*
                    _ => None,
                }
            }
        })
    } else {
        None
    };

    // Prepend item_spacing override if configured
    if let Some(item_sp) = sp_item {
        code_body.insert(0, quote! { ui.spacing_mut().item_spacing.y = #item_sp; });
    }

    ParsedMarkdown {
        code_body,
        widget_fields,
        references_state,
        display_refs,
        style_table,
    }
}
