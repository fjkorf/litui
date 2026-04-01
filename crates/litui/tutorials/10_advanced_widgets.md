# Advanced Widgets

> Run it: `cargo run -p tut_10_advanced`

This tutorial covers the **remaining widget types**, **CSS-like selectors**, and **advanced button tracking**.

## What's new

litui has 18 widget types total. Earlier tutorials covered slider, checkbox, textedit, button, and display. This tutorial adds the rest.

## Selection widgets

```text
[radio](size){size_opts}

[combobox](color){color_opts}

[selectable](view){view_opts}

[select](chosen_species){species_list}
```

- `[radio]` — horizontal radio buttons from `options` list (`usize` index)
- `[combobox]` — dropdown from `options` list (`usize` index)
- `[selectable]` — clickable label group from `options` list (`usize` index)
- `[select]` — scrollable list from a runtime `Vec<String>` field (`usize` index)

Config:

```yaml
widgets:
  size_opts: { options: ["Small", "Medium", "Large"] }
  species_list: { max_height: 120.0 }
```

## Input widgets

```text
[dragvalue](angle){angle_cfg}
[textarea](notes){notes_cfg}
[password](secret){secret_cfg}
```

- `[dragvalue]` — drag to change a number (`f64`, supports `min`/`max`/`speed`/`suffix`/`prefix`/`decimals`)
- `[textarea]` — multi-line text input (`String`, supports `rows`/`hint`)
- `[password]` — masked text input (`String`, supports `hint`)

## Numeric precision

Sliders and drag values support precision options:

```yaml
widgets:
  octaves: { min: 0, max: 6, integer: true, label: Octaves }
  rotation: { min: -180, max: 180, step: 5.0, suffix: "°" }
  smooth_k: { min: 0.0, max: 0.1, decimals: 3 }
  roughness: { min: 0.0, max: 1.0, speed: 0.01, decimals: 2 }
```

- `integer: true` — snap to whole numbers (slider only)
- `step: 5.0` — quantize to discrete steps (slider only)
- `decimals: 3` — fixed decimal places (slider and dragvalue)
- `min`/`max` on dragvalue — clamps the drag range
- `suffix`/`prefix` — works on both slider and dragvalue

## Display widgets

```text
[progress](hp_frac){hp_bar}
[spinner]()
[toggle](dark_mode)
[color](tint)
[log](messages){msg_cfg}
```

- `[progress]` — progress bar (`f64` 0.0–1.0, supports `fill` color)
- `[spinner]` — animated loading indicator (stateless)
- `[toggle]` — iOS-style toggle switch (`bool`)
- `[color]` — color picker (`[u8; 4]` RGBA)
- `[log]` — scrollable message list (`Vec<String>`, supports `max_height`)

## Styled buttons with selectors

Apply frontmatter styles to widgets with `.class` syntax:

```text
[button.primary](Submit){on_submit}
[button.danger](Cancel)
[button.primary.large](Big_Action)
```

Classes compose left-to-right: `.primary.large` merges both styles.

## Advanced button tracking

```yaml
widgets:
  on_submit:
    track_hover: true
    track_secondary: true
```

Generates extra fields: `on_submit_count: u32` (always), `on_submit_hovered: bool`, `on_submit_secondary_count: u32`.

## Theme switching

The `[toggle](dark_mode)` widget stores a `bool` on state. To apply it as an actual egui theme, call `ctx.set_visuals()` in your app's `update()` function:

```rust,ignore
ctx.set_visuals(if self.state.dark_mode {
    egui::Visuals::dark()
} else {
    egui::Visuals::light()
});
```

litui's runtime helpers read all colors from `ui.visuals()`, so headings, text, code blocks, quote bars, and list markers automatically adapt. Styles using semantic keywords (like `color: error`) also adapt. Only hex colors (like `color: "#FF0000"`) remain fixed.

## Global theme customization

Customize egui's Visuals from your root `_app.md` frontmatter with a `theme:` section:

```yaml
theme:
  hyperlink_color: "#00AAFF"
  dark:
    panel_fill: "#1E1E2E"
    code_bg_color: "#2A2A3E"
  light:
    panel_fill: "#F0F0F5"
    code_bg_color: "#E8E8F0"
```

Base values apply to both themes. `dark:` and `light:` sub-sections apply conditionally. This generates a setup function called automatically in `show_all()`.

Styles using semantic keywords like `background: panel_fill` will pick up these custom values — they reference the active Visuals at runtime.

Run the example and toggle Dark Mode to see the entire UI switch between dark and light.

## Expert tip

Third-party egui widgets can integrate via two patterns. **Pattern A:** Add a match arm to `WIDGET_NAMES` for directives like `[double_slider]` — the macro emits the widget call directly. **Pattern B:** Render macro-generated content alongside hand-coded widgets in the same `update()` function — the generated `render_*()` functions are just regular Rust functions you can call anywhere.

## What we built

A complete widget showcase with selection controls, input fields, display widgets, styled buttons, and advanced response tracking.
