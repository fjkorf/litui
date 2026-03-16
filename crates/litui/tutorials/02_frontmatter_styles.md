# Frontmatter Styles

> Run it: `cargo run -p tut_02_styles`

This tutorial adds **YAML frontmatter** to control colors, sizes, and text decoration. Styles are resolved at compile time — zero runtime cost.

## What's new

A `---`-delimited YAML block at the top of the file defines named style presets. Apply them with `::key` suffixes on paragraphs, headings, and list items, or `::key(text)` for inline spans.

## The frontmatter

```yaml
---
styles:
  title:
    bold: true
    color: "#FFD700"
    size: 28.0
  accent:
    color: "#FF6B00"
    bold: true
  danger:
    bold: true
    color: "#FF4444"
  success:
    bold: true
    color: "#00CC66"
  muted:
    italic: true
    color: "#888888"
    weak: true
---
```

## Applying styles

```text
# Hello litui ::title

This paragraph has a success style. ::success

This paragraph warns of danger. ::danger

::accent(This inline span is orange and bold.)

::muted(This text is gray and subtle.)
```

- `::title` after a heading applies the `title` style (gold, bold, 28pt)
- `::success` after a paragraph applies green bold text
- `::accent(text)` wraps a specific span in the `accent` style

## Style properties

| Property | Type | Effect |
|----------|------|--------|
| `bold` | bool | Bold weight |
| `italic` | bool | Italic |
| `strikethrough` | bool | Strikethrough |
| `underline` | bool | Underline |
| `color` | `"#RRGGBB"` | Text color |
| `background` | `"#RRGGBB"` | Background highlight |
| `size` | float | Font size in points |
| `monospace` | bool | Monospace font |
| `weak` | bool | Dimmed text |

## Expert tip

The macro calls `detect_style_suffix()` at each flush point (end of paragraph, heading, list item). It finds the trailing `::key`, looks it up in the frontmatter `styles` map, and emits a `styled_label_rich()` call with all properties as compile-time literals: `styled_label_rich(ui, "text", true, false, false, false, Some([255, 215, 0]), None, Some(28.0), false, false)`. The hex color `"#FFD700"` becomes `[255, 215, 0]` at compile time — no parsing at runtime.

## What we built

Styled markdown with named color presets, all resolved at compile time.
