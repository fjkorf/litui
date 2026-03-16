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
| `text-align: center` | `Layout::top_down(Align::Center)` | `::: center` directive |
| `text-align: right` | `Layout::top_down(Align::Max)` | `::: right` directive |
| `width: 100%` | `Layout::top_down_justified()` | `::: fill` directive |
| `justify-content: space-between` | Nested left/right layouts | `::: horizontal space-between` |

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

**Phase 5: Alignment and flex layout** — Block alignment directives and enhanced column/horizontal control.

- `::: center` — center-aligns block content via `Layout::top_down(Align::Center)`
- `::: right` — right-aligns block content via `Layout::top_down(Align::Max)`
- `::: fill` — stretches widgets to fill available width via `Layout::top_down_justified(Align::Min)`
- `::: horizontal center` — centered horizontal row
- `::: horizontal right` — right-aligned horizontal row
- `::: horizontal space-between` — left/right split with `::: next` separator; uses nested left/right layouts
- `::: columns 3:1:1` — weighted columns via `egui_extras::StripBuilder` with relative sizes
- GFM table column alignment (`:---`, `:---:`, `---:`) — parsed from pulldown-cmark, stored as `ColumnAlignment` in AST

### Alignment directive → egui mapping

| Directive | egui Layout | Effect |
|-----------|-------------|--------|
| `::: center` | `Layout::top_down(Align::Center)` | Content horizontally centered |
| `::: right` | `Layout::top_down(Align::Max)` | Content right-aligned |
| `::: fill` | `Layout::top_down_justified(Align::Min)` | Widgets stretch to fill width |
| `::: horizontal center` | `Layout::left_to_right(Align::Center).with_main_align(Align::Center)` | Items centered in row |
| `::: horizontal right` | `Layout::right_to_left(Align::Center)` | Items right-aligned in row |

### Weighted columns

`::: columns 3:1` parses to `weights: [3, 1]`, which generates:
```rust
egui_extras::StripBuilder::new(ui)
    .size(Size::relative(0.75))  // 3/(3+1)
    .size(Size::relative(0.25))  // 1/(3+1)
    .horizontal(|mut strip| { ... });
```

When weights are equal or absent, falls back to `ui.columns()` for simplicity.
