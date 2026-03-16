# 3rd-Party Widget Integration

Two patterns for integrating external egui widget crates with the markdown macro system.

## Pattern A: Built-in Directive (simple state)

For widgets with simple state (numeric fields), add them directly to the macro. The macro emits code referencing the 3rd-party crate — the consumer provides the dependency.

**Example**: `egui_double_slider` is integrated as `[double_slider]`:

```markdown
[double_slider](frequency){freq_range}
```

The macro generates `egui_double_slider::DoubleSlider::new(...)` in the output code. The consumer crate must have `egui_double_slider` in its Cargo.toml.

**When to use this pattern**:
- Widget state maps to simple Rust types (f64, bool, String)
- Widget API follows the `ui.add(Widget::new(&mut state))` convention
- No complex generics, closures, or runtime configuration

**Current built-in 3rd-party widgets**: `double_slider` (egui_double_slider)

## Pattern B: Manual Integration (complex state)

For widgets with complex state or that don't fit the directive model, render them in app code alongside macro-generated content.

```rust
struct App<S> {
    render: fn(&mut egui::Ui, &mut S),
    state: S,
    // Manual widget state
    selected_tags: Vec<bool>,
    color: egui::Color32,
}

impl<S> eframe::App for App<S> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Macro-generated content (Pattern A)
            (self.render)(ui, &mut self.state);

            ui.separator();

            // Hand-coded widgets (Pattern B)
            ui.horizontal_wrapped(|ui| {
                for (i, &tag) in TAGS.iter().enumerate() {
                    ui.toggle_value(&mut self.selected_tags[i], tag);
                }
            });
            ui.color_edit_button_srgba(&mut self.color);
        });
    }
}
```

**When to use this pattern**:
- Widget needs generic type parameters or closures
- Widget has complex dependencies (notification systems, etc.)
- Widget state doesn't map to simple struct fields
- 3rd-party crate targets an older egui version (semver incompatibility)

**Note on version compatibility**: 3rd-party crates must target the same egui major.minor version (0.33). Crates targeting older versions (e.g., egui-multiselect targets eframe ^0.32) cause duplicate egui versions in the dependency tree, leading to type mismatches. Wait for the crate to update, or use `[patch.crates-io]` with a local/git source.

## Adding a New Built-in Widget

To add a new 3rd-party widget as a macro directive:

1. Add the widget name to `WIDGET_NAMES` in `crates/litui_macro/src/parse.rs`
2. Add a match arm in the widget code generation block (same file, `match link_text.as_str()`)
3. Push `WidgetField` entries for each state field the widget needs
4. Emit `quote!` code that constructs the widget using `ui.add()`
5. The consumer crate adds the 3rd-party dependency to their Cargo.toml
6. No changes needed to the macro crate's Cargo.toml (it only emits code, doesn't use the types)

## Compatibility

All egui dependencies now use crates.io (v0.33). 3rd-party crates targeting egui 0.33.x work without `[patch.crates-io]`.

## Example

See tutorial 10 (Advanced Widgets) for integration patterns. Pattern A (built-in directive) and Pattern B (manual alongside macro) are both covered.
