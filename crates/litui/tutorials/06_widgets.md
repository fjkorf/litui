# Widgets

> Run it: `cargo run -p tut_06_widgets`

This tutorial adds **interactive widgets** — sliders, checkboxes, text inputs, buttons, and display fields. The macro auto-generates a state struct.

## What's new

Widget directives use markdown link syntax: `[widget_type](field_name)`. The macro detects the widget type, declares a field on the generated state struct, and emits the corresponding egui widget.

## Widget syntax

```text
[slider](volume){vol}

[checkbox](muted)

[textedit](name){name_cfg}

[button](Submit)

[display](name)
```

- `[slider](volume)` — creates `volume: f64` on state, renders `egui::Slider`
- `{vol}` — references a widget config from frontmatter for min/max/label
- `[checkbox](muted)` — creates `muted: bool`, renders checkbox
- `[textedit](name)` — creates `name: String`, renders single-line text input
- `[button](Submit)` — renders a button (stateless unless `{config}` added)
- `[display](name)` — reads `name` from state and renders as a label

## Widget configs

```yaml
widgets:
  vol:
    min: 0
    max: 100
    label: Volume
  name_cfg:
    hint: Enter your name
```

Attach a config with `{key}` after the widget link.

## State generation

With widgets, `include_litui_ui!` returns a tuple:

```rust,ignore
let (render, mut state) = include_litui_ui!("content.md");

// In your update loop:
render(ui, &mut state);

// Access state fields:
println!("Volume: {}", state.volume);
println!("Name: {}", state.name);
```

The generated `LituiFormState` struct has one `pub` field per widget, all with `Default`.

## Expert tip

Widget detection works by intercepting markdown links. When the parser sees `[slider](volume)`, it checks the link text against `WIDGET_NAMES` — a compile-time list of 18 recognized widget types. If matched, the URL becomes the field name and a lookahead checks the next text event for `{config}`. The link never becomes a hyperlink — it's consumed as a widget directive. This means any unrecognized `[text](url)` still renders as a normal clickable link.

## What we built

Interactive form widgets with auto-generated state, configured via YAML frontmatter.
