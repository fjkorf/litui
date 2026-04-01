---
widgets:
  int_cfg: { min: 0, max: 10, integer: true, label: "Integer" }
  step_cfg: { min: 0, max: 360, step: 15.0, suffix: "°", label: "Stepped" }
  dec_cfg: { min: 0.0, max: 1.0, decimals: 3, label: "Precise" }
  drag_cfg: { min: 0.0, max: 100.0, speed: 0.5, suffix: "%", decimals: 1 }
---

## Numeric Config

[slider](int_val){int_cfg}
[slider](step_val){step_cfg}
[slider](dec_val){dec_cfg}
[dragvalue](drag_val){drag_cfg}
