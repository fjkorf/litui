# Layout and Spacing Design

## egui's CSS-equivalent primitives

| CSS | egui | API |
|-----|------|-----|
| `padding` | `Frame::inner_margin()` | `Frame::default().inner_margin(8.0).show(ui, \|ui\| { ... })` |
| `margin` | `Frame::outer_margin()` | Same pattern |
| `border` | `Frame::stroke()` | `Stroke::new(width, color)` |
| `border-radius` | `Frame::corner_radius()` | float |
| `background` | `Frame::fill()` | `Color32` |
| `gap` | `Spacing::item_spacing` | `ui.spacing_mut().item_spacing = vec2(x, y)` |
| `display: flex` | `Layout` | Direction + alignment + justify |
| `flex: 1` | `Size::remainder()` | egui_extras StripBuilder |

## Default spacing values

- Paragraph gap: 8px
- Table gap: 8px
- Before H1: 16px, H2: 12px, H3: 8px, H4+: 4px
- List indent: row_height / 2.0 per depth
- Quote bar spacing: row_height / 2.0

## Implemented features

**Phase 1: Hard-coded defaults** — Paragraph, heading, and table spacing with sensible defaults.

**Phase 2: StyleDef frame properties** — `inner_margin`, `outer_margin`, `stroke`, `stroke_color`, `corner_radius` on styles, rendered via `::: frame stylename` block directive.

**Phase 3: Spacing configuration** — Frontmatter `spacing:` section with `paragraph`, `table`, `heading_h1`–`heading_h4`, and `item` fields. Values resolve at compile time; `item` maps to `ui.spacing_mut().item_spacing.y` at render start.

**Phase 4: Layout directives** — `::: horizontal` wraps content in `ui.horizontal()` for side-by-side elements. `::: columns N` with `::: next` separators generates `ui.columns(N, ...)` with independent per-column Uis.
