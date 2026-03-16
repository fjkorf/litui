//! Markdown-to-AST parser.
//!
//! Converts markdown content (after frontmatter stripping) into a [`Document`]
//! AST using the pulldown-cmark 0.9 event stream. This module mirrors the logic
//! of the original `litui()` parser but produces pure data instead of
//! `TokenStream` values.
//!
//! See `knowledge/pulldown-cmark-0.9.md` for the event model.

use crate::ast::*;
use crate::error::ParseError;
use crate::frontmatter::{Frontmatter, detect_style_suffix, parse_selectors, resolve_classes};
use pulldown_cmark::{Event, Tag};
use std::collections::HashSet;

// ── Inline styled span parsing ─────────────────────────────

/// Scan text for `::class(text)` inline styled span patterns.
/// Returns true if any spans were found and processed.
fn parse_inline_styled_spans(
    text: &str,
    fragments: &mut Vec<Inline>,
    frontmatter: &Frontmatter,
    bold: bool,
    italic: bool,
    strikethrough: bool,
) -> Result<bool, ParseError> {
    let mut found = false;
    let mut remaining = text;

    while let Some(dcolon) = remaining.find("::") {
        let after_colons = &remaining[dcolon + 2..];
        let ident_end = after_colons
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .unwrap_or(after_colons.len());
        if ident_end == 0 {
            break;
        }
        let class_name = &after_colons[..ident_end];
        let after_ident = &after_colons[ident_end..];

        if !after_ident.starts_with('(') {
            break;
        }

        if let Some(close_paren) = after_ident.find(')') {
            let span_text = &after_ident[1..close_paren];

            let before = &remaining[..dcolon];
            if !before.is_empty() {
                fragments.push(Inline::Text {
                    text: before.to_owned(),
                    bold,
                    italic,
                    strikethrough,
                });
            }

            // Validate the class exists in frontmatter
            if !frontmatter.styles.contains_key(class_name) {
                return Err(ParseError::new(format!(
                    "Undefined style class '::{class_name}' in inline span"
                )));
            }

            fragments.push(Inline::StyledSpan {
                class: class_name.to_owned(),
                text: span_text.to_owned(),
                bold,
                italic,
                strikethrough,
            });

            remaining = &after_ident[close_paren + 1..];
            found = true;
        } else {
            break;
        }
    }

    if found && !remaining.is_empty() {
        fragments.push(Inline::Text {
            text: remaining.to_owned(),
            bold,
            italic,
            strikethrough,
        });
    }

    Ok(found)
}

/// Parse text containing `{field}` references into alternating Text and `ForeachField`
/// fragments. Used inside foreach blocks for field substitution.
fn parse_foreach_text(
    text: &str,
    fragments: &mut Vec<Inline>,
    row_fields: &mut Vec<RowField>,
    bold: bool,
    italic: bool,
    strikethrough: bool,
) {
    let mut remaining = text;
    while let Some(open) = remaining.find('{') {
        if let Some(close) = remaining[open..].find('}') {
            let close = open + close;
            let field_name = remaining[open + 1..close].trim();
            if !field_name.is_empty() && field_name.chars().all(|c| c.is_alphanumeric() || c == '_')
            {
                let before = &remaining[..open];
                if !before.is_empty() {
                    fragments.push(Inline::Text {
                        text: before.to_owned(),
                        bold,
                        italic,
                        strikethrough,
                    });
                }
                if !row_fields.iter().any(|rf| rf.name() == field_name) {
                    row_fields.push(RowField::Display(field_name.to_owned()));
                }
                fragments.push(Inline::ForeachField(field_name.to_owned()));
                remaining = &remaining[close + 1..];
                continue;
            }
        }
        break;
    }
    if !remaining.is_empty() {
        fragments.push(Inline::Text {
            text: remaining.to_owned(),
            bold,
            italic,
            strikethrough,
        });
    }
}

// ── Block directive stack ──────────────────────────────────

/// The type of block directive opened by `:::`.
enum BlockDirectiveKind {
    Foreach {
        row_fields: Vec<RowField>,
    },
    If,
    Style,
    Frame {
        style_name: Option<String>,
    },
    Horizontal {
        align: HorizontalAlign,
        /// For space-between: stores the left body after ::: next.
        left_body: Vec<Block>,
    },
    Columns {
        count: usize,
        weights: Vec<usize>,
        current_col: usize,
        column_bodies: Vec<Vec<Block>>,
    },
    Center,
    Right,
    Fill,
}

/// A stack frame for a `:::` block directive.
struct BlockFrame {
    directive: BlockDirectiveKind,
    field_name: String,
    saved_blocks: Vec<Block>,
}

// ── Style suffix extraction ────────────────────────────────

/// Extract style suffix from the last fragment in a list, returning the suffix
/// and trimming it from the fragment text.
fn extract_style_suffix(fragments: &mut Vec<Inline>) -> Option<StyleSuffix> {
    let suffix = {
        if let Some(Inline::Text { text, .. }) = fragments.last() {
            let (trimmed, key) = detect_style_suffix(text);
            key.map(|k| (trimmed.to_owned(), k.to_owned()))
        } else {
            None
        }
    };

    if let Some((trimmed_text, key)) = suffix {
        if let Some(Inline::Text { text, .. }) = fragments.last_mut() {
            *text = trimmed_text;
            if text.is_empty() {
                fragments.pop();
            }
        }
        if let Some(field) = key.strip_prefix('$') {
            Some(StyleSuffix::Dynamic(field.to_owned()))
        } else {
            Some(StyleSuffix::Static(key))
        }
    } else {
        None
    }
}

// ── Main parser ────────────────────────────────────────────

/// Parse markdown content into a [`Document`] AST.
///
/// This is the pure-data equivalent of the original litui parser.
/// It walks every pulldown-cmark event, tracks inline style state, list nesting,
/// blockquote depth, table structure, and widget/block directives.
///
/// # Errors
/// Returns `ParseError` for undefined styles, unknown widgets, malformed directives, etc.
pub fn parse_document(content: &str, frontmatter: &Frontmatter) -> Result<Document, ParseError> {
    use pulldown_cmark::{HeadingLevel, Options};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    let events: Vec<Event<'_>> = pulldown_cmark::Parser::new_ext(content, options).collect();

    // Inline style state
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
    let mut table_alignments: Vec<ColumnAlignment> = Vec::new();
    let mut table_header_cells: Vec<Vec<Inline>> = Vec::new();
    let mut table_rows: Vec<Vec<Vec<Inline>>> = Vec::new();
    let mut table_current_row: Vec<Vec<Inline>> = Vec::new();
    let mut table_count: usize = 0;

    let mut widget_fields: Vec<WidgetField> = Vec::new();
    let mut used_widget_configs: HashSet<String> = HashSet::new();
    let mut references_state = false;
    let mut display_refs: Vec<String> = Vec::new();

    let mut pending_text = String::new();
    let mut fragments: Vec<Inline> = Vec::new();
    let mut blocks: Vec<Block> = Vec::new();

    // Block directive stack
    let mut block_stack: Vec<BlockFrame> = Vec::new();
    let mut needs_style_table = false;

    // Resolve spacing from frontmatter
    let sp = frontmatter.spacing.as_ref();
    let sp_paragraph = sp.and_then(|s| s.paragraph).unwrap_or(8.0_f32);
    let sp_table = sp.and_then(|s| s.table).unwrap_or(8.0_f32);
    let sp_h1 = sp.and_then(|s| s.heading_h1).unwrap_or(16.0_f32);
    let sp_h2 = sp.and_then(|s| s.heading_h2).unwrap_or(12.0_f32);
    let sp_h3 = sp.and_then(|s| s.heading_h3).unwrap_or(8.0_f32);
    let sp_h4 = sp.and_then(|s| s.heading_h4).unwrap_or(4.0_f32);
    let sp_item = sp.and_then(|s| s.item);

    // ── Helper closures ────────────────────────────────────

    fn flush_pending(
        pending_text: &mut String,
        fragments: &mut Vec<Inline>,
        frontmatter: &Frontmatter,
        bold: bool,
        italic: bool,
        strikethrough: bool,
    ) -> Result<(), ParseError> {
        if pending_text.is_empty() {
            return Ok(());
        }
        let text = std::mem::take(pending_text);
        if text.contains("::")
            && text.contains('(')
            && parse_inline_styled_spans(
                &text,
                fragments,
                frontmatter,
                bold,
                italic,
                strikethrough,
            )?
        {
            return Ok(());
        }
        fragments.push(Inline::Text {
            text,
            bold,
            italic,
            strikethrough,
        });
        Ok(())
    }

    /// Emit accumulated fragments as a paragraph block.
    fn emit_paragraph(
        fragments: &mut Vec<Inline>,
        blocks: &mut Vec<Block>,
        blockquote_depth: usize,
        paragraph_spacing: f32,
    ) -> Option<StyleSuffix> {
        if fragments.is_empty() {
            return None;
        }

        let style_suffix = extract_style_suffix(fragments);

        if fragments.is_empty() {
            return style_suffix;
        }

        let para = Block::Paragraph {
            fragments: std::mem::take(fragments),
            style_suffix: style_suffix.clone(),
        };

        if blockquote_depth > 0 {
            blocks.push(Block::BlockQuote {
                depth: blockquote_depth,
                blocks: vec![para],
            });
        } else {
            blocks.push(para);
        }
        blocks.push(Block::Spacing(paragraph_spacing));
        style_suffix
    }

    /// Emit accumulated fragments as a list item.
    fn emit_list_item(
        fragments: &mut Vec<Inline>,
        blocks: &mut Vec<Block>,
        list_stack: &mut [Option<usize>],
        blockquote_depth: usize,
    ) -> Option<StyleSuffix> {
        if fragments.is_empty() {
            return None;
        }

        let style_suffix = extract_style_suffix(fragments);

        if fragments.is_empty() {
            return style_suffix;
        }

        // Determine list kind from current stack
        let kind = match list_stack.last() {
            Some(Some(n)) => ListKind::Ordered(*n),
            _ => ListKind::Unordered,
        };

        // Increment ordered counter
        if let Some(Some(n)) = list_stack.last_mut() {
            *n += 1;
        }

        let list_depth = list_stack.len();

        let item = ListItem {
            fragments: std::mem::take(fragments),
            children: Vec::new(),
            style_suffix: style_suffix.clone(),
            depth: list_depth,
            kind,
        };

        // If the last block is a List of the same kind, append to it;
        // otherwise create a new List.
        let needs_new_list = match blocks.last() {
            Some(Block::List {
                kind: existing_kind,
                ..
            }) => !matches!(
                (existing_kind, &kind),
                (ListKind::Unordered, ListKind::Unordered)
                    | (ListKind::Ordered(_), ListKind::Ordered(_))
            ),
            _ => true,
        };

        if needs_new_list {
            let list_block = if blockquote_depth > 0 {
                Block::BlockQuote {
                    depth: blockquote_depth,
                    blocks: vec![Block::List {
                        kind,
                        items: vec![item],
                    }],
                }
            } else {
                Block::List {
                    kind,
                    items: vec![item],
                }
            };
            blocks.push(list_block);
        } else {
            // Append to existing list
            if blockquote_depth > 0 {
                if let Some(Block::BlockQuote { blocks: inner, .. }) = blocks.last_mut()
                    && let Some(Block::List { items, .. }) = inner.last_mut()
                {
                    items.push(item);
                }
            } else if let Some(Block::List { items, .. }) = blocks.last_mut() {
                items.push(item);
            }
        }

        style_suffix
    }

    fn emit_table(
        header_cells: &[Vec<Inline>],
        rows: &[Vec<Vec<Inline>>],
        num_columns: usize,
        table_id: usize,
        alignments: &[ColumnAlignment],
        blocks: &mut Vec<Block>,
        table_spacing: f32,
    ) {
        let headers: Vec<TableCell> = header_cells
            .iter()
            .map(|cell| TableCell {
                fragments: cell.clone(),
            })
            .collect();
        let body_rows: Vec<Vec<TableCell>> = rows
            .iter()
            .map(|row| {
                row.iter()
                    .map(|cell| TableCell {
                        fragments: cell.clone(),
                    })
                    .collect()
            })
            .collect();

        blocks.push(Block::Table {
            headers,
            rows: body_rows,
            num_columns,
            table_index: table_id,
            alignments: alignments.to_vec(),
        });
        blocks.push(Block::Spacing(table_spacing));
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
                    table_alignments = alignments
                        .iter()
                        .map(|a| match a {
                            pulldown_cmark::Alignment::Center => ColumnAlignment::Center,
                            pulldown_cmark::Alignment::Right => ColumnAlignment::Right,
                            _ => ColumnAlignment::Left,
                        })
                        .collect();
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
                    pending_text.clear();
                    fragments.clear();
                }
                Tag::Paragraph if in_table => {}
                Tag::Heading(level, _, _) => {
                    heading_level = Some(*level);
                    pending_text.clear();
                }
                Tag::BlockQuote => {
                    blockquote_depth += 1;
                }
                Tag::List(start) => {
                    flush_pending(
                        &mut pending_text,
                        &mut fragments,
                        frontmatter,
                        bold,
                        italic,
                        strikethrough,
                    )?;
                    if !fragments.is_empty() && !list_stack.is_empty() {
                        let _ = emit_list_item(
                            &mut fragments,
                            &mut blocks,
                            &mut list_stack,
                            blockquote_depth,
                        );
                    }
                    list_stack.push(start.map(|n| n as usize));
                }
                Tag::Emphasis => {
                    flush_pending(
                        &mut pending_text,
                        &mut fragments,
                        frontmatter,
                        bold,
                        italic,
                        strikethrough,
                    )?;
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
                    )?;
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
                    )?;
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
                    )?;
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
                    )?;
                    in_image = Some(dest.to_string());
                    pending_text.clear();
                }
                Tag::Paragraph | Tag::Item | Tag::FootnoteDefinition(_) => {}
            },
            Event::End(tag) => match tag {
                Tag::Table(_) => {
                    emit_table(
                        &table_header_cells,
                        &table_rows,
                        table_num_columns,
                        table_count,
                        &table_alignments,
                        &mut blocks,
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
                    )?;
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
                    )?;
                    let runtime_field = if !list_stack.is_empty() {
                        emit_list_item(
                            &mut fragments,
                            &mut blocks,
                            &mut list_stack,
                            blockquote_depth,
                        )
                    } else {
                        emit_paragraph(&mut fragments, &mut blocks, blockquote_depth, sp_paragraph)
                    };
                    // Handle runtime style fields
                    if let Some(StyleSuffix::Dynamic(field_name)) = &runtime_field {
                        needs_style_table = true;
                        // Route to foreach row_fields or top-level widget_fields
                        let in_foreach = block_stack.iter_mut().rev().find_map(|frame| {
                            if let BlockDirectiveKind::Foreach { row_fields } = &mut frame.directive
                            {
                                Some(row_fields)
                            } else {
                                None
                            }
                        });
                        if let Some(row_fields) = in_foreach {
                            if !row_fields.iter().any(|rf| rf.name() == *field_name) {
                                row_fields.push(RowField::Widget {
                                    name: field_name.clone(),
                                    ty: WidgetType::String,
                                    kind: WidgetKind::Display,
                                });
                            }
                        } else {
                            references_state = true;
                            let already = widget_fields.iter().any(|f| f.name() == *field_name);
                            if !already {
                                widget_fields.push(WidgetField::Stateful {
                                    name: field_name.clone(),
                                    ty: WidgetType::String,
                                });
                            }
                        }
                    }
                }
                Tag::Heading(level, _, _) => {
                    let raw_text = std::mem::take(&mut pending_text);
                    let (text, style_key) = detect_style_suffix(&raw_text);
                    let text = text.to_owned();

                    let ast_level = match level {
                        HeadingLevel::H1 => crate::ast::HeadingLevel::H1,
                        HeadingLevel::H2 => crate::ast::HeadingLevel::H2,
                        HeadingLevel::H3 => crate::ast::HeadingLevel::H3,
                        _ => crate::ast::HeadingLevel::H4,
                    };
                    let heading_space = match level {
                        HeadingLevel::H1 => sp_h1,
                        HeadingLevel::H2 => sp_h2,
                        HeadingLevel::H3 => sp_h3,
                        _ => sp_h4,
                    };

                    let style_suffix = if let Some(key) = style_key {
                        // Validate style exists
                        if !key.starts_with('$') && !frontmatter.styles.contains_key(key) {
                            let hint = if frontmatter.widgets.contains_key(key) {
                                format!(
                                    " '{key}' is a widget config, not a style. Attach it to a widget directive like [slider](field){{{key}}}, not to a paragraph."
                                )
                            } else {
                                String::new()
                            };
                            return Err(ParseError::new(format!(
                                "Undefined style key '{key}' in frontmatter.{hint}"
                            )));
                        }
                        if let Some(field) = key.strip_prefix('$') {
                            Some(StyleSuffix::Dynamic(field.to_owned()))
                        } else {
                            Some(StyleSuffix::Static(key.to_owned()))
                        }
                    } else {
                        None
                    };

                    blocks.push(Block::Spacing(heading_space));
                    blocks.push(Block::Heading {
                        level: ast_level,
                        text,
                        style_suffix,
                    });
                    heading_level = None;
                }
                Tag::BlockQuote => {
                    blockquote_depth = blockquote_depth.saturating_sub(1);
                }
                Tag::List(_) => {
                    list_stack.pop();
                }
                Tag::Item => {
                    flush_pending(
                        &mut pending_text,
                        &mut fragments,
                        frontmatter,
                        bold,
                        italic,
                        strikethrough,
                    )?;
                    if !fragments.is_empty() {
                        let runtime_field = emit_list_item(
                            &mut fragments,
                            &mut blocks,
                            &mut list_stack,
                            blockquote_depth,
                        );
                        if let Some(StyleSuffix::Dynamic(field_name)) = &runtime_field {
                            needs_style_table = true;
                            let in_foreach = block_stack.iter_mut().rev().find_map(|frame| {
                                if let BlockDirectiveKind::Foreach { row_fields } =
                                    &mut frame.directive
                                {
                                    Some(row_fields)
                                } else {
                                    None
                                }
                            });
                            if let Some(row_fields) = in_foreach {
                                if !row_fields.iter().any(|rf| rf.name() == *field_name) {
                                    row_fields.push(RowField::Widget {
                                        name: field_name.clone(),
                                        ty: WidgetType::String,
                                        kind: WidgetKind::Display,
                                    });
                                }
                            } else {
                                references_state = true;
                                let already = widget_fields.iter().any(|f| f.name() == *field_name);
                                if !already {
                                    widget_fields.push(WidgetField::Stateful {
                                        name: field_name.clone(),
                                        ty: WidgetType::String,
                                    });
                                }
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
                    )?;
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
                    )?;
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
                    )?;
                    strikethrough = false;
                }
                Tag::CodeBlock(_info) => {
                    let code_text = std::mem::take(&mut pending_text);
                    blocks.push(Block::CodeBlock { text: code_text });
                    in_code_block = false;
                }
                Tag::Link(_link_type, dest, _title) => {
                    let raw_link_text = std::mem::take(&mut pending_text);
                    let url = dest.to_string();
                    let selector = parse_selectors(&raw_link_text);

                    if let Some(widget_kind) = WidgetKind::parse(&selector.base_name) {
                        // Widget directive detected
                        in_link = None;

                        // Lookahead for {attrs}
                        let mut widget_attrs = String::new();
                        if let Some(Event::Text(next_text)) = events.get(event_idx + 1) {
                            let t = next_text.as_ref().trim();
                            if t.starts_with('{') && t.ends_with('}') {
                                widget_attrs = t[1..t.len() - 1].to_string();
                                skip_next = true;
                            }
                        }
                        if !widget_attrs.is_empty() {
                            used_widget_configs.insert(widget_attrs.clone());
                        }

                        // Record widget field based on type.
                        // Collect into a local vec first, then route to
                        // either top-level widget_fields or foreach row_fields.
                        let content = url.clone();
                        let mut new_fields: Vec<(String, WidgetType)> = Vec::new();
                        match widget_kind {
                            WidgetKind::Button => {
                                if !widget_attrs.is_empty() {
                                    let wdef = frontmatter
                                        .widgets
                                        .get(&widget_attrs)
                                        .cloned()
                                        .unwrap_or_default();
                                    new_fields
                                        .push((format!("{widget_attrs}_count"), WidgetType::U32));
                                    if wdef.track_hover.unwrap_or(false) {
                                        new_fields.push((
                                            format!("{widget_attrs}_hovered"),
                                            WidgetType::Bool,
                                        ));
                                    }
                                    if wdef.track_secondary.unwrap_or(false) {
                                        new_fields.push((
                                            format!("{widget_attrs}_secondary_count"),
                                            WidgetType::U32,
                                        ));
                                    }
                                }
                            }
                            WidgetKind::Progress => {
                                if content.parse::<f32>().is_err() {
                                    new_fields.push((content.clone(), WidgetType::F64));
                                }
                            }
                            WidgetKind::Spinner => {}
                            WidgetKind::Slider | WidgetKind::Dragvalue => {
                                new_fields.push((content.clone(), WidgetType::F64));
                            }
                            WidgetKind::DoubleSlider => {
                                new_fields.push((format!("{content}_low"), WidgetType::F64));
                                new_fields.push((format!("{content}_high"), WidgetType::F64));
                            }
                            WidgetKind::Checkbox | WidgetKind::Toggle => {
                                new_fields.push((content.clone(), WidgetType::Bool));
                            }
                            WidgetKind::Textedit | WidgetKind::Textarea | WidgetKind::Password => {
                                new_fields.push((content.clone(), WidgetType::String));
                            }
                            WidgetKind::Display => {
                                references_state = true;
                                display_refs.push(content.clone());
                                let already_declared =
                                    widget_fields.iter().any(|f| f.name() == content);
                                if !already_declared {
                                    new_fields.push((content.clone(), WidgetType::String));
                                }
                            }
                            WidgetKind::Radio | WidgetKind::Combobox | WidgetKind::Selectable => {
                                new_fields.push((content.clone(), WidgetType::Usize));
                            }
                            WidgetKind::Color => {
                                new_fields.push((content.clone(), WidgetType::ByteArray4));
                            }
                            WidgetKind::Select => {
                                new_fields.push((content.clone(), WidgetType::Usize));
                                if !widget_attrs.is_empty() {
                                    new_fields.push((widget_attrs.clone(), WidgetType::VecString));
                                }
                            }
                            WidgetKind::Log => {
                                new_fields.push((content.clone(), WidgetType::VecString));
                            }
                            WidgetKind::Datepicker => {
                                new_fields.push((content.clone(), WidgetType::Date));
                            }
                        }

                        // Route fields: foreach row_fields or top-level widget_fields
                        let in_foreach = block_stack.iter_mut().rev().find_map(|frame| {
                            if let BlockDirectiveKind::Foreach { row_fields } = &mut frame.directive
                            {
                                Some(row_fields)
                            } else {
                                None
                            }
                        });
                        if let Some(row_fields) = in_foreach {
                            for (name, ty) in new_fields {
                                if !row_fields.iter().any(|rf| rf.name() == name) {
                                    row_fields.push(RowField::Widget {
                                        name,
                                        ty,
                                        kind: widget_kind,
                                    });
                                }
                            }
                        } else {
                            for (name, ty) in new_fields {
                                widget_fields.push(WidgetField::Stateful { name, ty });
                            }
                        }

                        let widget = WidgetDirective {
                            widget_type: widget_kind,
                            field: url,
                            id: selector.id,
                            classes: selector.classes,
                            config_key: widget_attrs,
                        };

                        if in_table {
                            fragments.push(Inline::Widget(widget));
                        } else {
                            blocks.push(Block::Widget(widget));
                        }
                    } else {
                        // Not a widget — check for typos or emit as link
                        if !selector.base_name.is_empty()
                            && !url.starts_with("http://")
                            && !url.starts_with("https://")
                            && !url.starts_with("mailto:")
                            && !url.starts_with("file://")
                            && !url.starts_with('#')
                            && !url.starts_with('/')
                        {
                            return Err(ParseError::new(format!(
                                "Unknown widget or link '[{}]({})'. \
                                 If this is a widget, valid names are: {}. \
                                 If this is a link, the URL must start with \
                                 http://, https://, mailto:, file://, #, or /.",
                                selector.base_name,
                                url,
                                WidgetKind::ALL_NAMES.join(", ")
                            )));
                        }

                        let class_style = resolve_classes(&selector.classes, frontmatter)?;

                        if selector.base_name.is_empty() && class_style.is_some() {
                            // Styled inline text span: [.class](text)
                            let display_content = url.replace('_', " ");
                            fragments.push(Inline::ClassSpan {
                                classes: selector.classes,
                                text: display_content,
                                bold,
                                italic,
                                strikethrough,
                            });
                        } else {
                            // Normal hyperlink
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
                            fragments.push(Inline::Link {
                                text: selector.base_name,
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

                    if in_table {
                        fragments.push(Inline::Image { alt, url });
                    } else {
                        blocks.push(Block::Image { alt, url });
                    }
                }
                Tag::FootnoteDefinition(_) => {}
            },
            Event::Text(text) => {
                let text_str: &str = text.as_ref();
                let trimmed = text_str.trim();

                // Detect ::: block fence directives
                if let Some(after_fence) = trimmed.strip_prefix(":::") {
                    let rest = after_fence.trim();
                    if rest == "next" {
                        // Column/section separator
                        if let Some(frame) = block_stack.last_mut() {
                            match &mut frame.directive {
                                BlockDirectiveKind::Columns {
                                    count,
                                    current_col,
                                    column_bodies,
                                    ..
                                } => {
                                    column_bodies[*current_col] = std::mem::take(&mut blocks);
                                    *current_col += 1;
                                    if *current_col >= *count {
                                        return Err(ParseError::new(format!(
                                            "::: next used too many times — only {count} columns defined"
                                        )));
                                    }
                                }
                                BlockDirectiveKind::Horizontal {
                                    align: HorizontalAlign::SpaceBetween,
                                    left_body,
                                } => {
                                    // First ::: next splits left/right in space-between
                                    *left_body = std::mem::take(&mut blocks);
                                }
                                _ => {
                                    return Err(ParseError::new(
                                        "::: next can only be used inside ::: columns or ::: horizontal space-between",
                                    ));
                                }
                            }
                        } else {
                            return Err(ParseError::new("::: next outside of any block directive"));
                        }
                        event_idx += 1;
                        continue;
                    }
                    if rest.is_empty() || rest.starts_with('/') {
                        // Close the innermost block
                        if let Some(frame) = block_stack.pop() {
                            let body = std::mem::take(&mut blocks);
                            blocks = frame.saved_blocks;

                            match frame.directive {
                                BlockDirectiveKind::Foreach { row_fields } => {
                                    if row_fields.is_empty() {
                                        #[expect(
                                            clippy::literal_string_with_formatting_args,
                                            reason = "{{field}} is an escaped literal, not a format arg"
                                        )]
                                        return Err(ParseError::new(format!(
                                            "[foreach]({}) body contains no {{field}} references. \
                                             Ensure blank lines surround the table or list inside the \
                                             foreach block — CommonMark requires paragraph separation \
                                             for block-level elements like tables.",
                                            frame.field_name
                                        )));
                                    }
                                    widget_fields.push(WidgetField::Foreach {
                                        name: frame.field_name.clone(),
                                        row_fields,
                                    });
                                    blocks.push(Block::Directive(Directive::Foreach {
                                        field: frame.field_name,
                                        body,
                                        row_fields: Vec::new(), // row_fields already captured in WidgetField
                                    }));
                                }
                                BlockDirectiveKind::If => {
                                    blocks.push(Block::Directive(Directive::If {
                                        field: frame.field_name,
                                        body,
                                    }));
                                }
                                BlockDirectiveKind::Style => {
                                    needs_style_table = true;
                                    blocks.push(Block::Directive(Directive::Style {
                                        field: frame.field_name,
                                        body,
                                    }));
                                }
                                BlockDirectiveKind::Horizontal { align, left_body } => {
                                    if align == HorizontalAlign::SpaceBetween {
                                        // body = right side (after ::: next)
                                        // left_body = left side (saved at ::: next)
                                        blocks.push(Block::Directive(Directive::Horizontal {
                                            align,
                                            body: left_body,
                                            right_body: body,
                                        }));
                                    } else {
                                        blocks.push(Block::Directive(Directive::Horizontal {
                                            align,
                                            body,
                                            right_body: Vec::new(),
                                        }));
                                    }
                                }
                                BlockDirectiveKind::Columns {
                                    count: _,
                                    weights,
                                    current_col,
                                    mut column_bodies,
                                } => {
                                    column_bodies[current_col] = body;
                                    blocks.push(Block::Directive(Directive::Columns {
                                        count: column_bodies.len(),
                                        weights,
                                        columns: column_bodies,
                                    }));
                                }
                                BlockDirectiveKind::Center => {
                                    blocks.push(Block::Directive(Directive::Center { body }));
                                }
                                BlockDirectiveKind::Right => {
                                    blocks.push(Block::Directive(Directive::Right { body }));
                                }
                                BlockDirectiveKind::Fill => {
                                    blocks.push(Block::Directive(Directive::Fill { body }));
                                }
                                BlockDirectiveKind::Frame { style_name } => {
                                    blocks.push(Block::Directive(Directive::Frame {
                                        style_name,
                                        body,
                                    }));
                                }
                            }
                        }
                    } else {
                        // Open a new block
                        let (directive_name, arg) = rest.split_once(' ').unwrap_or((rest, ""));
                        let field_name = arg.to_owned();

                        match directive_name {
                            "foreach" => {
                                block_stack.push(BlockFrame {
                                    directive: BlockDirectiveKind::Foreach {
                                        row_fields: Vec::new(),
                                    },
                                    field_name,
                                    saved_blocks: std::mem::take(&mut blocks),
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
                                    directive: BlockDirectiveKind::If,
                                    field_name,
                                    saved_blocks: std::mem::take(&mut blocks),
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
                                    directive: BlockDirectiveKind::Style,
                                    field_name,
                                    saved_blocks: std::mem::take(&mut blocks),
                                });
                            }
                            "frame" => {
                                let style_name = if field_name.is_empty() {
                                    None
                                } else {
                                    Some(field_name.clone())
                                };
                                block_stack.push(BlockFrame {
                                    directive: BlockDirectiveKind::Frame { style_name },
                                    field_name,
                                    saved_blocks: std::mem::take(&mut blocks),
                                });
                            }
                            "horizontal" => {
                                let align = match field_name.as_str() {
                                    "center" => HorizontalAlign::Center,
                                    "right" => HorizontalAlign::Right,
                                    "space-between" => HorizontalAlign::SpaceBetween,
                                    "" => HorizontalAlign::Left,
                                    other => {
                                        return Err(ParseError::new(format!(
                                            "Unknown horizontal alignment '{other}'. \
                                             Valid: center, right, space-between"
                                        )));
                                    }
                                };
                                block_stack.push(BlockFrame {
                                    directive: BlockDirectiveKind::Horizontal {
                                        align,
                                        left_body: Vec::new(),
                                    },
                                    field_name: String::new(),
                                    saved_blocks: std::mem::take(&mut blocks),
                                });
                            }
                            "columns" => {
                                // Parse either plain integer or colon-separated weights
                                let (count, weights) = if field_name.contains(':') {
                                    let weights: Vec<usize> = field_name
                                        .split(':')
                                        .map(|w| {
                                            w.trim().parse::<usize>().map_err(|_err| {
                                                ParseError::new(format!(
                                                    "::: columns weight must be a positive integer, got '{w}'"
                                                ))
                                            })
                                        })
                                        .collect::<Result<Vec<_>, _>>()?;
                                    if weights.len() < 2 {
                                        return Err(ParseError::new(
                                            "::: columns weights need at least 2 values (e.g., 3:1)",
                                        ));
                                    }
                                    (weights.len(), weights)
                                } else {
                                    let count: usize = field_name.parse().map_err(|_err| {
                                        ParseError::new(format!(
                                            "::: columns requires a number or weights (e.g., 3 or 3:1:1), got '{field_name}'"
                                        ))
                                    })?;
                                    (count, Vec::new())
                                };
                                block_stack.push(BlockFrame {
                                    directive: BlockDirectiveKind::Columns {
                                        count,
                                        weights,
                                        current_col: 0,
                                        column_bodies: vec![Vec::new(); count],
                                    },
                                    field_name: String::new(),
                                    saved_blocks: std::mem::take(&mut blocks),
                                });
                            }
                            "center" => {
                                block_stack.push(BlockFrame {
                                    directive: BlockDirectiveKind::Center,
                                    field_name: String::new(),
                                    saved_blocks: std::mem::take(&mut blocks),
                                });
                            }
                            "right" => {
                                block_stack.push(BlockFrame {
                                    directive: BlockDirectiveKind::Right,
                                    field_name: String::new(),
                                    saved_blocks: std::mem::take(&mut blocks),
                                });
                            }
                            "fill" => {
                                block_stack.push(BlockFrame {
                                    directive: BlockDirectiveKind::Fill,
                                    field_name: String::new(),
                                    saved_blocks: std::mem::take(&mut blocks),
                                });
                            }
                            _ => {
                                return Err(ParseError::new(format!(
                                    "Unknown block directive ':::{directive_name}'. \
                                     Valid: foreach, if, style, frame, horizontal, columns, center, right, fill"
                                )));
                            }
                        }
                    }
                    event_idx += 1;
                    continue;
                }

                // Inside foreach: parse {field} references
                let in_foreach = matches!(
                    block_stack.last().map(|f| &f.directive),
                    Some(BlockDirectiveKind::Foreach { .. })
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
                        )?;
                        if let Some(BlockFrame {
                            directive: BlockDirectiveKind::Foreach { row_fields },
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
                )?;
                fragments.push(Inline::InlineCode(code_text.to_string()));
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
                )?;
                let _ = emit_paragraph(&mut fragments, &mut blocks, blockquote_depth, sp_paragraph);
            }
            Event::Rule => {
                blocks.push(Block::HorizontalRule);
            }
            Event::Html(_) | Event::FootnoteReference(_) | Event::TaskListMarker(_) => {}
        }
        event_idx += 1;
    }

    // Prepend item_spacing override if configured
    if let Some(item_sp) = sp_item {
        blocks.insert(0, Block::ItemSpacingOverride(item_sp));
    }

    Ok(Document {
        blocks,
        widget_fields,
        references_state,
        display_refs,
        needs_style_table,
        used_widget_configs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontmatter::{Frontmatter, StyleDef};

    fn parse(md: &str) -> Document {
        parse_document(md, &Frontmatter::default()).unwrap()
    }

    fn parse_with_fm(md: &str, fm: &Frontmatter) -> Document {
        parse_document(md, fm).unwrap()
    }

    // ── Headings ──────────────────────────────────────────

    #[test]
    fn heading_h1() {
        let doc = parse("# Hello");
        // First block is spacing, second is heading
        assert!(doc.blocks.len() >= 2);
        match &doc.blocks[1] {
            Block::Heading {
                level,
                text,
                style_suffix,
            } => {
                assert_eq!(*level, HeadingLevel::H1);
                assert_eq!(text, "Hello");
                assert!(style_suffix.is_none());
            }
            other => panic!("Expected Heading, got {other:?}"),
        }
    }

    #[test]
    fn heading_with_style_suffix() {
        let mut fm = Frontmatter::default();
        fm.styles.insert(
            "title".to_owned(),
            StyleDef {
                color: Some("#FF0000".to_owned()),
                ..StyleDef::default()
            },
        );
        let doc = parse_with_fm("# Welcome ::title", &fm);
        match &doc.blocks[1] {
            Block::Heading {
                text, style_suffix, ..
            } => {
                assert_eq!(text, "Welcome");
                assert_eq!(*style_suffix, Some(StyleSuffix::Static("title".to_owned())));
            }
            other => panic!("Expected Heading, got {other:?}"),
        }
    }

    #[test]
    fn heading_h2_h3_h4() {
        let doc = parse("## Two\n### Three\n#### Four");
        let headings: Vec<_> = doc
            .blocks
            .iter()
            .filter(|b| matches!(b, Block::Heading { .. }))
            .collect();
        assert_eq!(headings.len(), 3);
        match &headings[0] {
            Block::Heading { level, text, .. } => {
                assert_eq!(*level, HeadingLevel::H2);
                assert_eq!(text, "Two");
            }
            _ => panic!(),
        }
        match &headings[1] {
            Block::Heading { level, text, .. } => {
                assert_eq!(*level, HeadingLevel::H3);
                assert_eq!(text, "Three");
            }
            _ => panic!(),
        }
        match &headings[2] {
            Block::Heading { level, text, .. } => {
                assert_eq!(*level, HeadingLevel::H4);
                assert_eq!(text, "Four");
            }
            _ => panic!(),
        }
    }

    // ── Paragraphs ────────────────────────────────────────

    #[test]
    fn simple_paragraph() {
        let doc = parse("Hello world");
        let para = doc
            .blocks
            .iter()
            .find(|b| matches!(b, Block::Paragraph { .. }));
        assert!(para.is_some());
        match para.unwrap() {
            Block::Paragraph { fragments, .. } => {
                assert_eq!(fragments.len(), 1);
                match &fragments[0] {
                    Inline::Text {
                        text,
                        bold,
                        italic,
                        strikethrough,
                    } => {
                        assert_eq!(text, "Hello world");
                        assert!(!bold);
                        assert!(!italic);
                        assert!(!strikethrough);
                    }
                    other => panic!("Expected Text, got {other:?}"),
                }
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn bold_italic_paragraph() {
        let doc = parse("Hello **bold** and *italic* text");
        let para = doc
            .blocks
            .iter()
            .find(|b| matches!(b, Block::Paragraph { .. }));
        match para.unwrap() {
            Block::Paragraph { fragments, .. } => {
                assert!(fragments.len() >= 3);
                // "Hello " — plain
                // "bold" — bold
                // " and " — plain
                // "italic" — italic
                // " text" — plain
                let bold_frag = fragments
                    .iter()
                    .find(|f| matches!(f, Inline::Text { bold: true, .. }));
                assert!(bold_frag.is_some());
                let italic_frag = fragments
                    .iter()
                    .find(|f| matches!(f, Inline::Text { italic: true, .. }));
                assert!(italic_frag.is_some());
            }
            _ => unreachable!(),
        }
    }

    // ── Inline code ───────────────────────────────────────

    #[test]
    fn inline_code() {
        let doc = parse("Use `foo()` here");
        let para = doc
            .blocks
            .iter()
            .find(|b| matches!(b, Block::Paragraph { .. }));
        match para.unwrap() {
            Block::Paragraph { fragments, .. } => {
                let code = fragments
                    .iter()
                    .find(|f| matches!(f, Inline::InlineCode(_)));
                assert!(code.is_some());
                match code.unwrap() {
                    Inline::InlineCode(text) => assert_eq!(text, "foo()"),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    // ── Links ─────────────────────────────────────────────

    #[test]
    fn hyperlink() {
        let doc = parse("[Click here](https://example.com)");
        let para = doc
            .blocks
            .iter()
            .find(|b| matches!(b, Block::Paragraph { .. }));
        match para.unwrap() {
            Block::Paragraph { fragments, .. } => {
                let link = fragments.iter().find(|f| matches!(f, Inline::Link { .. }));
                assert!(link.is_some());
                match link.unwrap() {
                    Inline::Link { text, url, .. } => {
                        assert_eq!(text, "Click here");
                        assert_eq!(url, "https://example.com");
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    // ── Code blocks ───────────────────────────────────────

    #[test]
    fn code_block() {
        let doc = parse("```\nlet x = 1;\n```");
        let cb = doc
            .blocks
            .iter()
            .find(|b| matches!(b, Block::CodeBlock { .. }));
        assert!(cb.is_some());
        match cb.unwrap() {
            Block::CodeBlock { text } => assert!(text.contains("let x = 1;"), "Got: {text}"),
            _ => unreachable!(),
        }
    }

    // ── Horizontal rule ───────────────────────────────────

    #[test]
    fn horizontal_rule() {
        let doc = parse("---");
        assert!(
            doc.blocks
                .iter()
                .any(|b| matches!(b, Block::HorizontalRule))
        );
    }

    // ── Tables ────────────────────────────────────────────

    #[test]
    fn simple_table() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |";
        let doc = parse(md);
        let table = doc.blocks.iter().find(|b| matches!(b, Block::Table { .. }));
        assert!(table.is_some());
        match table.unwrap() {
            Block::Table {
                headers,
                rows,
                num_columns,
                ..
            } => {
                assert_eq!(*num_columns, 2);
                assert_eq!(headers.len(), 2);
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].len(), 2);
            }
            _ => unreachable!(),
        }
    }

    // ── Widgets ───────────────────────────────────────────

    #[test]
    fn slider_widget() {
        let doc = parse("[slider](volume)");
        let widget = doc.blocks.iter().find(|b| matches!(b, Block::Widget(_)));
        assert!(widget.is_some());
        match widget.unwrap() {
            Block::Widget(w) => {
                assert_eq!(w.widget_type, WidgetKind::Slider);
                assert_eq!(w.field, "volume");
                assert!(w.id.is_none());
                assert!(w.classes.is_empty());
                assert!(w.config_key.is_empty());
            }
            _ => unreachable!(),
        }
        assert!(doc.widget_fields.iter().any(|f| f.name() == "volume"));
    }

    #[test]
    fn button_widget_with_config() {
        let doc = parse("[button](click){ok}");
        let widget = doc.blocks.iter().find(|b| matches!(b, Block::Widget(_)));
        match widget.unwrap() {
            Block::Widget(w) => {
                assert_eq!(w.widget_type, WidgetKind::Button);
                assert_eq!(w.field, "click");
                assert_eq!(w.config_key, "ok");
            }
            _ => unreachable!(),
        }
        assert!(doc.used_widget_configs.contains("ok"));
    }

    #[test]
    fn widget_with_selectors() {
        let doc = parse("[button#submit.primary](click)");
        let widget = doc.blocks.iter().find(|b| matches!(b, Block::Widget(_)));
        match widget.unwrap() {
            Block::Widget(w) => {
                assert_eq!(w.widget_type, WidgetKind::Button);
                assert_eq!(w.id, Some("submit".to_owned()));
                assert_eq!(w.classes, vec!["primary"]);
            }
            _ => unreachable!(),
        }
    }

    // ── Error handling ────────────────────────────────────

    #[test]
    fn unknown_widget_error() {
        let result = parse_document("[foo](bar)", &Frontmatter::default());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("Unknown widget or link")
        );
    }

    #[test]
    fn undefined_style_in_heading() {
        let result = parse_document("# Hello ::missing", &Frontmatter::default());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Undefined style key"));
    }

    // ── Lists ─────────────────────────────────────────────

    #[test]
    fn unordered_list() {
        let doc = parse("- one\n- two\n- three");
        let list = doc.blocks.iter().find(|b| matches!(b, Block::List { .. }));
        assert!(list.is_some());
        match list.unwrap() {
            Block::List { kind, items } => {
                assert_eq!(*kind, ListKind::Unordered);
                assert_eq!(items.len(), 3);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn ordered_list() {
        let doc = parse("1. first\n2. second");
        let list = doc.blocks.iter().find(|b| matches!(b, Block::List { .. }));
        assert!(list.is_some());
        match list.unwrap() {
            Block::List { kind, items } => {
                assert!(matches!(*kind, ListKind::Ordered(_)));
                assert_eq!(items.len(), 2);
            }
            _ => unreachable!(),
        }
    }

    // ── Block directives ──────────────────────────────────

    #[test]
    fn columns_directive() {
        let md = "::: columns 2\n\nLeft content\n\n::: next\n\nRight content\n\n:::";
        let doc = parse(md);
        let dir = doc
            .blocks
            .iter()
            .find(|b| matches!(b, Block::Directive(Directive::Columns { .. })));
        assert!(dir.is_some(), "Blocks: {:?}", doc.blocks);
        match dir.unwrap() {
            Block::Directive(Directive::Columns { count, columns, .. }) => {
                assert_eq!(*count, 2);
                assert_eq!(columns.len(), 2);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn if_directive() {
        let md = "::: if show_details\n\nSome details\n\n:::";
        let doc = parse(md);
        let dir = doc
            .blocks
            .iter()
            .find(|b| matches!(b, Block::Directive(Directive::If { .. })));
        assert!(dir.is_some());
        assert!(doc.widget_fields.iter().any(|f| f.name() == "show_details"));
    }

    #[test]
    fn unknown_directive_error() {
        let result = parse_document("::: bogus thing\n\n:::", &Frontmatter::default());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("Unknown block directive")
        );
    }

    // ── Display widget ────────────────────────────────────

    #[test]
    fn display_widget_references_state() {
        let doc = parse("[display](score)");
        assert!(doc.references_state);
        assert!(doc.display_refs.contains(&"score".to_owned()));
    }

    #[test]
    fn nested_blockquotes() {
        let md = "> Level 1\n> > Level 2\n> > > Level 3";
        let doc = parse(md);
        let bqs: Vec<_> = doc
            .blocks
            .iter()
            .filter(|b| matches!(b, Block::BlockQuote { .. }))
            .collect();
        assert_eq!(bqs.len(), 3);
        // Verify depths
        match &bqs[0] {
            Block::BlockQuote { depth, .. } => assert_eq!(*depth, 1),
            _ => unreachable!(),
        }
        match &bqs[1] {
            Block::BlockQuote { depth, .. } => assert_eq!(*depth, 2),
            _ => unreachable!(),
        }
        match &bqs[2] {
            Block::BlockQuote { depth, .. } => assert_eq!(*depth, 3),
            _ => unreachable!(),
        }
    }

    #[test]
    fn nested_ordered_list() {
        let md = "1. First\n2. Second\n    1. Nested A\n    2. Nested B";
        let doc = parse(md);
        let list = doc.blocks.iter().find(|b| matches!(b, Block::List { .. }));
        assert!(list.is_some());
        match list.unwrap() {
            Block::List { items, .. } => {
                // Should have 4 items total (flat), with depths 1,1,2,2
                assert_eq!(items.len(), 4);
                assert_eq!(items[0].depth, 1);
                assert_eq!(items[1].depth, 1);
                assert_eq!(items[2].depth, 2);
                assert_eq!(items[3].depth, 2);
            }
            _ => unreachable!(),
        }
    }

    // ── Alignment directives ─────────────────────────────

    #[test]
    fn center_directive() {
        let md = "::: center\n\nCentered text\n\n:::";
        let doc = parse(md);
        let dir = doc
            .blocks
            .iter()
            .find(|b| matches!(b, Block::Directive(Directive::Center { .. })));
        assert!(dir.is_some(), "Blocks: {:?}", doc.blocks);
    }

    #[test]
    fn right_directive() {
        let md = "::: right\n\nRight text\n\n:::";
        let doc = parse(md);
        let dir = doc
            .blocks
            .iter()
            .find(|b| matches!(b, Block::Directive(Directive::Right { .. })));
        assert!(dir.is_some());
    }

    #[test]
    fn fill_directive() {
        let md = "::: fill\n\nFill text\n\n:::";
        let doc = parse(md);
        let dir = doc
            .blocks
            .iter()
            .find(|b| matches!(b, Block::Directive(Directive::Fill { .. })));
        assert!(dir.is_some());
    }

    #[test]
    fn horizontal_center() {
        let md = "::: horizontal center\n\n[button](ok)\n\n:::";
        let doc = parse(md);
        let dir = doc.blocks.iter().find(|b| {
            matches!(
                b,
                Block::Directive(Directive::Horizontal {
                    align: HorizontalAlign::Center,
                    ..
                })
            )
        });
        assert!(dir.is_some());
    }

    #[test]
    fn horizontal_right() {
        let md = "::: horizontal right\n\n[button](ok)\n\n:::";
        let doc = parse(md);
        let dir = doc.blocks.iter().find(|b| {
            matches!(
                b,
                Block::Directive(Directive::Horizontal {
                    align: HorizontalAlign::Right,
                    ..
                })
            )
        });
        assert!(dir.is_some());
    }

    #[test]
    fn horizontal_space_between() {
        let md =
            "::: horizontal space-between\n\n[button](back)\n\n::: next\n\n[button](next)\n\n:::";
        let doc = parse(md);
        let dir = doc.blocks.iter().find(|b| {
            matches!(
                b,
                Block::Directive(Directive::Horizontal {
                    align: HorizontalAlign::SpaceBetween,
                    ..
                })
            )
        });
        assert!(dir.is_some(), "Blocks: {:?}", doc.blocks);
    }

    #[test]
    fn weighted_columns() {
        let md = "::: columns 3:1\n\nMain content\n\n::: next\n\nSidebar\n\n:::";
        let doc = parse(md);
        let dir = doc
            .blocks
            .iter()
            .find(|b| matches!(b, Block::Directive(Directive::Columns { .. })));
        assert!(dir.is_some());
        match dir.unwrap() {
            Block::Directive(Directive::Columns {
                count,
                weights,
                columns,
            }) => {
                assert_eq!(*count, 2);
                assert_eq!(*weights, vec![3, 1]);
                assert_eq!(columns.len(), 2);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn weighted_columns_three_way() {
        let md = "::: columns 1:2:1\n\nLeft\n\n::: next\n\nCenter\n\n::: next\n\nRight\n\n:::";
        let doc = parse(md);
        match doc
            .blocks
            .iter()
            .find(|b| matches!(b, Block::Directive(Directive::Columns { .. })))
            .unwrap()
        {
            Block::Directive(Directive::Columns { count, weights, .. }) => {
                assert_eq!(*count, 3);
                assert_eq!(*weights, vec![1, 2, 1]);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn table_column_alignment() {
        let md = "| Left | Center | Right |\n|:-----|:------:|------:|\n| a | b | c |";
        let doc = parse(md);
        let table = doc.blocks.iter().find(|b| matches!(b, Block::Table { .. }));
        assert!(table.is_some());
        match table.unwrap() {
            Block::Table { alignments, .. } => {
                assert_eq!(alignments.len(), 3);
                assert_eq!(alignments[0], ColumnAlignment::Left);
                assert_eq!(alignments[1], ColumnAlignment::Center);
                assert_eq!(alignments[2], ColumnAlignment::Right);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn invalid_horizontal_align_error() {
        let result = parse_document(
            "::: horizontal bogus\n\ntext\n\n:::",
            &Frontmatter::default(),
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("Unknown horizontal alignment")
        );
    }
}
