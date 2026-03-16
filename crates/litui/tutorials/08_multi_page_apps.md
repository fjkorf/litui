# Multi-Page Apps

> Run it: `cargo run -p tut_08_multi_page`

This tutorial switches from `include_markdown_ui!` to `define_markdown_app!` — generating a full multi-page app with navigation, shared state, and container panels.

## What's new

`define_markdown_app!` reads multiple `.md` files, each with a `page:` frontmatter section. It generates a `Page` enum, shared `AppState`, per-page render functions, and an `MdApp` struct with navigation.

## The macro

```rust,ignore
mod pages {
    use egui;
    use litui::*;

    define_markdown_app! {
        parent: "content/_app.md",
        "content/about.md",
        "content/form.md",
        "content/monitor.md",
    }
}
```

## Page frontmatter

Each page needs a `page:` section:

```yaml
---
page:
  name: About
  label: About
  default: true
---
```

- `name` — Rust enum variant (e.g., `Page::About`)
- `label` — display text in navigation
- `default: true` — the landing page (exactly one required)

## Parent frontmatter

`_app.md` defines shared styles and widget configs inherited by all pages:

```yaml
---
styles:
  title: { bold: true, color: "#FFD700", size: 28.0 }
  panel: { inner_margin: 8, background: "#1A1A2E", corner_radius: 4 }
widgets:
  vol: { min: 0, max: 100, label: Volume }
---
```

No `page:` section — it's a parent, not a page.

## Container panels

Add `panel:` to place a page in a persistent side panel:

```yaml
page:
  name: Monitor
  label: Monitor
  panel: right
  width: 200
```

Panel values: `left`, `right`, `top`, `bottom`, `window`. Omit for central panel (default).

When any page has `panel:`, `MdApp` gains `show_all(&egui::Context)`:

```rust,ignore
self.md.show_all(ctx);  // one line renders everything
```

Side panels are always visible. Central pages switch via navigation. Windows appear on demand.

## Window visibility control

Add `open:` to give a window its own close button:

```yaml
page:
  name: Log
  panel: window
  open: show_log
```

This auto-declares `show_log: bool` on `AppState`. Set `state.show_log = true` to open it from code.

## Expert tip

`define_markdown_app!` calls `load_and_parse_md()` for each file, merging parent frontmatter into each child. The codegen generates: a `Page` enum with `ALL` constant and `label()` method, a flat `AppState` struct merging all widget fields across pages (two pages can share a field if types match), per-page `render_*()` functions with the correct signature (`&mut AppState` for mutable widgets, `&AppState` for display-only, or just `&mut Ui` for stateless), and `MdApp` with `show_nav()`, `show_page()`, and optionally `show_all()`.

## What we built

A multi-page app with navigation, shared state, parent inheritance, and container panels — from markdown files.
