# Multi-Page Apps

> Run it: `cargo run -p eframe_demo`

Build a tabbed application from multiple markdown files. Each file becomes a page with its own
content and widgets. State is shared across pages.

## The macro

```rust,ignore
define_markdown_app! {
    parent: "content/_app.md",
    "content/about.md",
    "content/form.md",
    "content/monitor.md",
}
```

This generates everything you need: a `Page` enum, per-page render functions, an `AppState`
struct, and an `MdApp` struct that handles navigation and dispatch.

## Parent frontmatter

The `parent:` file holds shared styles and widget configs inherited by all pages. It must NOT
have a `page:` section or stateful widgets.

`content/_app.md`:

```markdown
---
styles:
  danger:
    color: "#FF4444"
    bold: true
  success:
    color: "#44BB44"
  accent:
    color: "#FF6B00"
    bold: true
widgets:
  vol:
    min: 0
    max: 100
    label: "Volume"
---
```

Any markdown body in the parent generates a `render_shared(ui)` function -- useful for
headers or footers that appear on every page.

## Per-page frontmatter

Each page file needs a `page:` section. Exactly one page must have `default: true`.

`content/about.md`:

```markdown
---
page:
  name: About
  label: About
  default: true
---

# About

Welcome to the app. This page has no widgets -- just text.
```

`content/form.md`:

```markdown
---
page:
  name: Form
  label: Settings
---

# Settings

[slider](volume){vol}

[checkbox](muted)

[textedit](username){user_hint}
```

`content/monitor.md`:

```markdown
---
page:
  name: Monitor
  label: Monitor
widgets:
  vol_fmt:
    format: "{:.0}"
---

# Live State

| Field    | Value                    |
|----------|--------------------------|
| Volume   | [display](volume){vol_fmt} |
| Muted    | [display](muted)         |
| Username | [display](username)      |
```

## What gets generated

The macro produces these types and functions:

**`Page` enum:**

```rust,ignore
#[derive(Clone, Copy, PartialEq, Eq)]
enum Page { About, Form, Monitor }

impl Page {
    const ALL: &[Page] = &[Page::About, Page::Form, Page::Monitor];
    fn label(&self) -> &str { /* "About", "Settings", "Monitor" */ }
}

impl Default for Page {
    fn default() -> Self { Page::About } // the one with default: true
}
```

**`AppState` struct** -- all widget fields across all pages, merged flat:

```rust,ignore
struct AppState {
    volume: f64,
    muted: bool,
    username: String,
}
```

**Per-page render functions:**

```rust,ignore
fn render_about(ui: &mut egui::Ui);                          // no widgets
fn render_form(ui: &mut egui::Ui, state: &mut AppState);     // mutable widgets
fn render_monitor(ui: &mut egui::Ui, state: &AppState);      // display only (read-only)
```

**`MdApp` struct:**

```rust,ignore
struct MdApp {
    page: Page,
    state: AppState,
}

impl MdApp {
    fn show_nav(&mut self, ui: &mut egui::Ui);   // tab bar
    fn show_page(&mut self, ui: &mut egui::Ui);   // dispatches to render_*
}
```

![Multi-page app](img/multi_page_nav.png)

## Wiring into eframe

```rust,ignore
use eframe::egui;
use litui::*;

define_markdown_app! {
    parent: "content/_app.md",
    "content/about.md",
    "content/form.md",
    "content/monitor.md",
}

fn main() -> eframe::Result {
    eframe::run_native(
        "My App",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(MdApp::default()))),
    )
}

impl eframe::App for MdApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("nav").show(ctx, |ui| {
            self.show_nav(ui);
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.show_page(ui);
            });
        });
    }
}
```

That's it. `show_nav()` renders selectable tabs, `show_page()` dispatches to the right
render function based on the current page.

## Cross-page state

The killer feature. The Form page writes `state.volume` via a slider. The Monitor page
reads it with `[display](volume)`. Both reference the same `AppState` field. Change the
slider on the Form page, switch to Monitor, see the updated value.

Display widgets are read-only. If no input widget declares the field, display **self-declares**
it as `String` on `AppState`. This enables display-only pages where all data comes from code:

```text
| Stat | Value |
|------|-------|
| **Name** | [display](monster_name) |
| **HP** | [display](hp) |
```

This generates `monster_name: String` and `hp: String` on `AppState`. Populate them from
an ECS system or your app code. Pages with only display fields get `&AppState` (read-only).

If an input widget on another page already declares the field, the input widget's type wins.

## Shared fields across pages

Multiple pages can declare the same widget field — they control the same value in `AppState`.
For example, both a "Settings" page and a "Quick Controls" page could have:

```text
[slider](volume){vol}
```

Both sliders read and write the same `state.volume: f64`. The types must match — a
`[slider](foo)` on one page and a `[checkbox](foo)` on another would be a compile error
because `f64` and `bool` conflict.

## Display widgets in tables

Tables are ideal for monitoring dashboards. Each cell can hold a `[display]` widget:

```markdown
| Metric   | Value                        |
|----------|------------------------------|
| Volume   | [display](volume){vol_fmt}   |
| Muted    | [display](muted)             |
| Username | [display](username)          |
```

## bevy_ecs integration

For apps that need business logic beyond what markdown can express, store `Page` and
`AppState` as ECS Resources:

```rust,ignore
use bevy_ecs::prelude::*;

impl Resource for Page {}
impl Resource for AppState {}

fn auto_unmute(mut state: ResMut<AppState>) {
    if state.volume > 80.0 {
        state.muted = false;
    }
}
```

The ECS World owns the state. Each frame, run your schedule, then pass state into the
render functions. Systems run every frame regardless of which page is active -- they
enforce invariants across the entire app state.

See `examples/demo_content/` and `examples/eframe_demo/` for a complete working example.

## Container directives

By default, all pages render into whatever `Ui` you provide. With `panel:` in the page frontmatter, litui creates the container automatically.

### Frontmatter syntax

```yaml
page:
  name: Stats
  label: Stats
  panel: right
  width: 180
```

Panel types: `left`, `right`, `top`, `bottom`, `window`. Omit `panel:` for central panel (default).

### show_all(ctx)

When any page has a `panel:` directive, `MdApp` gains a `show_all()` method:

```rust,ignore
// Instead of manual panel setup:
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.md.show_all(ctx); // handles all containers automatically
    }
}
```

`show_all()` renders:
1. Side panels (always visible, regardless of current page)
2. Top/bottom panels (always visible)
3. Navigation bar (auto-generated top panel)
4. Central panel (dispatches the current non-container page)
5. Windows (shown only when current page matches)

### Behavior

- **Side panels persist** — a stat bar on `panel: right` stays visible when you switch pages
- **Windows appear on demand** — an inventory on `panel: window` opens when you navigate to it
- **Central pages** work as before — `show_page(ui)` still works for manual control
- **Non-breaking** — `show_all(ctx)` is a new method; `show_page(ui)` is unchanged

### Example

```yaml
# stats.md — always-visible side panel
page:
  name: Stats
  label: Stats
  panel: right
  width: 180

# inventory.md — popup window
page:
  name: Inventory
  label: Items
  panel: window
  width: 350

# char_create.md — central content (default)
page:
  name: CharCreate
  label: Create
  default: true
```

App code: `self.md.show_all(ctx)` — one line handles everything.

## Next up

[Images](crate::_tutorial::_08_images) -- embed images in your markdown UI.
