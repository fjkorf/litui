---
styles:
  title:
    bold: true
    color: "#FFD700"
    size: 28.0
  accent:
    color: "#FF6B00"
    bold: true
  success:
    bold: true
    color: "#00CC66"
  danger:
    bold: true
    color: "#FF4444"
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
widgets:
  vol:
    min: 0
    max: 100
    label: Volume
  name_cfg:
    hint: Enter your name
spacing:
  paragraph: 6.0
  heading_h2: 14.0
---

# Layout Demo ::title

This example adds horizontal layout, columns, and spacing configuration.

## Horizontal Buttons

::: horizontal

[button](Save) [button](Cancel) [button](Reset)

:::

## Two-Column Layout

::: columns 2

::: frame panel

### Controls

[slider](volume){vol}

[checkbox](muted)

[textedit](name){name_cfg}

:::

::: next

::: frame alert

### Status

**Name:** [display](name)

**Volume:** [display](volume)

> All systems go. ::success

:::

:::

## Styled Lists

- Ready ::success
- Blocked ::danger
- Pending ::muted

::muted(Custom spacing: paragraph 6px, heading_h2 14px — set via frontmatter.)
