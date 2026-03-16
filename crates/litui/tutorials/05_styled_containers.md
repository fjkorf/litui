# Styled Containers

> Run it: `cargo run -p tut_05_containers`

This tutorial adds **styled blockquotes**, **styled lists**, and **frame containers** — applying frontmatter styles to structural elements.

## What's new

The `::key` suffix works on blockquotes and list items to color their structural elements (quote bars, bullet dots, numbers). Frame styles with box-model properties wrap content in `egui::Frame`.

## Styled blockquotes

```text
> Operation completed successfully. ::success

> Critical failure detected. ::danger

> Approaching rate limit. ::warning
```

The style's `color` is applied to both the vertical quote bar AND the text content.

## Styled lists

```text
- All systems operational ::success
- Deployment pending ::warning
- Build failed on main ::danger

1. Configure environment ::success
2. Review settings ::warning
3. Fix vulnerability ::danger
```

Bullet dots and numbers inherit the style color.

## Frame containers

Add frame properties to a style:

```yaml
styles:
  panel:
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

Wrap content with `::: frame stylename`:

```text
::: frame panel

Padded content with a dark background.

:::

::: frame alert

Alert box with a red border and colored text.

:::
```

## Frame properties

| Property | egui API | Effect |
|----------|----------|--------|
| `inner_margin` | `Frame::inner_margin()` | Padding inside the frame |
| `outer_margin` | `Frame::outer_margin()` | Margin outside the frame |
| `stroke` | `Frame::stroke()` width | Border width |
| `stroke_color` | `Frame::stroke()` color | Border color |
| `corner_radius` | `Frame::corner_radius()` | Rounded corners |
| `background` | `Frame::fill()` | Background color |

## Expert tip

Frame properties map directly to egui's box model. The CSS equivalents are: `inner_margin` = `padding`, `outer_margin` = `margin`, `stroke` = `border-width`, `corner_radius` = `border-radius`, `background` = `background-color`. The `::: frame` block directive uses the same `code_body` swap pattern as other block directives — it saves the current code body, accumulates content, then wraps everything in `egui::Frame::default().inner_margin(...).show(ui, |ui| { body })`.

## What we built

Colored blockquote bars, styled list bullets, and CSS-like frame containers — all from frontmatter styles.
