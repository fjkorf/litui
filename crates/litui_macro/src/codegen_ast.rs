//! AST-to-TokenStream code generation.
//!
//! Converts a [`litui_parser::ast::Document`] into a [`ParsedMarkdown`] that
//! the existing [`crate::codegen`] module consumes. This replaces the inline
//! token generation that was previously interleaved with parsing in `parse.rs`.

use litui_parser::ast::*;
use litui_parser::frontmatter::{
    Frontmatter, StyleDef, capitalize_first, parse_hex_color, resolve_classes,
};
use quote::quote;

/// Shared context threaded through all codegen functions.
struct CodegenContext<'a> {
    frontmatter: &'a Frontmatter,
    source_span: proc_macro2::Span,
    /// True when generating code inside a `foreach` loop body.
    /// Widgets reference `__row.field` instead of `state.field`.
    in_foreach: bool,
    /// True when inside a `foreach ... children` tree body.
    /// Collapsing directives use dynamic ID salts incorporating depth + pointer.
    in_tree_foreach: bool,
}

impl<'a> CodegenContext<'a> {
    fn new(frontmatter: &'a Frontmatter, source_span: proc_macro2::Span) -> Self {
        Self {
            frontmatter,
            source_span,
            in_foreach: false,
            in_tree_foreach: false,
        }
    }

    /// Create a child context for foreach body generation.
    fn for_foreach(&self) -> Self {
        Self {
            frontmatter: self.frontmatter,
            source_span: self.source_span,
            in_foreach: true,
            in_tree_foreach: self.in_tree_foreach,
        }
    }

    /// Create a child context for tree foreach body generation.
    fn for_tree_foreach(&self) -> Self {
        Self {
            frontmatter: self.frontmatter,
            source_span: self.source_span,
            in_foreach: true,
            in_tree_foreach: true,
        }
    }

    /// Returns a TokenStream for `state` or `__row` depending on context.
    fn state_ref(&self) -> proc_macro2::TokenStream {
        if self.in_foreach {
            quote! { __row }
        } else {
            quote! { state }
        }
    }
}

// ── Bridge: output type matching crate::parse::ParsedMarkdown ────
// During the coexistence phase, codegen_ast produces this struct which
// matches the shape expected by crate::codegen. Once the old parser is
// removed, this becomes the sole definition.

pub(crate) struct ParsedMarkdownFromAst {
    pub(crate) code_body: Vec<proc_macro2::TokenStream>,
    pub(crate) widget_fields: Vec<crate::parse::WidgetField>,
    pub(crate) references_state: bool,
    pub(crate) display_refs: Vec<String>,
    pub(crate) style_table: Option<proc_macro2::TokenStream>,
    pub(crate) used_widget_configs: std::collections::HashSet<String>,
}

fn convert_row_field(rf: &litui_parser::ast::RowField) -> crate::parse::RowField {
    match rf {
        litui_parser::ast::RowField::Display(n) => crate::parse::RowField::Display(n.clone()),
        litui_parser::ast::RowField::Widget { name, ty, .. } => crate::parse::RowField::Widget {
            name: name.clone(),
            ty: convert_widget_type(*ty),
        },
        litui_parser::ast::RowField::Foreach {
            name,
            row_fields,
            is_tree,
        } => crate::parse::RowField::Foreach {
            name: name.clone(),
            row_fields: row_fields.iter().map(convert_row_field).collect(),
            is_tree: *is_tree,
        },
    }
}

/// Convert `litui_parser` `WidgetField` → `crate::parse::WidgetField`
fn convert_widget_field(f: &litui_parser::ast::WidgetField) -> crate::parse::WidgetField {
    match f {
        litui_parser::ast::WidgetField::Stateful { name, ty } => {
            crate::parse::WidgetField::Stateful {
                name: name.clone(),
                ty: convert_widget_type(*ty),
            }
        }
        litui_parser::ast::WidgetField::Foreach {
            name,
            row_fields,
            is_tree,
        } => crate::parse::WidgetField::Foreach {
            name: name.clone(),
            row_fields: row_fields.iter().map(convert_row_field).collect(),
            is_tree: *is_tree,
        },
    }
}

fn convert_widget_type(ty: litui_parser::ast::WidgetType) -> crate::parse::WidgetType {
    match ty {
        litui_parser::ast::WidgetType::F64 => crate::parse::WidgetType::F64,
        litui_parser::ast::WidgetType::Bool => crate::parse::WidgetType::Bool,
        litui_parser::ast::WidgetType::U32 => crate::parse::WidgetType::U32,
        litui_parser::ast::WidgetType::Usize => crate::parse::WidgetType::Usize,
        litui_parser::ast::WidgetType::String => crate::parse::WidgetType::String,
        litui_parser::ast::WidgetType::ByteArray4 => crate::parse::WidgetType::ByteArray4,
        litui_parser::ast::WidgetType::VecString => crate::parse::WidgetType::VecString,
        litui_parser::ast::WidgetType::Date => crate::parse::WidgetType::Date,
    }
}

// ── Local style_def_to_label_tokens (works with litui_parser types) ──

/// Resolve a color string (hex or semantic keyword) into generated token code.
/// Returns tokens that evaluate to `Option<egui::Color32>` at runtime.
fn color_value_tokens(
    color_str: &str,
    context: &str,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    fn color_error(msg: impl std::fmt::Display) -> proc_macro2::TokenStream {
        syn::Error::new(proc_macro2::Span::call_site(), msg.to_string()).to_compile_error()
    }

    let cv = litui_parser::frontmatter::parse_color_value(color_str)
        .map_err(|e| color_error(format!("{context}: {e}")))?;

    match cv {
        litui_parser::frontmatter::ColorValue::Hex([r, g, b]) => {
            Ok(quote! { Some(egui::Color32::from_rgb(#r, #g, #b)) })
        }
        litui_parser::frontmatter::ColorValue::Semantic(sem) => Ok(semantic_color_tokens(sem)),
    }
}

/// Generate tokens for a semantic color that resolves at runtime via `ui.visuals()`.
fn semantic_color_tokens(
    sem: litui_parser::frontmatter::SemanticColor,
) -> proc_macro2::TokenStream {
    use litui_parser::frontmatter::SemanticColor;
    match sem {
        SemanticColor::Text => {
            quote! { Some(ui.visuals().widgets.noninteractive.fg_stroke.color) }
        }
        SemanticColor::Strong => quote! { Some(ui.visuals().strong_text_color()) },
        SemanticColor::Weak => quote! { Some(ui.visuals().weak_text_color()) },
        SemanticColor::Hyperlink => quote! { Some(ui.visuals().hyperlink_color) },
        SemanticColor::Warn => quote! { Some(ui.visuals().warn_fg_color) },
        SemanticColor::Error => quote! { Some(ui.visuals().error_fg_color) },
        SemanticColor::CodeBg => quote! { Some(ui.visuals().code_bg_color) },
        SemanticColor::FaintBg => quote! { Some(ui.visuals().faint_bg_color) },
        SemanticColor::ExtremeBg => quote! { Some(ui.visuals().extreme_bg_color) },
        SemanticColor::PanelFill => quote! { Some(ui.visuals().panel_fill) },
        SemanticColor::WindowFill => quote! { Some(ui.visuals().window_fill) },
        SemanticColor::Selection => quote! { Some(ui.visuals().selection.bg_fill) },
    }
}

fn style_def_to_label_tokens(
    text: &str,
    style: &StyleDef,
    base_bold: bool,
    base_italic: bool,
    base_strikethrough: bool,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    let bold = style.bold.unwrap_or(base_bold);
    let italic = style.italic.unwrap_or(base_italic);
    let strikethrough = style.strikethrough.unwrap_or(base_strikethrough);
    let underline = style.underline.unwrap_or(false);
    let monospace = style.monospace.unwrap_or(false);
    let weak = style.weak.unwrap_or(false);

    let color_tokens = if let Some(color_str) = &style.color {
        color_value_tokens(color_str, "Invalid color in frontmatter")?
    } else {
        quote! { None::<egui::Color32> }
    };
    let bg_tokens = if let Some(bg_str) = &style.background {
        color_value_tokens(bg_str, "Invalid background color in frontmatter")?
    } else {
        quote! { None::<egui::Color32> }
    };
    let size_tokens = if let Some(s) = style.size {
        quote! { Some(#s) }
    } else {
        quote! { None }
    };

    // Generate RichText construction directly (supports both hex and semantic colors)
    Ok(quote! {
        {
            let mut __rt = egui::RichText::new(#text);
            if #bold { __rt = __rt.strong(); }
            if #italic { __rt = __rt.italics(); }
            if #strikethrough { __rt = __rt.strikethrough(); }
            if #underline { __rt = __rt.underline(); }
            if #monospace { __rt = __rt.monospace(); }
            if #weak { __rt = __rt.weak(); }
            if let Some(__c) = #color_tokens { __rt = __rt.color(__c); }
            if let Some(__bg) = #bg_tokens { __rt = __rt.background_color(__bg); }
            if let Some(__sz) = #size_tokens { __rt = __rt.size(__sz); }
            ui.label(__rt);
        }
    })
}

fn md_error(span: proc_macro2::Span, msg: impl std::fmt::Display) -> proc_macro2::TokenStream {
    syn::Error::new(span, msg.to_string()).to_compile_error()
}

fn get_widget_def(attrs: &str, frontmatter: &Frontmatter) -> litui_parser::frontmatter::WidgetDef {
    if attrs.is_empty() {
        litui_parser::frontmatter::WidgetDef::default()
    } else {
        frontmatter.widgets.get(attrs).cloned().unwrap_or_default()
    }
}

// ── Inline → tokens ──────────────────────────────────────────

fn inline_to_tokens(inline: &Inline) -> proc_macro2::TokenStream {
    match inline {
        Inline::Text {
            text,
            bold,
            italic,
            strikethrough,
        } => {
            quote! { styled_label(ui, #text, #bold, #italic, #strikethrough); }
        }
        Inline::InlineCode(text) => {
            quote! { inline_code(ui, #text); }
        }
        Inline::Link {
            text,
            url,
            bold,
            italic,
            strikethrough,
        } => {
            quote! { styled_hyperlink(ui, #text, #url, #bold, #italic, #strikethrough); }
        }
        Inline::ForeachField(name) => {
            let field = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote! { ui.label(format!("{}", __row.#field)); }
        }
        // These are handled specially in context (styled spans, widgets, images)
        Inline::StyledSpan { .. }
        | Inline::ClassSpan { .. }
        | Inline::Widget(_)
        | Inline::Image { .. } => {
            quote! {}
        }
    }
}

/// Convert a single inline to tokens, resolving styled spans and widgets.
fn inline_to_tokens_full(
    inline: &Inline,
    ctx: &CodegenContext<'_>,
    in_table: bool,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    match inline {
        Inline::StyledSpan {
            class,
            text,
            bold,
            italic,
            strikethrough,
        } => {
            let style = ctx.frontmatter.styles.get(class.as_str()).ok_or_else(|| {
                md_error(
                    ctx.source_span,
                    format!("Undefined style class '::{class}' in inline span"),
                )
            })?;
            style_def_to_label_tokens(text, style, *bold, *italic, *strikethrough)
        }
        Inline::ClassSpan {
            classes,
            text,
            bold,
            italic,
            strikethrough,
        } => {
            let style = resolve_classes(classes, ctx.frontmatter)
                .map_err(|e| md_error(ctx.source_span, e))?
                .unwrap_or_default();
            style_def_to_label_tokens(text, &style, *bold, *italic, *strikethrough)
        }
        Inline::Widget(w) => widget_to_tokens(w, ctx, in_table),
        Inline::Image { alt, url } => {
            let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
            let abs_url = if url.starts_with("http://")
                || url.starts_with("https://")
                || url.starts_with("file://")
            {
                url.clone()
            } else {
                format!("file://{manifest_dir}/{url}")
            };
            if alt.is_empty() {
                Ok(quote! { ui.add(egui::Image::new(#abs_url)); })
            } else {
                Ok(quote! { ui.add(egui::Image::new(#abs_url).alt_text(#alt)); })
            }
        }
        other => Ok(inline_to_tokens(other)),
    }
}

/// Convert a list of inlines to token calls, applying an optional style override.
fn fragments_to_tokens(
    fragments: &[Inline],
    style: Option<&StyleDef>,
    ctx: &CodegenContext<'_>,
) -> Result<Vec<proc_macro2::TokenStream>, proc_macro2::TokenStream> {
    fragments
        .iter()
        .map(|f| {
            if let Some(style) = style {
                // Apply style to text fragments
                if let Inline::Text {
                    text,
                    bold,
                    italic,
                    strikethrough,
                } = f
                {
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
                    return style_def_to_label_tokens(
                        text,
                        &merged,
                        *bold,
                        *italic,
                        *strikethrough,
                    );
                }
            }
            inline_to_tokens_full(f, ctx, false)
        })
        .collect()
}

// ── Widget → tokens ──────────────────────────────────────────

fn widget_to_tokens(
    w: &WidgetDirective,
    ctx: &CodegenContext<'_>,
    _in_table: bool,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    let url = &w.field;
    let content = url.clone();
    let display_content = url.replace('_', " ");
    let widget_attrs = &w.config_key;
    let state_ref = ctx.state_ref();

    // Resolve class styles
    let class_style =
        resolve_classes(&w.classes, ctx.frontmatter).map_err(|e| md_error(ctx.source_span, e))?;
    let style = class_style;

    let widget_code = match w.widget_type {
        WidgetKind::Button => {
            let button_expr = if let Some(s) = &style {
                let bold_val = s.bold.unwrap_or(false);
                let italic_val = s.italic.unwrap_or(false);
                let strike_val = s.strikethrough.unwrap_or(false);
                let color_tokens = match &s.color {
                    Some(color_str) => {
                        let cv = color_value_tokens(color_str, "Invalid button color")?;
                        quote! { .color((#cv).unwrap()) }
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

            if !widget_attrs.is_empty() {
                let wdef = get_widget_def(widget_attrs, ctx.frontmatter);
                let count_name = format!("{widget_attrs}_count");
                let count_field = syn::Ident::new(&count_name, proc_macro2::Span::call_site());

                let hover_code = if wdef.track_hover.unwrap_or(false) {
                    let hover_name = format!("{widget_attrs}_hovered");
                    let hover_field = syn::Ident::new(&hover_name, proc_macro2::Span::call_site());
                    quote! { #state_ref.#hover_field = __btn_resp.hovered(); }
                } else {
                    quote! {}
                };

                let secondary_code = if wdef.track_secondary.unwrap_or(false) {
                    let sec_name = format!("{widget_attrs}_secondary_count");
                    let sec_field = syn::Ident::new(&sec_name, proc_macro2::Span::call_site());
                    quote! {
                        if __btn_resp.secondary_clicked() {
                            #state_ref.#sec_field += 1;
                        }
                    }
                } else {
                    quote! {}
                };

                quote! {
                    {
                        let __btn_resp = #button_expr;
                        if __btn_resp.clicked() {
                            #state_ref.#count_field += 1;
                        }
                        #hover_code
                        #secondary_code
                    }
                }
            } else {
                quote! { let _ = #button_expr; }
            }
        }
        WidgetKind::Progress => {
            if let Ok(val) = content.parse::<f32>() {
                quote! { ui.add(egui::ProgressBar::new(#val).show_percentage()); }
            } else {
                let field = syn::Ident::new(&content, proc_macro2::Span::call_site());
                let wdef = get_widget_def(widget_attrs, ctx.frontmatter);
                let fill_tokens = if let Some(ref hex) = wdef.fill {
                    let [r, g, b] = parse_hex_color(hex).map_err(|e| {
                        md_error(ctx.source_span, format!("Invalid fill color: {e}"))
                    })?;
                    quote! { .fill(egui::Color32::from_rgb(#r, #g, #b)) }
                } else {
                    quote! {}
                };
                quote! {
                    ui.add(egui::ProgressBar::new(#state_ref.#field as f32).show_percentage() #fill_tokens);
                }
            }
        }
        WidgetKind::Spinner => quote! { ui.spinner(); },
        WidgetKind::Slider => {
            let field = syn::Ident::new(&content, proc_macro2::Span::call_site());
            let wdef = get_widget_def(widget_attrs, ctx.frontmatter);
            let min_val = wdef.min.unwrap_or(0.0);
            let max_val = wdef.max.unwrap_or(1.0);
            let label = wdef.label.unwrap_or_default();
            let suffix = wdef.suffix.unwrap_or_default();
            let prefix = wdef.prefix.unwrap_or_default();
            let integer_call = if wdef.integer == Some(true) {
                quote! { .integer() }
            } else {
                quote! {}
            };
            let step_call = if let Some(step) = wdef.step {
                quote! { .step_by(#step) }
            } else {
                quote! {}
            };
            let decimals_call = if let Some(dec) = wdef.decimals {
                let d = dec;
                quote! { .fixed_decimals(#d) }
            } else {
                quote! {}
            };
            quote! {
                ui.add(
                    egui::Slider::new(&mut #state_ref.#field, #min_val..=#max_val)
                        .text(#label).suffix(#suffix).prefix(#prefix)
                        #integer_call #step_call #decimals_call
                );
            }
        }
        WidgetKind::DoubleSlider => {
            let low_name = format!("{content}_low");
            let high_name = format!("{content}_high");
            let low_field = syn::Ident::new(&low_name, proc_macro2::Span::call_site());
            let high_field = syn::Ident::new(&high_name, proc_macro2::Span::call_site());
            let wdef = get_widget_def(widget_attrs, ctx.frontmatter);
            let min_val = wdef.min.unwrap_or(0.0);
            let max_val = wdef.max.unwrap_or(1.0);
            quote! {
                ui.add(egui_double_slider::DoubleSlider::new(
                    &mut #state_ref.#low_field, &mut #state_ref.#high_field, #min_val..=#max_val,
                ));
            }
        }
        WidgetKind::Checkbox => {
            let field = syn::Ident::new(&content, proc_macro2::Span::call_site());
            let label = get_widget_def(widget_attrs, ctx.frontmatter)
                .label
                .unwrap_or(content.clone());
            quote! { ui.checkbox(&mut #state_ref.#field, #label); }
        }
        WidgetKind::Textedit => {
            let field = syn::Ident::new(&content, proc_macro2::Span::call_site());
            let hint = get_widget_def(widget_attrs, ctx.frontmatter)
                .hint
                .unwrap_or_default();
            if hint.is_empty() {
                quote! { ui.text_edit_singleline(&mut #state_ref.#field); }
            } else {
                quote! {
                    ui.add(egui::TextEdit::singleline(&mut #state_ref.#field).hint_text(#hint));
                }
            }
        }
        WidgetKind::Textarea => {
            let field = syn::Ident::new(&content, proc_macro2::Span::call_site());
            let wdef = get_widget_def(widget_attrs, ctx.frontmatter);
            let hint = wdef.hint.unwrap_or_default();
            let rows = wdef.rows.unwrap_or(4);
            quote! {
                ui.add(egui::TextEdit::multiline(&mut #state_ref.#field)
                    .hint_text(#hint).desired_rows(#rows));
            }
        }
        WidgetKind::Password => {
            let field = syn::Ident::new(&content, proc_macro2::Span::call_site());
            let hint = get_widget_def(widget_attrs, ctx.frontmatter)
                .hint
                .unwrap_or_default();
            quote! {
                ui.add(egui::TextEdit::singleline(&mut #state_ref.#field)
                    .password(true).hint_text(#hint));
            }
        }
        WidgetKind::Dragvalue => {
            let field = syn::Ident::new(&content, proc_macro2::Span::call_site());
            let wdef = get_widget_def(widget_attrs, ctx.frontmatter);
            let speed = wdef.speed.unwrap_or(0.1);
            let range_call = match (wdef.min, wdef.max) {
                (Some(lo), Some(hi)) => quote! { .range(#lo..=#hi) },
                (Some(lo), None) => quote! { .range(#lo..=f64::MAX) },
                (None, Some(hi)) => quote! { .range(f64::MIN..=#hi) },
                (None, None) => quote! {},
            };
            let suffix_call = match &wdef.suffix {
                Some(s) if !s.is_empty() => quote! { .suffix(#s) },
                _ => quote! {},
            };
            let prefix_call = match &wdef.prefix {
                Some(s) if !s.is_empty() => quote! { .prefix(#s) },
                _ => quote! {},
            };
            let decimals_call = if let Some(dec) = wdef.decimals {
                let d = dec;
                quote! { .fixed_decimals(#d) }
            } else {
                quote! {}
            };
            quote! {
                ui.add(
                    egui::DragValue::new(&mut #state_ref.#field)
                        .speed(#speed) #range_call #suffix_call #prefix_call #decimals_call
                );
            }
        }
        WidgetKind::Display => {
            let field = syn::Ident::new(&content, proc_macro2::Span::call_site());
            let wdef = get_widget_def(widget_attrs, ctx.frontmatter);
            let fmt = wdef.format.as_deref().unwrap_or("{}");
            quote! { ui.label(format!(#fmt, #state_ref.#field)); }
        }
        WidgetKind::Radio => {
            let field = syn::Ident::new(&content, proc_macro2::Span::call_site());
            let wdef = get_widget_def(widget_attrs, ctx.frontmatter);
            let options = wdef
                .options
                .unwrap_or_else(|| vec!["Option A".into(), "Option B".into()]);
            let radio_calls: Vec<proc_macro2::TokenStream> = options
                .iter()
                .enumerate()
                .map(|(i, opt)| {
                    quote! { ui.radio_value(&mut #state_ref.#field, #i, #opt); }
                })
                .collect();
            quote! { #(#radio_calls)* }
        }
        WidgetKind::Combobox => {
            let field = syn::Ident::new(&content, proc_macro2::Span::call_site());
            let wdef = get_widget_def(widget_attrs, ctx.frontmatter);
            let options = wdef
                .options
                .unwrap_or_else(|| vec!["Option A".into(), "Option B".into()]);
            let label = wdef.label.unwrap_or_else(|| display_content.clone());
            let num_options = options.len();
            quote! {
                {
                    const OPTIONS: &[&str] = &[#(#options),*];
                    egui::ComboBox::from_label(#label)
                        .selected_text(OPTIONS[#state_ref.#field])
                        .show_index(ui, &mut #state_ref.#field, #num_options, |i| OPTIONS[i]);
                }
            }
        }
        WidgetKind::Color => {
            let field = syn::Ident::new(&content, proc_macro2::Span::call_site());
            quote! {
                {
                    let mut __c = egui::Color32::from_rgba_unmultiplied(
                        #state_ref.#field[0], #state_ref.#field[1],
                        #state_ref.#field[2], #state_ref.#field[3],
                    );
                    egui::color_picker::color_edit_button_srgba(
                        ui, &mut __c, egui::color_picker::Alpha::Opaque,
                    );
                    #state_ref.#field = [__c.r(), __c.g(), __c.b(), __c.a()];
                }
            }
        }
        WidgetKind::Toggle => {
            let field = syn::Ident::new(&content, proc_macro2::Span::call_site());
            let label = get_widget_def(widget_attrs, ctx.frontmatter)
                .label
                .unwrap_or_default();
            if label.is_empty() {
                quote! { toggle_switch(ui, &mut #state_ref.#field); }
            } else {
                quote! {
                    ui.horizontal(|ui| {
                        toggle_switch(ui, &mut #state_ref.#field);
                        ui.label(#label);
                    });
                }
            }
        }
        WidgetKind::Selectable => {
            let field = syn::Ident::new(&content, proc_macro2::Span::call_site());
            let wdef = get_widget_def(widget_attrs, ctx.frontmatter);
            let options = wdef
                .options
                .unwrap_or_else(|| vec!["Option A".into(), "Option B".into()]);
            let selectable_calls: Vec<proc_macro2::TokenStream> = options
                .iter()
                .enumerate()
                .map(|(i, opt)| {
                    quote! { ui.selectable_value(&mut #state_ref.#field, #i, #opt); }
                })
                .collect();
            quote! { ui.horizontal(|ui| { #(#selectable_calls)* }); }
        }
        WidgetKind::Select => {
            let index_field = syn::Ident::new(&content, proc_macro2::Span::call_site());
            let list_field = syn::Ident::new(widget_attrs, proc_macro2::Span::call_site());
            let wdef = get_widget_def(widget_attrs, ctx.frontmatter);
            let max_h = wdef.max_height.unwrap_or(200.0) as f32;
            let select_id = format!("md_select_{content}");
            quote! {
                egui::ScrollArea::vertical()
                    .id_salt(#select_id)
                    .max_height(#max_h)
                    .show(ui, |ui| {
                        for (__i, __label) in #state_ref.#list_field.iter().enumerate() {
                            if ui.selectable_label(#state_ref.#index_field == __i, __label).clicked() {
                                #state_ref.#index_field = __i;
                            }
                        }
                    });
            }
        }
        WidgetKind::Log => {
            let field = syn::Ident::new(&content, proc_macro2::Span::call_site());
            let wdef = get_widget_def(widget_attrs, ctx.frontmatter);
            let max_h = wdef.max_height.unwrap_or(200.0) as f32;
            let log_id = format!("md_log_{content}");
            quote! {
                egui::ScrollArea::vertical()
                    .id_salt(#log_id)
                    .max_height(#max_h)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for __msg in &#state_ref.#field {
                            ui.label(__msg.as_str());
                        }
                    });
            }
        }
        WidgetKind::Datepicker => {
            let field = syn::Ident::new(&content, proc_macro2::Span::call_site());
            quote! {
                ui.add(egui_extras::DatePickerButton::new(&mut #state_ref.#field));
            }
        }
    };

    // Wrap with push_id if selector has an ID
    let widget_code = if let Some(ref id_str) = w.id {
        quote! { ui.push_id(#id_str, |ui| { #widget_code }); }
    } else {
        widget_code
    };

    Ok(widget_code)
}

// ── Block → tokens ───────────────────────────────────────────

/// Convert a `Document` AST into a `ParsedMarkdownFromAst` with generated `TokenStream`.
///
/// This produces the same structure as `crate::parse::ParsedMarkdown`, using
/// bridge types during the coexistence phase. The output can be converted to
/// `crate::parse::ParsedMarkdown` via `into_parsed_markdown()`.
pub(crate) fn document_to_parsed(
    doc: &Document,
    frontmatter: &Frontmatter,
    source_span: proc_macro2::Span,
) -> Result<ParsedMarkdownFromAst, proc_macro2::TokenStream> {
    let mut code_body: Vec<proc_macro2::TokenStream> = Vec::new();
    let ctx = CodegenContext::new(frontmatter, source_span);

    for block in &doc.blocks {
        block_to_tokens(block, &mut code_body, &ctx)?;
    }

    // Generate style lookup table if dynamic styling is used
    let style_table = if doc.needs_style_table {
        let mut arms: Vec<proc_macro2::TokenStream> = Vec::new();
        for (name, style) in &frontmatter.styles {
            if let Some(color_str) = &style.color {
                let color_expr = color_value_tokens(color_str, "Invalid color in frontmatter")?;
                arms.push(quote! { #name => #color_expr, });
            }
        }
        Some(quote! {
            fn __resolve_style_color(ui: &egui::Ui, name: &str) -> Option<egui::Color32> {
                match name {
                    #(#arms)*
                    _ => None,
                }
            }
        })
    } else {
        None
    };

    Ok(ParsedMarkdownFromAst {
        code_body,
        widget_fields: doc.widget_fields.iter().map(convert_widget_field).collect(),
        references_state: doc.references_state,
        display_refs: doc.display_refs.clone(),
        style_table,
        used_widget_configs: doc.used_widget_configs.clone(),
    })
}

impl ParsedMarkdownFromAst {
    /// Convert to the old `ParsedMarkdown` type for consumption by `codegen.rs`.
    pub(crate) fn into_parsed_markdown(self) -> crate::parse::ParsedMarkdown {
        crate::parse::ParsedMarkdown {
            code_body: self.code_body,
            widget_fields: self.widget_fields,
            references_state: self.references_state,
            display_refs: self.display_refs,
            style_table: self.style_table,
            used_widget_configs: self.used_widget_configs,
        }
    }
}

fn block_to_tokens(
    block: &Block,
    code_body: &mut Vec<proc_macro2::TokenStream>,
    ctx: &CodegenContext<'_>,
) -> Result<(), proc_macro2::TokenStream> {
    match block {
        Block::Spacing(amount) => {
            code_body.push(quote! { ui.add_space(#amount); });
        }
        Block::ItemSpacingOverride(val) => {
            code_body.push(quote! { ui.spacing_mut().item_spacing.y = #val; });
        }
        Block::HorizontalRule => {
            code_body.push(quote! { separator(ui); });
        }
        Block::CodeBlock { text } => {
            code_body.push(quote! { code(ui, #text); });
        }
        Block::Heading {
            level,
            text,
            style_suffix,
        } => {
            heading_to_tokens(level, text, style_suffix, code_body, ctx)?;
        }
        Block::Paragraph {
            fragments,
            style_suffix,
        } => {
            paragraph_to_tokens(fragments, style_suffix, 0, code_body, ctx)?;
        }
        Block::BlockQuote {
            depth,
            blocks: inner_blocks,
        } => {
            for inner in inner_blocks {
                match inner {
                    Block::Paragraph {
                        fragments,
                        style_suffix,
                    } => {
                        paragraph_to_tokens(fragments, style_suffix, *depth, code_body, ctx)?;
                    }
                    Block::List { kind, items } => {
                        list_to_tokens(kind, items, *depth, code_body, ctx)?;
                    }
                    other => {
                        block_to_tokens(other, code_body, ctx)?;
                    }
                }
            }
        }
        Block::List { kind, items } => {
            list_to_tokens(kind, items, 0, code_body, ctx)?;
        }
        Block::Table {
            headers,
            rows,
            num_columns,
            table_index,
            alignments,
        } => {
            table_to_tokens(
                headers,
                rows,
                *num_columns,
                *table_index,
                alignments,
                false,
                code_body,
                ctx,
            )?;
        }
        Block::Image { alt, url } => {
            let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
            let abs_url = if url.starts_with("http://")
                || url.starts_with("https://")
                || url.starts_with("file://")
            {
                url.clone()
            } else {
                format!("file://{manifest_dir}/{url}")
            };
            if alt.is_empty() {
                code_body.push(quote! { ui.add(egui::Image::new(#abs_url)); });
            } else {
                code_body.push(quote! { ui.add(egui::Image::new(#abs_url).alt_text(#alt)); });
            }
        }
        Block::Widget(w) => {
            let tokens = widget_to_tokens(w, ctx, false)?;
            code_body.push(tokens);
        }
        Block::Directive(directive) => {
            directive_to_tokens(directive, code_body, ctx)?;
        }
    }
    Ok(())
}

fn heading_to_tokens(
    level: &HeadingLevel,
    text: &str,
    style_suffix: &Option<StyleSuffix>,
    code_body: &mut Vec<proc_macro2::TokenStream>,
    ctx: &CodegenContext<'_>,
) -> Result<(), proc_macro2::TokenStream> {
    if let Some(StyleSuffix::Static(key)) = style_suffix {
        let style = ctx.frontmatter.styles.get(key.as_str()).ok_or_else(|| {
            md_error(
                ctx.source_span,
                format!("Undefined style key '{key}' in frontmatter"),
            )
        })?;
        let default_size = match level {
            HeadingLevel::H1 => 28.0_f32,
            HeadingLevel::H2 => 22.0_f32,
            HeadingLevel::H3 => 18.0_f32,
            HeadingLevel::H4 => 14.0_f32,
        };
        let mut merged = style.clone();
        if merged.bold.is_none() {
            merged.bold = Some(true);
        }
        if merged.size.is_none() {
            merged.size = Some(default_size);
        }
        let tokens = style_def_to_label_tokens(text, &merged, true, false, false)?;
        code_body.push(tokens);
    } else {
        code_body.push(match level {
            HeadingLevel::H1 => quote! { h1(ui, #text); },
            HeadingLevel::H2 => quote! { h2(ui, #text); },
            HeadingLevel::H3 => quote! { h3(ui, #text); },
            HeadingLevel::H4 => quote! { body(ui, #text); },
        });
    }
    Ok(())
}

fn paragraph_to_tokens(
    fragments: &[Inline],
    style_suffix: &Option<StyleSuffix>,
    blockquote_depth: usize,
    code_body: &mut Vec<proc_macro2::TokenStream>,
    ctx: &CodegenContext<'_>,
) -> Result<(), proc_macro2::TokenStream> {
    if fragments.is_empty() {
        return Ok(());
    }

    let resolved_style = match style_suffix {
        Some(StyleSuffix::Static(key)) => {
            let style = ctx.frontmatter.styles.get(key.as_str()).ok_or_else(|| {
                md_error(
                    ctx.source_span,
                    format!("Undefined style key '{key}' in frontmatter"),
                )
            })?;
            Some(style.clone())
        }
        _ => None,
    };

    let calls = fragments_to_tokens(fragments, resolved_style.as_ref(), ctx)?;

    if blockquote_depth > 0 {
        let depth = blockquote_depth;
        let bar_color_tokens = if let Some(ref style) = resolved_style {
            if let Some(ref color_str) = style.color {
                // Only hex colors can be passed as [u8; 3]; semantic colors use default visuals
                if color_str.starts_with('#') {
                    let [r, g, b] = parse_hex_color(color_str).map_err(|e| {
                        md_error(
                            ctx.source_span,
                            format!("Invalid color in frontmatter: {e}"),
                        )
                    })?;
                    quote! { Some([#r, #g, #b]) }
                } else {
                    quote! { None }
                }
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

    // Handle runtime style wrapper
    if let Some(StyleSuffix::Dynamic(field_name)) = style_suffix {
        let field_ident = syn::Ident::new(field_name, proc_macro2::Span::call_site());
        let style_ref = ctx.state_ref();
        let emitted: Vec<_> = code_body
            .drain(code_body.len().saturating_sub(1)..)
            .collect();
        code_body.push(quote! {
            {
                let __style_color = __resolve_style_color(ui, &#style_ref.#field_ident);
                if let Some(__c) = __style_color {
                    ui.visuals_mut().override_text_color = Some(__c);
                }
                #(#emitted)*
            }
        });
    }

    Ok(())
}

fn list_to_tokens(
    _kind: &ListKind,
    items: &[ListItem],
    blockquote_depth: usize,
    code_body: &mut Vec<proc_macro2::TokenStream>,
    ctx: &CodegenContext<'_>,
) -> Result<(), proc_macro2::TokenStream> {
    for item in items {
        list_item_to_tokens(item, blockquote_depth, code_body, ctx)?;
    }
    Ok(())
}

fn list_item_to_tokens(
    item: &ListItem,
    blockquote_depth: usize,
    code_body: &mut Vec<proc_macro2::TokenStream>,
    ctx: &CodegenContext<'_>,
) -> Result<(), proc_macro2::TokenStream> {
    if item.fragments.is_empty() {
        return Ok(());
    }

    let depth = item.depth;

    let resolved_style = match &item.style_suffix {
        Some(StyleSuffix::Static(key)) => {
            let style = ctx.frontmatter.styles.get(key.as_str()).ok_or_else(|| {
                md_error(
                    ctx.source_span,
                    format!("Undefined style key '{key}' in frontmatter"),
                )
            })?;
            Some(style.clone())
        }
        _ => None,
    };

    let calls = fragments_to_tokens(&item.fragments, resolved_style.as_ref(), ctx)?;

    let prefix_color_tokens = if let Some(ref style) = resolved_style {
        if let Some(ref color_str) = style.color {
            if color_str.starts_with('#') {
                let [r, g, b] = parse_hex_color(color_str).map_err(|e| {
                    md_error(
                        ctx.source_span,
                        format!("Invalid color in frontmatter: {e}"),
                    )
                })?;
                quote! { Some([#r, #g, #b]) }
            } else {
                quote! { None }
            }
        } else {
            quote! { None }
        }
    } else {
        quote! { None }
    };

    let prefix = match &item.kind {
        ListKind::Ordered(n) => {
            let num_str = n.to_string();
            quote! { emit_numbered_prefix_colored(ui, #depth, #num_str, #prefix_color_tokens); }
        }
        ListKind::Unordered => {
            quote! { emit_bullet_prefix_colored(ui, #depth, #prefix_color_tokens); }
        }
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

    // Handle runtime style wrapper for list items
    if let Some(StyleSuffix::Dynamic(field_name)) = &item.style_suffix {
        let field_ident = syn::Ident::new(field_name, proc_macro2::Span::call_site());
        let style_ref = ctx.state_ref();
        let emitted = code_body.pop();
        if let Some(emitted) = emitted {
            code_body.push(quote! {
                {
                    let __style_color = __resolve_style_color(ui, &#style_ref.#field_ident);
                    if let Some(__c) = __style_color {
                        ui.visuals_mut().override_text_color = Some(__c);
                    }
                    #emitted
                }
            });
        }
    }

    // Render children (nested lists)
    for child in &item.children {
        block_to_tokens(child, code_body, ctx)?;
    }

    Ok(())
}

fn table_to_tokens(
    headers: &[TableCell],
    rows: &[Vec<TableCell>],
    num_columns: usize,
    table_id: usize,
    _alignments: &[ColumnAlignment],
    in_foreach: bool,
    code_body: &mut Vec<proc_macro2::TokenStream>,
    ctx: &CodegenContext<'_>,
) -> Result<(), proc_macro2::TokenStream> {
    let id_str = format!("md_table_{table_id}");
    let ncols = num_columns;

    let header_tokens = headers
        .iter()
        .map(
            |cell| -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
                let calls: Vec<proc_macro2::TokenStream> = cell
                    .fragments
                    .iter()
                    .map(|f| match f {
                        Inline::Text {
                            text,
                            italic,
                            strikethrough,
                            ..
                        } => {
                            let i = *italic;
                            let s = *strikethrough;
                            Ok(quote! { styled_label(ui, #text, true, #i, #s); })
                        }
                        other => inline_to_tokens_full(other, ctx, true),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if calls.len() == 1 {
                    Ok(calls.into_iter().next().unwrap_or_default())
                } else {
                    Ok(quote! { ui.horizontal_wrapped(|ui| { #(#calls)* }); })
                }
            },
        )
        .collect::<Result<Vec<proc_macro2::TokenStream>, _>>()?;

    let row_tokens = rows
        .iter()
        .map(
            |row| -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
                let cell_tokens = row
                    .iter()
                    .map(
                        |cell| -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
                            let calls: Vec<proc_macro2::TokenStream> = cell
                                .fragments
                                .iter()
                                .map(|f| inline_to_tokens_full(f, ctx, true))
                                .collect::<Result<Vec<_>, _>>()?;
                            if calls.len() == 1 {
                                Ok(calls.into_iter().next().unwrap_or_default())
                            } else {
                                Ok(quote! { ui.horizontal_wrapped(|ui| { #(#calls)* }); })
                            }
                        },
                    )
                    .collect::<Result<Vec<proc_macro2::TokenStream>, _>>()?;
                Ok(quote! {
                    #(#cell_tokens)*
                    ui.end_row();
                })
            },
        )
        .collect::<Result<Vec<proc_macro2::TokenStream>, _>>()?;

    if in_foreach {
        let base_id = id_str;
        code_body.push(quote! {
            ui.push_id(format!("{}_{}", #base_id, __row_idx), |ui| {
                egui::Grid::new(format!("{}_{}", #base_id, __row_idx))
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
    Ok(())
}

fn directive_to_tokens(
    directive: &Directive,
    code_body: &mut Vec<proc_macro2::TokenStream>,
    ctx: &CodegenContext<'_>,
) -> Result<(), proc_macro2::TokenStream> {
    match directive {
        Directive::Foreach {
            field,
            body,
            row_fields: _,
            is_tree,
        } => {
            let foreach_ctx = if *is_tree {
                ctx.for_tree_foreach()
            } else {
                ctx.for_foreach()
            };
            let mut body_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
            for block in body {
                // Tables inside foreach need the in_foreach flag
                match block {
                    Block::Table {
                        headers,
                        rows,
                        num_columns,
                        table_index,
                        alignments,
                    } => {
                        table_to_tokens(
                            headers,
                            rows,
                            *num_columns,
                            *table_index,
                            alignments,
                            true,
                            &mut body_tokens,
                            &foreach_ctx,
                        )?;
                    }
                    _ => {
                        block_to_tokens(block, &mut body_tokens, &foreach_ctx)?;
                    }
                }
            }
            let field_ident = syn::Ident::new(field, proc_macro2::Span::call_site());

            if *is_tree {
                // Generate a recursive local function for tree rendering.
                let struct_name = format!("{}Row", capitalize_first(field));
                let struct_ident = syn::Ident::new(&struct_name, proc_macro2::Span::call_site());
                let fn_name = syn::Ident::new(
                    &format!("__render_{field}_tree"),
                    proc_macro2::Span::call_site(),
                );
                code_body.push(quote! {
                    fn #fn_name(
                        ui: &mut egui::Ui,
                        __tree_nodes: &mut Vec<#struct_ident>,
                        __tree_depth: usize,
                    ) {
                        for (__row_idx, __row) in __tree_nodes.iter_mut().enumerate() {
                            #(#body_tokens)*
                            if !__row.children.is_empty() {
                                #fn_name(ui, &mut __row.children, __tree_depth + 1);
                            }
                        }
                    }
                    #fn_name(ui, &mut state.#field_ident, 0);
                });
            } else {
                // Collection source: state.field (top-level) or __row.field (inner foreach)
                let source = if ctx.in_foreach {
                    quote! { __row.#field_ident }
                } else {
                    quote! { state.#field_ident }
                };
                code_body.push(quote! {
                    for (__row_idx, __row) in #source.iter_mut().enumerate() {
                        #(#body_tokens)*
                    }
                });
            }
        }
        Directive::If { field, body } => {
            let mut body_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
            for block in body {
                block_to_tokens(block, &mut body_tokens, ctx)?;
            }
            let field_ident = syn::Ident::new(field, proc_macro2::Span::call_site());
            code_body.push(quote! {
                if state.#field_ident {
                    #(#body_tokens)*
                }
            });
        }
        Directive::Style { field, body } => {
            let mut body_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
            for block in body {
                block_to_tokens(block, &mut body_tokens, ctx)?;
            }
            let field_ident = syn::Ident::new(field, proc_macro2::Span::call_site());
            let style_ref = ctx.state_ref();
            code_body.push(quote! {
                {
                    let __style_color = __resolve_style_color(ui, &#style_ref.#field_ident);
                    if let Some(__c) = __style_color {
                        ui.visuals_mut().override_text_color = Some(__c);
                    }
                    #(#body_tokens)*
                }
            });
        }
        Directive::Horizontal {
            align,
            body,
            right_body,
        } => {
            let mut body_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
            for block in body {
                block_to_tokens(block, &mut body_tokens, ctx)?;
            }
            match align {
                HorizontalAlign::Left => {
                    code_body.push(quote! {
                        ui.horizontal(|ui| {
                            #(#body_tokens)*
                        });
                    });
                }
                HorizontalAlign::Center => {
                    code_body.push(quote! {
                        ui.with_layout(
                            egui::Layout::left_to_right(egui::Align::Center)
                                .with_main_align(egui::Align::Center),
                            |ui| { #(#body_tokens)* },
                        );
                    });
                }
                HorizontalAlign::Right => {
                    code_body.push(quote! {
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| { #(#body_tokens)* },
                        );
                    });
                }
                HorizontalAlign::SpaceBetween => {
                    let mut right_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
                    for block in right_body {
                        block_to_tokens(block, &mut right_tokens, ctx)?;
                    }
                    code_body.push(quote! {
                        ui.horizontal(|ui| {
                            #(#body_tokens)*
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| { #(#right_tokens)* },
                            );
                        });
                    });
                }
            }
        }
        Directive::Columns {
            count,
            weights,
            columns,
        } => {
            let col_count = *count;
            let col_tokens: Vec<proc_macro2::TokenStream> = columns
                .iter()
                .enumerate()
                .map(|(i, col_body)| {
                    let mut col_body_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
                    for block in col_body {
                        block_to_tokens(block, &mut col_body_tokens, ctx)?;
                    }
                    Ok(quote! {
                        cols[#i].vertical(|ui| {
                            #(#col_body_tokens)*
                        });
                    })
                })
                .collect::<Result<Vec<_>, proc_macro2::TokenStream>>()?;

            if weights.is_empty() || weights.iter().all(|w| *w == weights[0]) {
                // Equal-weight columns — use simple ui.columns()
                code_body.push(quote! {
                    ui.columns(#col_count, |cols| {
                        #(#col_tokens)*
                    });
                });
            } else {
                // Weighted columns — use StripBuilder with relative sizes
                let total: f32 = weights.iter().sum::<usize>() as f32;
                let fractions: Vec<f32> = weights.iter().map(|w| *w as f32 / total).collect();
                let strip_cells: Vec<proc_macro2::TokenStream> = columns
                    .iter()
                    .enumerate()
                    .map(|(i, col_body)| {
                        let mut col_body_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
                        for block in col_body {
                            block_to_tokens(block, &mut col_body_tokens, ctx)?;
                        }
                        Ok(quote! {
                            strip.cell(|ui| {
                                ui.vertical(|ui| {
                                    #(#col_body_tokens)*
                                });
                            });
                        })
                    })
                    .collect::<Result<Vec<_>, proc_macro2::TokenStream>>()?;
                let size_calls: Vec<proc_macro2::TokenStream> = fractions
                    .iter()
                    .map(|f| quote! { .size(egui_extras::Size::relative(#f)) })
                    .collect();
                code_body.push(quote! {
                    egui_extras::StripBuilder::new(ui)
                        #(#size_calls)*
                        .horizontal(|mut strip| {
                            #(#strip_cells)*
                        });
                });
            }
        }
        Directive::Center { body } => {
            let mut body_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
            for block in body {
                block_to_tokens(block, &mut body_tokens, ctx)?;
            }
            code_body.push(quote! {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    #(#body_tokens)*
                });
            });
        }
        Directive::Right { body } => {
            let mut body_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
            for block in body {
                block_to_tokens(block, &mut body_tokens, ctx)?;
            }
            code_body.push(quote! {
                ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
                    #(#body_tokens)*
                });
            });
        }
        Directive::Fill { body } => {
            let mut body_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
            for block in body {
                block_to_tokens(block, &mut body_tokens, ctx)?;
            }
            code_body.push(quote! {
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                    #(#body_tokens)*
                });
            });
        }
        Directive::Collapsing {
            title,
            title_field,
            open_field,
            default_open,
            body,
            collapsing_index,
        } => {
            let mut body_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
            for block in body {
                block_to_tokens(block, &mut body_tokens, ctx)?;
            }

            let state_ref = ctx.state_ref();
            let base_salt = format!("litui_collapsing_{collapsing_index}");

            // In tree foreach, use dynamic salt incorporating depth + pointer for uniqueness
            let salt = if ctx.in_tree_foreach {
                quote! { format!("{}_{}_{}", #base_salt, __tree_depth, __row_idx) }
            } else {
                quote! { #base_salt }
            };

            // Title expression: literal string or field reference
            let title_expr = if let Some(field) = title_field {
                let field_ident = syn::Ident::new(field, ctx.source_span);
                quote! { #state_ref.#field_ident.as_str() }
            } else {
                quote! { #title }
            };

            if let Some(open) = open_field {
                // Bidirectional state tracking via CollapsingState.
                // Uses show_toggle_button + show_body_indented (both &mut self)
                // instead of show_header (which consumes self by value).
                let open_ident = syn::Ident::new(open, ctx.source_span);
                code_body.push(quote! {
                    {
                        let __collapsing_id = ui.make_persistent_id(#salt);
                        let mut __cs = egui::collapsing_header::CollapsingState::load_with_default_open(
                            ui.ctx(), __collapsing_id, #state_ref.#open_ident,
                        );
                        if #state_ref.#open_ident != __cs.is_open() {
                            __cs.set_open(#state_ref.#open_ident);
                        }
                        let __toggle = ui.horizontal(|ui| {
                            let __btn = __cs.show_toggle_button(
                                ui, egui::collapsing_header::paint_default_icon,
                            );
                            ui.label(#title_expr);
                            __btn
                        });
                        __cs.show_body_indented(&__toggle.inner, ui, |ui| {
                            #(#body_tokens)*
                        });
                        #state_ref.#open_ident = __cs.is_open();
                    }
                });
            } else {
                // egui manages state internally
                let default_open_val = *default_open;
                code_body.push(quote! {
                    egui::CollapsingHeader::new(#title_expr)
                        .id_salt(#salt)
                        .default_open(#default_open_val)
                        .show(ui, |ui| {
                            #(#body_tokens)*
                        });
                });
            }
        }
        Directive::Frame { style_name, body } => {
            let style = style_name
                .as_ref()
                .and_then(|name| ctx.frontmatter.styles.get(name));

            let padding = style.and_then(|s| s.inner_margin).unwrap_or(8.0);
            let outer = style.and_then(|s| s.outer_margin).unwrap_or(0.0);
            let stroke_w = style.and_then(|s| s.stroke).unwrap_or(0.0);
            let stroke_c = if let Some(color_str) = style.and_then(|s| s.stroke_color.as_ref()) {
                let tokens = color_value_tokens(color_str, "Invalid stroke color")?;
                quote! { (#tokens).unwrap_or(egui::Color32::TRANSPARENT) }
            } else {
                quote! { egui::Color32::TRANSPARENT }
            };
            let radius = style.and_then(|s| s.corner_radius).unwrap_or(0.0);
            let bg = if let Some(color_str) = style.and_then(|s| s.background.as_ref()) {
                let tokens = color_value_tokens(color_str, "Invalid background color")?;
                quote! { (#tokens).unwrap_or(egui::Color32::TRANSPARENT) }
            } else {
                quote! { egui::Color32::TRANSPARENT }
            };

            let mut body_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
            for block in body {
                block_to_tokens(block, &mut body_tokens, ctx)?;
            }

            code_body.push(quote! {
                egui::Frame::default()
                    .inner_margin(#padding)
                    .outer_margin(#outer)
                    .fill(#bg)
                    .stroke(egui::Stroke::new(#stroke_w, #stroke_c))
                    .corner_radius(#radius)
                    .show(ui, |ui| {
                        #(#body_tokens)*
                    });
            });
        }
    }
    Ok(())
}
