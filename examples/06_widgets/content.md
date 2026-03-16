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
  muted:
    italic: true
    color: "#888888"
    weak: true
  panel:
    inner_margin: 8
    background: "#1A1A2E"
    corner_radius: 4
widgets:
  vol:
    min: 0
    max: 100
    label: Volume
  name_cfg:
    hint: Enter your name
---

# Widgets Demo ::title

This example adds interactive widgets that generate state automatically.

## Controls

[slider](volume){vol}

[checkbox](muted)

[textedit](name){name_cfg}

## Actions

[button](Submit)

[button](Reset)

## Display

::: frame panel

**Name:** [display](name)

**Volume:** [display](volume)

:::

::muted(Widgets auto-declare fields on the generated state struct.)
