---
styles:
  primary:
    bold: true
    color: "#4488FF"
  danger:
    bold: true
    color: "#FF4444"
  success:
    bold: true
    color: "#00CC66"
  warning:
    bold: true
    color: "#FFAA00"
  large:
    size: 20.0
    bold: true
  muted:
    italic: true
    color: "#888888"
    weak: true
  accent:
    color: "#FF6B00"
    bold: true
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
---

# Selectors and Styled Containers

## Class Selectors

Styled buttons using `.class` syntax:

[button.primary](Submit)

[button.danger](Cancel)

[button.primary.large](Big_Blue_Button)

## Inline Styled Spans

::accent(This text is orange and bold)

::muted(This text is gray and subtle)

## Styled Blockquotes

> Operation completed successfully. ::success

> Critical failure detected. ::danger

> Approaching rate limit — slow down. ::warning

## Styled Lists

- All systems operational ::success
- Deployment pending review ::warning
- Build failed on main ::danger
- Scheduled for deprecation ::muted

1. Configure environment ::success
2. Review security settings ::warning
3. Fix critical vulnerability ::danger

## Frame Containers

::: frame frame

Padded content with background and border.

:::

::: frame alert

Alert box with red border and colored text.

:::
