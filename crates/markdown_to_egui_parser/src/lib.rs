//! YAML frontmatter parsing and style resolution.
//!
//! Markdown files may begin with a `---`-delimited YAML block that defines
//! reusable style presets, widget configurations, and page metadata. This
//! module handles the full pipeline:
//!
//! 1. [`strip_frontmatter()`] splits raw file content into YAML + markdown
//!    (must happen before pulldown-cmark sees the file, since `---` would
//!    be parsed as `ThematicBreak`).
//! 2. `serde_yaml` deserializes the YAML into a [`Frontmatter`] struct.
//! 3. [`merge_frontmatter()`] merges a parent's styles/widgets into a child
//!    (used by `define_markdown_app!` with the `parent:` keyword).
//! 4. At flush points in the event loop, [`detect_style_suffix()`] checks for
//!    trailing `::key` suffixes, and [`style_def_to_label_tokens()`]
//!    emits the corresponding `styled_label_rich()` call.
//!
//! See `knowledge/frontmatter-and-styles.md` for the full style system design.

use std::collections::HashMap;
use serde::Deserialize;
use proc_macro2::TokenStream;
use quote::quote;

// -- Frontmatter types --
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Frontmatter {
    #[serde(default)]
    pub page: Option<PageDef>,
    #[serde(default)]
    pub styles: HashMap<String, StyleDef>,
    #[serde(default)]
    pub widgets: HashMap<String, WidgetDef>,
    #[serde(default)]
    pub spacing: Option<SpacingDef>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct SpacingDef {
    pub paragraph: Option<f32>,
    pub table: Option<f32>,
    pub heading_h1: Option<f32>,
    pub heading_h2: Option<f32>,
    pub heading_h3: Option<f32>,
    pub heading_h4: Option<f32>,
    pub item: Option<f32>,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct PageDef {
    pub name: String,
    pub label: String,
    #[serde(default)]
    pub default: bool,
    #[serde(default)]
    pub panel: Option<String>,
    #[serde(default)]
    pub width: Option<f32>,
    #[serde(default)]
    pub height: Option<f32>,
    #[serde(default)]
    pub open: Option<String>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct WidgetDef {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub speed: Option<f64>,
    pub label: Option<String>,
    pub hint: Option<String>,
    pub format: Option<String>,
    pub options: Option<Vec<String>>,
    pub track_hover: Option<bool>,
    pub track_secondary: Option<bool>,
    pub suffix: Option<String>,
    pub prefix: Option<String>,
    pub rows: Option<usize>,
    pub max_height: Option<f64>,
    pub fill: Option<String>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct StyleDef {
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub strikethrough: Option<bool>,
    pub underline: Option<bool>,
    pub color: Option<String>,
    pub background: Option<String>,
    pub size: Option<f32>,
    pub monospace: Option<bool>,
    pub weak: Option<bool>,
    pub inner_margin: Option<f32>,
    pub outer_margin: Option<f32>,
    pub stroke: Option<f32>,
    pub stroke_color: Option<String>,
    pub corner_radius: Option<f32>,
}

// --- Frontmatter merging and parsing functions ---

pub fn merge_frontmatter(parent: &Frontmatter, child: Frontmatter) -> Frontmatter {
    let mut styles = parent.styles.clone();
    for (k, v) in child.styles {
        styles.insert(k, v);
    }
    let mut widgets = parent.widgets.clone();
    for (k, v) in child.widgets {
        widgets.insert(k, v);
    }
    let spacing = match (parent.spacing.clone(), child.spacing) {
        (Some(p), Some(c)) => Some(merge_spacing_defs(&p, &c)),
        (None, c @ Some(_)) => c,
        (p @ Some(_), None) => p,
        (None, None) => None,
    };
    Frontmatter {
        page: child.page,
        styles,
        widgets,
        spacing,
    }
}

pub fn merge_spacing_defs(parent: &SpacingDef, child: &SpacingDef) -> SpacingDef {
    SpacingDef {
        paragraph: child.paragraph.or(parent.paragraph),
        table: child.table.or(parent.table),
        heading_h1: child.heading_h1.or(parent.heading_h1),
        heading_h2: child.heading_h2.or(parent.heading_h2),
        heading_h3: child.heading_h3.or(parent.heading_h3),
        heading_h4: child.heading_h4.or(parent.heading_h4),
        item: child.item.or(parent.item),
    }
}

pub fn merge_style_defs(base: &StyleDef, overlay: &StyleDef) -> StyleDef {
    StyleDef {
        bold: overlay.bold.or(base.bold),
        italic: overlay.italic.or(base.italic),
        strikethrough: overlay.strikethrough.or(base.strikethrough),
        underline: overlay.underline.or(base.underline),
        color: overlay.color.clone().or_else(|| base.color.clone()),
        background: overlay.background.clone().or_else(|| base.background.clone()),
        size: overlay.size.or(base.size),
        monospace: overlay.monospace.or(base.monospace),
        weak: overlay.weak.or(base.weak),
        inner_margin: overlay.inner_margin.or(base.inner_margin),
        outer_margin: overlay.outer_margin.or(base.outer_margin),
        stroke: overlay.stroke.or(base.stroke),
        stroke_color: overlay.stroke_color.clone().or_else(|| base.stroke_color.clone()),
        corner_radius: overlay.corner_radius.or(base.corner_radius),
    }
}

// --- Frontmatter utility functions (selectors, parsing, etc.) ---

pub struct ParsedSelector {
    pub base_name: String,
    pub id: Option<String>,
    pub classes: Vec<String>,
}

pub fn parse_selectors(link_text: &str) -> ParsedSelector {
    let split_pos = link_text.find(|c: char| c == '#' || c == '.');
    let (base, remainder) = match split_pos {
        Some(pos) => (&link_text[..pos], &link_text[pos..]),
        None => (link_text, ""),
    };
    let mut id = None;
    let mut classes = Vec::new();
    if !remainder.is_empty() {
        let mut current = String::new();
        let mut current_type = ' ';
        for ch in remainder.chars() {
            if ch == '#' || ch == '.' {
                if !current.is_empty() {
                    match current_type {
                        '#' => id = Some(std::mem::take(&mut current)),
                        '.' => classes.push(std::mem::take(&mut current)),
                        _ => {}
                    }
                }
                current_type = ch;
                current.clear();
            } else {
                current.push(ch);
            }
        }
        if !current.is_empty() {
            match current_type {
                '#' => id = Some(current),
                '.' => classes.push(current),
                _ => {}
            }
        }
    }
    ParsedSelector {
        base_name: base.to_string(),
        id,
        classes,
    }
}

// --- AST types for markdown parser ---

#[derive(Debug, Clone, PartialEq)]
pub enum Fragment {
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
    // Widget and ForeachField variants omitted for now (require proc-macro2)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WidgetType {
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

#[derive(Clone, Debug, PartialEq)]
pub enum WidgetField {
    Stateful { name: String, ty: WidgetType },
    Foreach { name: String, row_fields: Vec<String> },
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

#[derive(Debug, Clone)]
pub struct ParsedMarkdown {
    pub code_body: Vec<proc_macro2::TokenStream>,
    pub widget_fields: Vec<WidgetField>,
    pub references_state: bool,
    pub display_refs: Vec<String>,
    pub style_table: Option<proc_macro2::TokenStream>,
    pub used_widget_configs: std::collections::HashSet<String>,
}

// Note: Widget/TokenStream fields are omitted for now. Codegen integration will restore them.

pub fn strip_frontmatter(content: &str) -> (&str, &str) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return ("", content);
    }
    // Find the opening delimiter line
    let after_open = match trimmed.strip_prefix("---") {
        Some(rest) => {
            let rest = rest.trim_start_matches(' ');
            match rest.strip_prefix('\n') {
                Some(r) => r,
                None => match rest.strip_prefix("\r\n") {
                    Some(r) => r,
                    None => return ("", content),
                },
            }
        }
        None => return ("", content),
    };
    // Find closing ---
    if let Some(end_pos) = after_open.find("\n---") {
        let yaml = &after_open[..end_pos];
        let rest_start = end_pos + 4; // skip \n---
        let rest = &after_open[rest_start..];
        let rest = rest
            .strip_prefix('\n')
            .or_else(|| rest.strip_prefix("\r\n"))
            .unwrap_or(rest);
        (yaml, rest)
    } else {
        ("", content)
    }
}

pub fn parse_hex_color(s: &str) -> Result<[u8; 3], String> {
    let hex = s
        .strip_prefix('#')
        .ok_or_else(|| format!("Color must start with #: {s}"))?;
    if hex.len() != 6 {
        return Err(format!("Color must be #RRGGBB (6 hex digits): {s}"));
    }
    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| format!("Bad red in {s}: {e}"))?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| format!("Bad green in {s}: {e}"))?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| format!("Bad blue in {s}: {e}"))?;
    Ok([r, g, b])
}

pub fn detect_style_suffix(text: &str) -> (&str, Option<&str>) {
    let trimmed = text.trim_end();
    if let Some(pos) = trimmed.rfind("::") {
        let key = trimmed[pos + 2..].trim();
        if !key.is_empty()
            && key.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$')
        {
            let before = trimmed[..pos].trim_end();
            return (before, Some(key));
        }
    }
    (text, None)
}
