---
widgets:
  vol:
    min: 0
    max: 100
    label: Volume
  name_input:
    hint: Enter your name
  speed:
    speed: 0.5
---

## Audio

[slider](volume){vol}

[checkbox](muted)

## User Info

[textedit](username){name_input}

[dragvalue](count){speed}
