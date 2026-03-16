---
styles:
  title:
    bold: true
    color: "#4488FF"
    size: 28.0
  section:
    bold: true
    color: "#CCCCCC"
    size: 18.0
  action:
    bold: true
    color: "#00AA00"
  warning:
    bold: true
    color: "#FF4444"
  muted:
    italic: true
    color: "#888888"
widgets:
  vol:
    min: 0
    max: 100
    label: Volume
  name_input:
    hint: Enter your name
  speed:
    speed: 0.5
  quality_opts:
    options: ["Low", "Medium", "High", "Ultra"]
  output_opts:
    options: ["Speakers", "Headphones", "Bluetooth"]
    label: "Output Device"
  notes_cfg:
    hint: Write your notes here...
    rows: 3
  pass_cfg:
    hint: Enter API key
  dark_cfg:
    label: Dark Mode
  view_opts:
    options: ["Grid", "List", "Board"]
---

# Widget Demo ::title

Adjust the slider and watch the progress bar update in real time.

## Audio Controls ::section

[slider](volume){vol}

[checkbox](muted)

[combobox](output){output_opts}

## Preferences ::section

[radio](quality){quality_opts}

[color](accent_color)

## User Info ::section

[textedit](username){name_input}

[password](api_key){pass_cfg}

[textarea](notes){notes_cfg}

[dragvalue](count){speed}

## View ::section

[toggle](dark_mode){dark_cfg}

[selectable](view_mode){view_opts}

## Feature Status ::section

- Audio engine ready ::action
- Network module loading ::warning
- Debug mode active ::muted

## Actions ::section
