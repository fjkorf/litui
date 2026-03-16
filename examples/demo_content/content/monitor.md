---
page:
  name: Monitor
  label: Monitor
widgets:
  vol_fmt:
    format: "{:.1}"
  bright_fmt:
    format: "{:.1}"
  count_fmt:
    format: "{:.1}"
---

# State Monitor

This page reads `AppState` from the ECS World to display live widget values. Switch to the **Form** page, adjust the sliders, then come back here to see updated values.

| Field | Value |
|-------|-------|
| Volume | [display](volume){vol_fmt} |
| Muted | [display](muted) |
| Brightness | [display](brightness){bright_fmt} |
| Username | [display](username) |
| Count | [display](count){count_fmt} |
| Quality | [display](quality) |
| Theme | [display](theme) |
| Submit clicks | [display](on_submit_count) |
| Submit hovered | [display](on_submit_hovered) |
| Submit secondary | [display](on_submit_secondary_count) |
| Reset clicks | [display](on_reset_count) |

The **auto-unmute** system runs every frame: when volume exceeds 80, muted is forced to *false*.
