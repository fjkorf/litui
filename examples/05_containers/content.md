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
  warning:
    bold: true
    color: "#FFAA00"
  muted:
    italic: true
    color: "#888888"
    weak: true
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
---

# Styled Containers ::title

This example adds styled blockquotes, lists, and frame containers.

## Styled Blockquotes

> Operation completed successfully. ::success

> Critical failure detected. ::danger

> Approaching rate limit. ::warning

## Styled Lists

- All systems operational ::success
- Deployment pending ::warning
- Build failed on main ::danger
- Scheduled for deprecation ::muted

1. Configure environment ::success
2. Review settings ::warning
3. Fix vulnerability ::danger

## Frame Containers

::: frame panel

Padded content with a dark background and rounded corners.

:::

::: frame alert

Alert box with a red border and colored text.

:::

## Images

![Ferris the crab](https://rustacean.net/assets/rustacean-flat-noshadow.png)

## Tables

| Name | Role | Status |
|------|------|--------|
| Alice | Engineer | Active |
| Bob | Designer | On leave |

---

Containers, styles, tables, images, and markdown — all compile-time.
