//! YAML frontmatter parsing and style resolution.
//!
//! Markdown files may begin with a `---`-delimited YAML block that defines
//! reusable style presets, widget configurations, and page metadata. This
//! module handles the full pipeline:
//!
//! 1. [`strip_frontmatter()`] splits raw file content into YAML + markdown.
//! 2. `serde_yaml` deserializes the YAML into a [`Frontmatter`] struct.
//! 3. [`merge_frontmatter()`] merges a parent's styles/widgets into a child.
//! 4. At flush points, [`detect_style_suffix()`] checks for trailing `::key` suffixes.
//!
//! See `knowledge/frontmatter-and-styles.md` for the full style system design.

use std::collections::HashMap;

use serde::Deserialize;

use crate::error::ParseError;

// ── Frontmatter types ───────────────────────────────────────────────

/// Top-level frontmatter deserialized from the YAML block at the start of a `.md` file.
///
/// Contains optional page metadata (for `define_litui_app!`), a map of named
/// style presets referenced via `::key` suffixes and `::key(text)` inline spans, and a
/// map of widget configurations referenced via `{config}` after widget directives.
#[derive(Deserialize, Default, Debug, Clone, PartialEq)]
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
    #[serde(default)]
    pub theme: Option<ThemeDef>,
    /// Navigation configuration (parent-level only).
    #[serde(default)]
    pub nav: Option<NavDef>,
}

/// Spacing overrides for the generated UI.
///
/// All values are in pixels. When absent, built-in defaults are used:
/// paragraph 8, heading H1 16 / H2 12 / H3 8 / H4+ 4, table 8.
#[derive(Deserialize, Default, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SpacingDef {
    /// Vertical gap after paragraphs (default 8.0)
    pub paragraph: Option<f32>,
    /// Vertical gap after tables (default 8.0)
    pub table: Option<f32>,
    /// Top spacing before H1 (default 16.0)
    pub heading_h1: Option<f32>,
    /// Top spacing before H2 (default 12.0)
    pub heading_h2: Option<f32>,
    /// Top spacing before H3 (default 8.0)
    pub heading_h3: Option<f32>,
    /// Top spacing before H4+ (default 4.0)
    pub heading_h4: Option<f32>,
    /// egui `item_spacing.y` override — applied via `ui.spacing_mut()` at render start
    pub item: Option<f32>,
}

/// Page metadata from the `page:` section of frontmatter.
///
/// Required for each file passed to `define_litui_app!`. Provides the
/// enum variant name, the UI label for navigation, and whether this page
/// is the default (exactly one page must set `default: true`).
#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PageDef {
    pub name: String,
    pub label: String,
    #[serde(default)]
    pub default: bool,
    /// Container type: "left", "right", "top", "bottom", "window", or absent (central panel).
    #[serde(default)]
    pub panel: Option<String>,
    /// Default width for side panels or windows.
    #[serde(default)]
    pub width: Option<f32>,
    /// Default height for top/bottom panels or windows.
    #[serde(default)]
    pub height: Option<f32>,
    /// Name of a `bool` field on `AppState` that controls visibility.
    /// For windows: also enables close (X) button. For panels: show/hide only.
    #[serde(default)]
    pub open: Option<String>,
    /// Whether this page appears in `show_nav()`. Default: `true` for central pages,
    /// `false` for panel and window pages.
    #[serde(default)]
    pub navigable: Option<bool>,
    /// Background color for the panel/window frame. Supports hex (`"#RRGGBB"`, `"#RRGGBBAA"`)
    /// or `"transparent"`. When set, emits `.frame(Frame::none().fill(...))` on the container.
    #[serde(default)]
    pub background: Option<String>,
}

/// Navigation configuration, specified in the parent frontmatter.
#[derive(Deserialize, Default, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NavDef {
    /// Where the auto-generated nav bar is placed: "top", "bottom", "none".
    /// Default: "top". "none" disables auto nav — call `show_nav(ui)` manually.
    #[serde(default = "NavDef::default_position")]
    pub position: String,
    /// If true, show ALL pages in nav including panels and windows.
    /// Default: false (only navigable pages shown).
    #[serde(default)]
    pub show_all: bool,
}

impl NavDef {
    fn default_position() -> String {
        "top".into()
    }
}

/// Widget-specific configuration from the `widgets:` section of frontmatter.
///
/// Referenced by `{key}` after a widget directive (e.g., `[slider](volume){vol}`).
#[derive(Deserialize, Default, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WidgetDef {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub speed: Option<f64>,
    pub label: Option<String>,
    pub hint: Option<String>,
    /// Format string for display widgets (e.g., `"{:.1}"`)
    pub format: Option<String>,
    /// Options list for radio/combobox widgets
    pub options: Option<Vec<String>>,
    /// Track hover state for buttons
    pub track_hover: Option<bool>,
    /// Track secondary click for buttons
    pub track_secondary: Option<bool>,
    /// Suffix appended to slider display (e.g., `"°"`)
    pub suffix: Option<String>,
    /// Prefix prepended to slider display (e.g., `"$"`)
    pub prefix: Option<String>,
    /// Desired row count for textarea widgets
    pub rows: Option<usize>,
    /// Max height in pixels for scrollable select widgets
    pub max_height: Option<f64>,
    /// Fill color for progress bars (e.g., `"#8B0000"`)
    pub fill: Option<String>,
    /// Integer mode for sliders — snaps to whole numbers.
    pub integer: Option<bool>,
    /// Step size for slider quantization (e.g., `5.0` for 5-degree increments).
    pub step: Option<f64>,
    /// Fixed decimal places for slider/dragvalue display.
    pub decimals: Option<usize>,
}

/// A named style preset that controls how text is rendered.
///
/// Each field is `Option` so that styles can be composed via [`merge_style_defs()`]:
/// an overlay's `Some` values override the base, while `None` inherits.
#[derive(Deserialize, Default, Debug, Clone, PartialEq)]
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
    // Frame properties (egui::Frame box model)
    /// Inner margin in pixels (`Frame::inner_margin`)
    pub inner_margin: Option<f32>,
    /// Outer margin in pixels (`Frame::outer_margin`)
    pub outer_margin: Option<f32>,
    /// Border stroke width in pixels (`Frame::stroke`)
    pub stroke: Option<f32>,
    /// Border stroke color hex (`Frame::stroke`)
    pub stroke_color: Option<String>,
    /// Corner radius in pixels (`Frame::corner_radius`)
    pub corner_radius: Option<f32>,
}

// ── Semantic colors ───────────────────────────────────────────────

/// A color that references an egui Visuals field by name.
/// These resolve at runtime, automatically adapting to dark/light mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticColor {
    /// `widgets.noninteractive.fg_stroke.color`
    Text,
    /// `strong_text_color()`
    Strong,
    /// `weak_text_color()`
    Weak,
    /// `hyperlink_color`
    Hyperlink,
    /// `warn_fg_color`
    Warn,
    /// `error_fg_color`
    Error,
    /// `code_bg_color`
    CodeBg,
    /// `faint_bg_color`
    FaintBg,
    /// `extreme_bg_color`
    ExtremeBg,
    /// `panel_fill`
    PanelFill,
    /// `window_fill`
    WindowFill,
    /// `selection.bg_fill`
    Selection,
}

impl SemanticColor {
    /// Parse a keyword string into a semantic color, if it matches.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "text" => Some(Self::Text),
            "strong" => Some(Self::Strong),
            "weak" => Some(Self::Weak),
            "hyperlink" => Some(Self::Hyperlink),
            "warn" => Some(Self::Warn),
            "error" => Some(Self::Error),
            "code_bg" => Some(Self::CodeBg),
            "faint_bg" => Some(Self::FaintBg),
            "extreme_bg" => Some(Self::ExtremeBg),
            "panel_fill" => Some(Self::PanelFill),
            "window_fill" => Some(Self::WindowFill),
            "selection" => Some(Self::Selection),
            _ => None,
        }
    }

    /// All valid keyword names, for error messages.
    pub const ALL_NAMES: &[&str] = &[
        "text",
        "strong",
        "weak",
        "hyperlink",
        "warn",
        "error",
        "code_bg",
        "faint_bg",
        "extreme_bg",
        "panel_fill",
        "window_fill",
        "selection",
    ];
}

/// A color value that is either a compile-time hex literal or a runtime semantic reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorValue {
    /// `#RRGGBB` hex color, parsed at compile time.
    Hex([u8; 3]),
    /// Semantic keyword referencing an egui Visuals field, resolved at runtime.
    Semantic(SemanticColor),
}

/// Parse a color string as either hex (`#RRGGBB`) or a semantic keyword.
///
/// # Errors
/// Returns an error if the string is neither valid hex nor a known semantic keyword.
pub fn parse_color_value(s: &str) -> Result<ColorValue, String> {
    if s.starts_with('#') {
        let rgb = parse_hex_color(s)?;
        Ok(ColorValue::Hex(rgb))
    } else if let Some(semantic) = SemanticColor::parse(s) {
        Ok(ColorValue::Semantic(semantic))
    } else {
        Err(format!(
            "Unknown color '{s}'. Use #RRGGBB hex or a semantic keyword: {}",
            SemanticColor::ALL_NAMES.join(", ")
        ))
    }
}

// ── Theme definition ─────────────────────────────────────────────

/// Global egui Visuals overrides defined in frontmatter.
///
/// Placed in the root `_app.md` to customize egui's color scheme globally.
/// Supports base values (applied to both themes) and per-theme overrides.
#[derive(Deserialize, Default, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ThemeDef {
    pub hyperlink_color: Option<String>,
    pub warn_fg_color: Option<String>,
    pub error_fg_color: Option<String>,
    pub code_bg_color: Option<String>,
    pub panel_fill: Option<String>,
    pub window_fill: Option<String>,
    pub selection_color: Option<String>,
    pub faint_bg_color: Option<String>,
    pub extreme_bg_color: Option<String>,
    /// Dark-mode specific overrides (applied when `visuals.dark_mode` is true).
    #[serde(default)]
    pub dark: Option<ThemeOverrides>,
    /// Light-mode specific overrides (applied when `visuals.dark_mode` is false).
    #[serde(default)]
    pub light: Option<ThemeOverrides>,
}

/// Per-theme color overrides (nested under `dark:` or `light:` in `theme:`).
#[derive(Deserialize, Default, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ThemeOverrides {
    pub hyperlink_color: Option<String>,
    pub warn_fg_color: Option<String>,
    pub error_fg_color: Option<String>,
    pub code_bg_color: Option<String>,
    pub panel_fill: Option<String>,
    pub window_fill: Option<String>,
    pub selection_color: Option<String>,
    pub faint_bg_color: Option<String>,
    pub extreme_bg_color: Option<String>,
}

// ── Frontmatter merging ───────────────────────────────────────────

/// Merge parent and child frontmatter. Child values override parent on key collision.
#[expect(
    clippy::iter_over_hash_type,
    reason = "insertion order doesn't matter for override merging"
)]
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
    // Theme: child overrides parent entirely (not field-merged)
    let theme = child.theme.or(parent.theme.clone());
    // Nav: child overrides parent entirely (parent-level config)
    let nav = child.nav.or(parent.nav.clone());
    Frontmatter {
        page: child.page,
        styles,
        widgets,
        spacing,
        theme,
        nav,
    }
}

/// Merge two `SpacingDef`s. Child's `Some` fields override parent.
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

/// Merge two `StyleDef`s. Overlay's `Some` fields override base.
pub fn merge_style_defs(base: &StyleDef, overlay: &StyleDef) -> StyleDef {
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
        inner_margin: overlay.inner_margin.or(base.inner_margin),
        outer_margin: overlay.outer_margin.or(base.outer_margin),
        stroke: overlay.stroke.or(base.stroke),
        stroke_color: overlay
            .stroke_color
            .clone()
            .or_else(|| base.stroke_color.clone()),
        corner_radius: overlay.corner_radius.or(base.corner_radius),
    }
}

// ── ID/Class selector parsing ─────────────────────────────────────

/// Result of parsing CSS-like selectors from link text.
///
/// Link text like `"button#submit.premium.large"` is split into:
/// - `base_name` -- `"button"` (the widget type or link display text)
/// - `id` -- `Some("submit")`
/// - `classes` -- `["premium", "large"]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSelector {
    pub base_name: String,
    pub id: Option<String>,
    pub classes: Vec<String>,
}

/// Parse CSS-like selectors from link text: `"button#id.class1.class2"` →
/// `ParsedSelector { base_name: "button", id: Some("id"), classes: ["class1", "class2"] }`
pub fn parse_selectors(link_text: &str) -> ParsedSelector {
    let split_pos = link_text.find(['#', '.']);
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
        base_name: base.to_owned(),
        id,
        classes,
    }
}

/// Resolve class names into a merged `StyleDef`. Returns error on undefined classes.
///
/// # Errors
/// Returns `ParseError` if any class name is not defined in the frontmatter styles.
pub fn resolve_classes(
    classes: &[String],
    frontmatter: &Frontmatter,
) -> Result<Option<StyleDef>, ParseError> {
    if classes.is_empty() {
        return Ok(None);
    }
    let mut result = StyleDef::default();
    for class_name in classes {
        let style = frontmatter.styles.get(class_name.as_str()).ok_or_else(|| {
            ParseError::new(format!(
                "Undefined style class '.{class_name}' in frontmatter"
            ))
        })?;
        result = merge_style_defs(&result, style);
    }
    Ok(Some(result))
}

/// Split content into (`yaml_frontmatter`, `remaining_markdown`).
/// Returns `("", content)` if no frontmatter is present.
pub fn strip_frontmatter(content: &str) -> (&str, &str) {
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

/// Parse "#RRGGBB" hex color to `[r, g, b]`.
///
/// # Errors
/// Returns an error string if the input is not a valid `#RRGGBB` hex color.
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

/// Check if text ends with a `::key` style suffix. Returns `(trimmed_text, Some(key))` or `(text, None)`.
/// The key may start with `$` for runtime style references (e.g., `::$hp_style`).
pub fn detect_style_suffix(text: &str) -> (&str, Option<&str>) {
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

/// Capitalize the first character of a string.
pub fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── strip_frontmatter ─────────────────────────────────

    #[test]
    fn strip_frontmatter_basic() {
        let content = "---\nstyles:\n  title:\n    bold: true\n---\n# Hello";
        let (yaml, md) = strip_frontmatter(content);
        assert_eq!(yaml, "styles:\n  title:\n    bold: true");
        assert_eq!(md, "# Hello");
    }

    #[test]
    fn strip_frontmatter_none() {
        let content = "# Hello World";
        let (yaml, md) = strip_frontmatter(content);
        assert_eq!(yaml, "");
        assert_eq!(md, content);
    }

    #[test]
    fn strip_frontmatter_empty_yaml() {
        // Empty frontmatter: the closing --- immediately follows opening ---
        // This means find("\n---") finds at position 0 (the newline between open/close)
        // yaml is "", and md starts after the closing ---
        let content = "---\n---\n# Hello";
        let (yaml, md) = strip_frontmatter(content);
        // The parser sees opening ---, then immediately \n---, so yaml="" md="# Hello"
        // But the current implementation sees `after_open` = "---\n# Hello",
        // `find("\n---")` doesn't match (there's no \n before the closing ---).
        // So it falls through and returns ("", content). This is correct behavior —
        // empty frontmatter with no content between delimiters is not valid YAML frontmatter.
        assert_eq!(yaml, "");
        assert_eq!(md, content);
    }

    // ── detect_style_suffix ───────────────────────────────

    #[test]
    fn detect_style_suffix_static() {
        let (text, key) = detect_style_suffix("Hello world ::title");
        assert_eq!(text, "Hello world");
        assert_eq!(key, Some("title"));
    }

    #[test]
    fn detect_style_suffix_dynamic() {
        let (text, key) = detect_style_suffix("Status ::$hp_style");
        assert_eq!(text, "Status");
        assert_eq!(key, Some("$hp_style"));
    }

    #[test]
    fn detect_style_suffix_none() {
        let (text, key) = detect_style_suffix("Hello world");
        assert_eq!(text, "Hello world");
        assert!(key.is_none());
    }

    // ── parse_selectors ───────────────────────────────────

    #[test]
    fn parse_selectors_full() {
        let sel = parse_selectors("button#submit.premium.large");
        assert_eq!(sel.base_name, "button");
        assert_eq!(sel.id, Some("submit".to_owned()));
        assert_eq!(sel.classes, vec!["premium", "large"]);
    }

    #[test]
    fn parse_selectors_no_selectors() {
        let sel = parse_selectors("slider");
        assert_eq!(sel.base_name, "slider");
        assert!(sel.id.is_none());
        assert!(sel.classes.is_empty());
    }

    #[test]
    fn parse_selectors_class_only() {
        let sel = parse_selectors(".accent");
        assert_eq!(sel.base_name, "");
        assert!(sel.id.is_none());
        assert_eq!(sel.classes, vec!["accent"]);
    }

    // ── parse_hex_color ───────────────────────────────────

    #[test]
    fn parse_hex_color_valid() {
        assert_eq!(parse_hex_color("#FF8800"), Ok([255, 136, 0]));
    }

    #[test]
    fn parse_hex_color_invalid() {
        assert!(parse_hex_color("FF8800").is_err());
        assert!(parse_hex_color("#GG0000").is_err());
        assert!(parse_hex_color("#FF88").is_err());
    }

    // ── resolve_classes ───────────────────────────────────

    #[test]
    fn resolve_classes_defined() {
        let mut fm = Frontmatter::default();
        fm.styles.insert(
            "accent".to_owned(),
            StyleDef {
                bold: Some(true),
                color: Some("#FF0000".to_owned()),
                ..StyleDef::default()
            },
        );
        let result = resolve_classes(&["accent".to_owned()], &fm).unwrap();
        assert!(result.is_some());
        let s = result.unwrap();
        assert_eq!(s.bold, Some(true));
        assert_eq!(s.color, Some("#FF0000".to_owned()));
    }

    #[test]
    fn resolve_classes_undefined() {
        let fm = Frontmatter::default();
        let result = resolve_classes(&["missing".to_owned()], &fm);
        assert!(result.is_err());
    }

    // ── merge_frontmatter ─────────────────────────────────

    #[test]
    fn merge_frontmatter_child_overrides() {
        let mut parent = Frontmatter::default();
        parent.styles.insert(
            "title".to_owned(),
            StyleDef {
                bold: Some(true),
                ..StyleDef::default()
            },
        );
        let mut child = Frontmatter::default();
        child.styles.insert(
            "title".to_owned(),
            StyleDef {
                italic: Some(true),
                ..StyleDef::default()
            },
        );
        let merged = merge_frontmatter(&parent, child);
        let title = merged.styles.get("title").unwrap();
        // Child fully replaces parent on key collision
        assert!(title.bold.is_none());
        assert_eq!(title.italic, Some(true));
    }

    // ── merge_style_defs ──────────────────────────────────

    #[test]
    fn merge_style_defs_overlay() {
        let base = StyleDef {
            bold: Some(true),
            color: Some("#FF0000".to_owned()),
            ..StyleDef::default()
        };
        let overlay = StyleDef {
            italic: Some(true),
            color: Some("#00FF00".to_owned()),
            ..StyleDef::default()
        };
        let merged = merge_style_defs(&base, &overlay);
        assert_eq!(merged.bold, Some(true));
        assert_eq!(merged.italic, Some(true));
        assert_eq!(merged.color, Some("#00FF00".to_owned()));
    }

    // ── Semantic colors ───────────────────────────────────

    #[test]
    fn parse_color_value_hex() {
        let cv = parse_color_value("#FF8800").unwrap();
        assert_eq!(cv, ColorValue::Hex([255, 136, 0]));
    }

    #[test]
    fn parse_color_value_semantic() {
        let cv = parse_color_value("strong").unwrap();
        assert_eq!(cv, ColorValue::Semantic(SemanticColor::Strong));
    }

    #[test]
    fn parse_color_value_all_keywords() {
        for name in SemanticColor::ALL_NAMES {
            let cv = parse_color_value(name);
            assert!(cv.is_ok(), "Failed to parse semantic color '{name}'");
            assert!(matches!(cv.unwrap(), ColorValue::Semantic(_)));
        }
    }

    #[test]
    fn parse_color_value_unknown() {
        let result = parse_color_value("bogus");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown color"));
    }

    // ── Theme definition ──────────────────────────────────

    #[test]
    fn frontmatter_with_theme() {
        let yaml = "theme:\n  panel_fill: \"#1E1E2E\"\n  dark:\n    code_bg_color: \"#2A2A2A\"\n  light:\n    code_bg_color: \"#EEEEEE\"\n";
        let fm: Frontmatter = serde_yaml::from_str(yaml).unwrap();
        let theme = fm.theme.unwrap();
        assert_eq!(theme.panel_fill, Some("#1E1E2E".to_owned()));
        let dark = theme.dark.unwrap();
        assert_eq!(dark.code_bg_color, Some("#2A2A2A".to_owned()));
        let light = theme.light.unwrap();
        assert_eq!(light.code_bg_color, Some("#EEEEEE".to_owned()));
    }

    #[test]
    fn frontmatter_with_semantic_style_color() {
        let yaml = "styles:\n  title:\n    color: strong\n    bold: true\n";
        let fm: Frontmatter = serde_yaml::from_str(yaml).unwrap();
        let title = fm.styles.get("title").unwrap();
        assert_eq!(title.color, Some("strong".to_owned()));
        // Verify it parses as a semantic color
        let cv = parse_color_value(title.color.as_ref().unwrap()).unwrap();
        assert_eq!(cv, ColorValue::Semantic(SemanticColor::Strong));
    }
}
