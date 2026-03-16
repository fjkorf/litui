# Multi-Page Apps

> Run it: `cargo run -p tut_08_multi_page`

This tutorial switches from `include_litui_ui!` to `define_litui_app!` — generating a full multi-page app with navigation, shared state, and container panels.

## What's new

`define_litui_app!` reads multiple `.md` files, each with a `page:` frontmatter section. It generates a `Page` enum, shared `AppState`, per-page render functions, and an `LituiApp` struct with navigation.

## The macro

```rust,ignore
mod pages {
    use egui;
    use litui::*;

    define_litui_app! {
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

When any page has `panel:`, `LituiApp` gains `show_all(&egui::Context)`:

```rust,ignore
self.md.show_all(ctx);  // one line renders everything
```

Side panels are always visible. Central pages switch via navigation. Windows appear on demand.

## Panel/window visibility control

Add `open:` to give any panel or window state-driven visibility:

```yaml
page:
  name: Log
  panel: window
  open: show_log
```

This auto-declares `show_log: bool` on `AppState` (default `false`). For windows, this adds an X close button. For side/top/bottom panels, the panel is hidden when the bool is `false`. Set `state.show_log = true` to open it from code.

`open:` works on all panel types: `left`, `right`, `top`, `bottom`, and `window`.

## Navigation control

By default, only central pages (no `panel:`) appear in `show_nav()`. Panel and window pages are excluded. Override with `navigable:`:

```yaml
page:
  name: Stats
  panel: right
  navigable: true   # force into nav bar
```

The generated `Page` enum has both `ALL` (every page) and `NAV_PAGES` (only navigable pages).

Configure the nav bar itself in the parent `_app.md`:

```yaml
nav:
  position: top      # "top" | "bottom" | "none"
  show_all: false    # if true, include panels/windows in nav
```

`position: "none"` disables the auto nav panel — call `show_nav(ui)` manually for custom placement.

## Expert tip

`define_litui_app!` calls `load_and_parse_md()` for each file, merging parent frontmatter into each child. The codegen generates: a `Page` enum with `ALL` and `NAV_PAGES` constants and `label()` method, a flat `AppState` struct merging all widget fields across pages (two pages can share a field if types match), per-page `render_*()` functions with the correct signature (`&mut AppState` for mutable widgets, `&AppState` for display-only, or just `&mut Ui` for stateless), and `LituiApp` with `show_nav()`, `show_page()`, and optionally `show_all()`.

## What we built

A multi-page app with navigation, shared state, parent inheritance, and container panels — from markdown files.
