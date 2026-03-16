---
page:
  name: Form
  label: Form
styles:
  title:
    bold: true
    color: "#4488FF"
    size: 24.0
widgets:
  vol:
    min: 0
    max: 100
    label: Volume
  brightness:
    min: 0
    max: 100
    label: Brightness
  name_input:
    hint: Enter your name
  speed:
    speed: 0.5
  on_submit:
    track_hover: true
    track_secondary: true
  on_reset: {}
---

# Interactive Form ::title

This page demonstrates stateful widget directives. The macro generates an `AppState` struct with fields for each widget, and the app code can read the state to drive other UI.

## Audio ::section

[slider](volume){vol}

[checkbox](muted)

## Display ::section

[slider](brightness){brightness}

## User Info ::section

[textedit](username){name_input}

[dragvalue](count){speed}

## Preferences ::section

[radio](quality){quality_opts}

[combobox](theme){theme_opts}

[color](accent)

## Actions ::section

[button.success](Submit){on_submit}

[button.danger](Reset){on_reset}
