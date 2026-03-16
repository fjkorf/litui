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

- `[dragvalue]` — drag to change a number (`f64`, supports `min`/`max`/`speed`)
- `[textarea]` — multi-line text input (`String`, supports `rows`/`hint`)
- `[password]` — masked text input (`String`, supports `hint`)

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

## Expert tip

Third-party egui widgets can integrate via two patterns. **Pattern A:** Add a match arm to `WIDGET_NAMES` for directives like `[double_slider]` — the macro emits the widget call directly. **Pattern B:** Render macro-generated content alongside hand-coded widgets in the same `update()` function — the generated `render_*()` functions are just regular Rust functions you can call anywhere.

## What we built

A complete widget showcase with selection controls, input fields, display widgets, styled buttons, and advanced response tracking.
