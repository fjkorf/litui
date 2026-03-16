---
styles:
  title:
    bold: true
    color: "#4488FF"
    size: 28.0
  section:
    bold: true
    color: "#CCCCCC"
widgets:
  freq:
    min: 20
    max: 20000
  temp:
    min: -40
    max: 120
---

# 3rd-Party Widget Integration ::title

This example demonstrates how to use **third-party egui widget crates** with the markdown macro system.

## Built-in: Double Slider ::section

The `egui_double_slider` crate provides a range slider with two handles. It's integrated as a first-class widget directive:

[double_slider](frequency){freq}

[double_slider](temperature){temp}

## Standard Widgets ::section

Regular macro widgets work alongside 3rd-party ones:

[slider](volume){freq}

[checkbox](enabled)
