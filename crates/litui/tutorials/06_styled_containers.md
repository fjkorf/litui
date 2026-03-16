# Styled Containers

> Run it: `cargo run -p selectors_example`

Color blockquote bars and list bullets using the same `::key` syntax you already know from styled text.

## Styled blockquotes

When `::key` appears at the end of a blockquote paragraph, the style's `color` is applied to both
the vertical quote bar and the text content.

```markdown
---
styles:
  danger:
    color: "#FF4444"
    bold: true
  success:
    color: "#44BB44"
  warning:
    color: "#FFAA00"
  info:
    color: "#4488CC"
---

> All checks passed. ::success

> Warning: memory usage at 87%. ::warning

> CRITICAL: disk failure on node-3. ::danger

> Note: maintenance window starts at 02:00 UTC. ::info
```

The green bar, yellow bar, red bar, and blue bar each match their text color. Without `::key`,
blockquotes render with the default egui quote bar color.

![Styled blockquotes](img/styled_blockquote.png)

## Styled bullet lists

Same idea. Put `::key` at the end of a bullet item to color the bullet dot and text together.

```markdown
- All systems operational ::success
- Build queued ::warning
- Deploy failed ::danger
```

Green bullet, yellow bullet, red bullet. Each item is independently styled.

![Styled list](img/styled_list.png)

## Styled numbered lists

Works identically with numbered lists. The number gets the color instead of a bullet dot.

```markdown
1. Download complete ::success
2. Verification pending ::warning
3. Installation failed ::danger
```

## Practical example: status dashboard

Combine styled blockquotes and lists for a status-indicator layout:

```markdown
---
styles:
  ok:
    color: "#44BB44"
  warn:
    color: "#FFAA00"
    bold: true
  crit:
    color: "#FF4444"
    bold: true
  info:
    color: "#4488CC"
    italic: true
---

# System Status

> All services healthy. Last checked 12:34 UTC. ::ok

## Service Health

- API Gateway ::ok
- Auth Service ::ok
- Database Primary ::ok
- Database Replica ::warn
- Cache Cluster ::crit

## Recent Events

1. Cache node-2 restarted ::warn
2. Replica sync lag detected ::warn
3. Cache node-3 unreachable ::crit

> Next maintenance window: Saturday 02:00-04:00 UTC. ::info
```

This gives you colored indicators without any widget code. Pure markdown, pure styles.

## How it works

The style's `color` field does double duty:

1. It colors the **container element** -- the vertical quote bar, the bullet dot, or the list number
2. It applies the **full style** (color, bold, italic, etc.) to the text content

Under the hood, `emit_paragraph()` passes the resolved color to `emit_quote_bars_colored()`,
and `emit_list_item()` passes it to `emit_bullet_prefix_colored()` or
`emit_numbered_prefix_colored()`. When no `::key` is present, these fall back to the default
egui colors.

## Rules

- `::key` must be the last thing on the line (same as paragraphs and headings)
- The key must be defined in frontmatter `styles:` -- undefined keys fail the build
- Styles are inherited from parent frontmatter in `define_markdown_app!`
- Nested blockquotes: `::key` on the innermost quote colors all bars in that nesting chain
- Only `color` affects the container element -- `bold`, `italic`, etc. apply to text only

## Frame containers

Styles can include frame properties that wrap content in an `egui::Frame` with padding, borders, and rounded corners:

```yaml
styles:
  frame:
    inner_margin: 8
    background: "#1A1A2E"
    corner_radius: 4
  alert:
    inner_margin: 10
    stroke: 2
    stroke_color: "#FF4444"
    corner_radius: 6
    color: "#FF6666"
    bold: true
```

```text
::: frame frame

This content gets a padded dark frame.

:::

::: frame alert

This is an alert box with a red border.

:::
```

Frame properties map to egui's box model:
- `inner_margin` → `Frame::inner_margin()` (CSS padding)
- `outer_margin` → `Frame::outer_margin()` (CSS margin)
- `stroke` + `stroke_color` → `Frame::stroke()` (CSS border)
- `corner_radius` → `Frame::corner_radius()` (CSS border-radius)
- `background` → `Frame::fill()` (CSS background-color)

Text properties (bold, color, size) apply to content inside the frame.

## Next up

[Multi-Page Apps](crate::_tutorial::_07_multi_page_apps) -- build multi-page apps with shared state and navigation.
