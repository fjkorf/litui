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
---

# Images Demo ::title

Standard markdown image syntax renders via `egui::Image`.

## Ferris

![Ferris the crab](https://rustacean.net/assets/rustacean-flat-noshadow.png)

## Images in Tables

| Mascot | Description |
|--------|-------------|
| ![Ferris](https://rustacean.net/assets/rustacean-flat-noshadow.png) | The Rust mascot |

::muted(Images require egui_extras::install_image_loaders&lpar;&rpar; in your app setup.)

---

Images, tables, styles, and markdown — all compile-time.
