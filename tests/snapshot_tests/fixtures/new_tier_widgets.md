---
widgets:
  notes_cfg:
    hint: Write your notes here...
    rows: 3
  pass_cfg:
    hint: Enter password
  angle_cfg:
    min: 0
    max: 360
    label: Angle
    suffix: "°"
  price_cfg:
    min: 0
    max: 1000
    prefix: "$"
    label: Price
  view_opts:
    options: ["Grid", "List", "Board"]
  toggle_cfg:
    label: Dark Mode
---

## Textarea

[textarea](notes){notes_cfg}

## Password

[password](secret){pass_cfg}

## Toggle Switch

[toggle](dark_mode){toggle_cfg}

[toggle](notifications)

## Selectable Labels

[selectable](view){view_opts}

## Slider with Suffix/Prefix

[slider](angle){angle_cfg}

[slider](price){price_cfg}
