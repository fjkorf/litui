# Third-Party Widgets

> Run it: `cargo run -p third_party_widgets`

litui knows about a fixed set of widgets (slider, checkbox, textedit, etc.). When you need
something the macro doesn't have, there are two patterns.

## Pattern A: Built-in directive

Some third-party widgets are integrated directly into the macro. You use them like any other
widget directive -- the macro emits code that references the external crate, and you add the
crate to your `Cargo.toml`.

### Double slider (egui_double_slider)

A range slider with two handles. Syntax:

```markdown
---
widgets:
  freq_range:
    min: 20
    max: 20000
---

## Frequency Range

[double_slider](frequency){freq_range}
```

The macro generates code calling `egui_double_slider::DoubleSlider::new(...)`. It creates
**two** state fields: `frequency_low: f64` and `frequency_high: f64`, both derived from the
field name you provide.

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
egui_double_slider = "0.3"
```

If the crate is missing from your dependencies, the Rust compiler tells you:

```text
error[E0433]: failed to resolve: use of undeclared crate or module `egui_double_slider`
```

That's your cue to add it.

### Widget config

The `{freq_range}` config uses the same `min`/`max` fields as a regular slider:

```yaml
widgets:
  freq_range:
    min: 20
    max: 20000
```

Both handles are clamped to this range. The low handle can't exceed the high handle and
vice versa.

## Pattern B: Manual widgets

For anything the macro doesn't know about, render it yourself alongside macro-generated
content. This is the escape hatch -- it works with any egui widget, any crate, any
complexity.

### The idea

Use `include_markdown_ui!` for the text and layout, then add your custom widgets in Rust
code before or after the macro-rendered content:

```rust,ignore
use eframe::egui;
use litui::*;

struct MyApp {
    tags: Vec<bool>,
    color: egui::Color32,
}

const TAGS: &[&str] = &["Rust", "egui", "litui", "gamedev"];

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let render = include_markdown_ui!("content.md");

        egui::CentralPanel::default().show(ctx, |ui| {
            // Macro-generated content
            render(ui);

            ui.separator();

            // Hand-coded tag selector
            ui.label("Tags:");
            ui.horizontal_wrapped(|ui| {
                for (i, &tag) in TAGS.iter().enumerate() {
                    ui.toggle_value(&mut self.tags[i], tag);
                }
            });

            // Hand-coded color picker
            ui.horizontal(|ui| {
                ui.label("Accent color:");
                ui.color_edit_button_srgba(&mut self.color);
            });
        });
    }
}
```

The macro handles your headings, styled text, tables, and simple widgets. Your Rust code
handles the rest. They share the same `Ui`, so layout flows naturally.

### Mixing with stateful markdown

If your markdown has widgets too, destructure the tuple:

```rust,ignore
let (render, mut state) = include_markdown_ui!("form.md");

egui::CentralPanel::default().show(ctx, |ui| {
    render(ui, &mut state);

    // Read macro widget state
    if state.volume > 80.0 {
        ui.colored_label(egui::Color32::RED, "Volume is very high!");
    }

    // Add manual widgets that interact with macro state
    if ui.button("Reset volume").clicked() {
        state.volume = 50.0;
    }
});
```

You have full access to the macro's generated state struct. Read it, write it, react to it.

## When to use which

**Pattern A** (built-in directive) when:
- litui already supports the widget natively
- The widget has simple state (numbers, bools, strings)
- You want zero Rust boilerplate

**Pattern B** (manual) when:
- The widget needs generic type parameters or closures
- The widget has complex state (vectors, custom structs, trait objects)
- The third-party crate targets a different egui version
- You need layout control the macro can't express
- The widget doesn't exist yet as a litui directive

Pattern B is always available. If you're unsure, start with Pattern B -- you can always
request a Pattern A integration later.

## Currently supported Pattern A widgets

| Directive | Crate | State fields |
|-----------|-------|-------------|
| `[double_slider](field){config}` | `egui_double_slider` | `field_low: f64`, `field_high: f64` |

More will be added as the ecosystem grows. See the
[widget directives knowledge file](../../knowledge/widget-directives.md)
for the latest list.

---

Next: [Bevy Integration](crate::_tutorial::_10_bevy_integration) — render litui content inside a full Bevy application.
