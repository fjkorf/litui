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
//!    trailing `{key}` references, and [`style_def_to_label_tokens()`]
//!    emits the corresponding `styled_label_rich()` call.
//!
//! See `knowledge/frontmatter-and-styles.md` for the full style system design.

use std::collections::HashMap;

use quote::quote;
use serde::Deserialize;

// ── Frontmatter types ───────────────────────────────────────────────

/// Top-level frontmatter deserialized from the YAML block at the start of a `.md` file.
///
/// Contains optional page metadata (for `define_markdown_app!`), a map of named
/// style presets referenced via `::key` or `.class` in the markdown body, and a
/// map of widget configurations referenced via `{config}` after widget directives.
#[derive(Deserialize, Default)]
pub(crate) struct Frontmatter {
    #[serde(default)]
    pub(crate) page: Option<PageDef>,
    #[serde(default)]
    pub(crate) styles: HashMap<String, StyleDef>,
    #[serde(default)]
    pub(crate) widgets: HashMap<String, WidgetDef>,
}

/// Page metadata from the `page:` section of frontmatter.
///
/// Required for each file passed to `define_markdown_app!`. Provides the
/// enum variant name, the UI label for navigation, and whether this page
/// is the default (exactly one page must set `default: true`).
#[derive(Deserialize, Clone)]
pub(crate) struct PageDef {
    pub(crate) name: String,
    pub(crate) label: String,
    #[serde(default)]
    pub(crate) default: bool,
    /// Container type: "left", "right", "top", "bottom", "window", or absent (central panel).
    #[serde(default)]
    pub(crate) panel: Option<String>,
    /// Default width for side panels or windows.
    #[serde(default)]
    pub(crate) width: Option<f32>,
    /// Default height for top/bottom panels or windows.
    #[serde(default)]
    pub(crate) height: Option<f32>,
}

/// Widget-specific configuration from the `widgets:` section of frontmatter.
///
/// Referenced by `{key}` after a widget directive (e.g., `[slider](volume){vol}`).
/// Not all fields apply to every widget type:
/// - `min`/`max` -- slider, double_slider, dragvalue range bounds
/// - `speed` -- dragvalue drag sensitivity
/// - `label` -- slider/checkbox display label
/// - `hint` -- textedit placeholder hint text
/// - `format` -- display widget format string (e.g., `"{:.1}"`)
#[derive(Deserialize, Default, Clone)]
pub(crate) struct WidgetDef {
    pub(crate) min: Option<f64>,
    pub(crate) max: Option<f64>,
    pub(crate) speed: Option<f64>,
    pub(crate) label: Option<String>,
    pub(crate) hint: Option<String>,
    /// Format string for display widgets (e.g., `"{:.1}"`)
    pub(crate) format: Option<String>,
    /// Options list for radio/combobox widgets
    pub(crate) options: Option<Vec<String>>,
    /// Track hover state for buttons (generates `{name}_hovered: bool` field)
    pub(crate) track_hover: Option<bool>,
    /// Track secondary click for buttons (generates `{name}_secondary_count: u32` field)
    pub(crate) track_secondary: Option<bool>,
    /// Tooltip text shown on hover for any widget
    pub(crate) tooltip: Option<String>,
    /// Suffix appended to slider display (e.g., `"°"`)
    pub(crate) suffix: Option<String>,
    /// Prefix prepended to slider display (e.g., `"$"`)
    pub(crate) prefix: Option<String>,
    /// Desired row count for textarea widgets
    pub(crate) rows: Option<usize>,
    /// Max height in pixels for scrollable select widgets
    pub(crate) max_height: Option<f64>,
    /// Fill color for progress bars (e.g., `"#8B0000"`)
    pub(crate) fill: Option<String>,
}

/// A named style preset that controls how text is rendered.
///
/// Each field is `Option` so that styles can be composed via [`merge_style_defs()`]:
/// an overlay's `Some` values override the base, while `None` inherits.
///
/// - `bold`/`italic`/`strikethrough`/`underline` -- text decoration flags
/// - `color`/`background` -- hex color strings (`"#RRGGBB"`), parsed at compile time
/// - `size` -- font size in points (overrides heading defaults when applied to headings)
/// - `monospace` -- use monospace font instead of body font
/// - `weak` -- render with weak (dimmed) text color
#[derive(Deserialize, Default, Clone)]
pub(crate) struct StyleDef {
    pub(crate) bold: Option<bool>,
    pub(crate) italic: Option<bool>,
    pub(crate) strikethrough: Option<bool>,
    pub(crate) underline: Option<bool>,
    pub(crate) color: Option<String>,
    pub(crate) background: Option<String>,
    pub(crate) size: Option<f32>,
    pub(crate) monospace: Option<bool>,
    pub(crate) weak: Option<bool>,
}

// ── Frontmatter merging ───────────────────────────────────────────

/// Merge parent and child frontmatter. Child values override parent on key collision.
pub(crate) fn merge_frontmatter(parent: &Frontmatter, child: Frontmatter) -> Frontmatter {
    let mut styles = parent.styles.clone();
    for (k, v) in child.styles {
        styles.insert(k, v);
    }
    let mut widgets = parent.widgets.clone();
    for (k, v) in child.widgets {
        widgets.insert(k, v);
    }
    Frontmatter {
        page: child.page,
        styles,
        widgets,
    }
}

/// Merge two StyleDefs. Overlay's `Some` fields override base.
pub(crate) fn merge_style_defs(base: &StyleDef, overlay: &StyleDef) -> StyleDef {
    StyleDef {
        bold: overlay.bold.or(base.bold),
        italic: overlay.italic.or(base.italic),
        strikethrough: overlay.strikethrough.or(base.strikethrough),
        underline: overlay.underline.or(base.underline),
        color: overlay.color.clone().or_else(|| base.color.clone()),
        background: overlay
            .background
            .clone()
            .or_else(|| base.background.clone()),
        size: overlay.size.or(base.size),
        monospace: overlay.monospace.or(base.monospace),
        weak: overlay.weak.or(base.weak),
    }
}

// ── ID/Class selector parsing ─────────────────────────────────────

/// Result of parsing CSS-like selectors from link text.
///
/// Link text like `"button#submit.premium.large"` is split into:
/// - `base_name` -- `"button"` (the widget type or link display text)
/// - `id` -- `Some("submit")` (used as `egui::Id` via `ui.push_id()`)
/// - `classes` -- `["premium", "large"]` (resolved against frontmatter styles,
///   composed left-to-right via [`merge_style_defs()`])
pub(crate) struct ParsedSelector {
    pub(crate) base_name: String,
    pub(crate) id: Option<String>,
    pub(crate) classes: Vec<String>,
}

/// Parse CSS-like selectors from link text: `"button#id.class1.class2"` →
/// `ParsedSelector { base_name: "button", id: Some("id"), classes: ["class1", "class2"] }`
pub(crate) fn parse_selectors(link_text: &str) -> ParsedSelector {
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

/// Resolve class names into a merged StyleDef. Panics at compile time on undefined classes.
pub(crate) fn resolve_classes(classes: &[String], frontmatter: &Frontmatter) -> Option<StyleDef> {
    if classes.is_empty() {
        return None;
    }
    let mut result = StyleDef::default();
    for class_name in classes {
        let style = frontmatter
            .styles
            .get(class_name.as_str())
            .unwrap_or_else(|| panic!("Undefined style class '.{class_name}' in frontmatter"));
        result = merge_style_defs(&result, style);
    }
    Some(result)
}

/// Split content into (yaml_frontmatter, remaining_markdown).
/// Returns ("", content) if no frontmatter is present.
pub(crate) fn strip_frontmatter(content: &str) -> (&str, &str) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return ("", content);
    }
    // Find the opening delimiter line
    let after_open = match trimmed.strip_prefix("---") {
        Some(rest) => {
            // Must be followed by newline (possibly with trailing whitespace)
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
        // Skip the rest of the closing --- line
        let rest = rest
            .strip_prefix('\n')
            .or_else(|| rest.strip_prefix("\r\n"))
            .unwrap_or(rest);
        (yaml, rest)
    } else {
        ("", content)
    }
}

/// Parse "#RRGGBB" hex color to [r, g, b].
pub(crate) fn parse_hex_color(s: &str) -> Result<[u8; 3], String> {
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

/// Check if text ends with a `::key` style suffix. Returns (trimmed_text, Some(key)) or (text, None).
/// The key may start with `$` for runtime style references (e.g., `::$hp_style`).
pub(crate) fn detect_style_suffix(text: &str) -> (&str, Option<&str>) {
    let trimmed = text.trim_end();
    if let Some(pos) = trimmed.rfind("::") {
        let key = trimmed[pos + 2..].trim();
        if !key.is_empty()
            && key
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '$')
        {
            let before = trimmed[..pos].trim_end();
            return (before, Some(key));
        }
    }
    (text, None)
}

/// Emit tokens for a `styled_label_rich()` call using a resolved `StyleDef`.
pub(crate) fn style_def_to_label_tokens(
    text: &str,
    style: &StyleDef,
    base_bold: bool,
    base_italic: bool,
    base_strikethrough: bool,
) -> proc_macro2::TokenStream {
    let bold = style.bold.unwrap_or(base_bold);
    let italic = style.italic.unwrap_or(base_italic);
    let strikethrough = style.strikethrough.unwrap_or(base_strikethrough);
    let underline = style.underline.unwrap_or(false);
    let monospace = style.monospace.unwrap_or(false);
    let weak = style.weak.unwrap_or(false);

    let color_tokens = match &style.color {
        Some(hex) => {
            let [r, g, b] = parse_hex_color(hex).expect("Invalid color in frontmatter");
            quote! { Some([#r, #g, #b]) }
        }
        None => quote! { None },
    };
    let bg_tokens = match &style.background {
        Some(hex) => {
            let [r, g, b] = parse_hex_color(hex).expect("Invalid background color in frontmatter");
            quote! { Some([#r, #g, #b]) }
        }
        None => quote! { None },
    };
    let size_tokens = match style.size {
        Some(s) => quote! { Some(#s) },
        None => quote! { None },
    };

    quote! {
        styled_label_rich(ui, #text, #bold, #italic, #strikethrough, #underline, #color_tokens, #bg_tokens, #size_tokens, #monospace, #weak);
    }
}
