---
styles:
  title:
    bold: true
    color: "#FFD700"
    size: 28.0
  accent:
    color: "#FF6B00"
    bold: true
  danger:
    bold: true
    color: "#FF4444"
  success:
    bold: true
    color: "#00CC66"
  muted:
    italic: true
    color: "#888888"
    weak: true
---

# Tables Demo ::title

This example adds GFM tables to our styled markdown.

## Basic Table

| Name | Role | Status |
|------|------|--------|
| Alice | Engineer | Active |
| Bob | Designer | On leave |
| Carol | Manager | Active |

## Formatted Table

| Feature | Syntax | Notes |
|---------|--------|-------|
| **Bold** | `**text**` | Double asterisks |
| *Italic* | `*text*` | Single asterisks |
| `Code` | backticks | Inline code |
| [Link](https://egui.rs) | `[text](url)` | Clickable |

## Styled Text

::accent(Tables render as striped egui::Grid widgets.)

> Tables support inline formatting in cells. ::success

---

Tables, styles, and standard markdown — all compile-time.
