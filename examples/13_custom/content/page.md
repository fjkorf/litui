---
page:
  name: Mixed
  label: Mixed Page
  default: true
---

# Custom Escape Hatch ::title

This page is mostly Markdown, but the box below is raw egui drawn by a
user-supplied closure stored on the generated `AppState`.

::: frame panel

[custom](demo_slot)

:::

Markdown continues after the custom widget, proving the slot composes inline
with the rest of the page.
