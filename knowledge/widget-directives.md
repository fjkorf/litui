# Widget Directives

Widgets are embedded using markdown link syntax: `[widget](content){config}`. pulldown-cmark parses `[slider](volume)` as a link with text "slider" and URL "volume". The macro intercepts links whose text matches a known widget name.

## Detection

1. At `End(Link)`, parse selectors from link text via `parse_selectors()`: `button#id.class` -> base_name="button", id="id", classes=["class"]
2. Check if `base_name` is in `WIDGET_NAMES` list
3. Lookahead to the next event for a `Text("{config}")` event
4. Consume the `{config}` text and resolve it against frontmatter `widgets:` section
5. Resolve class-based styles from selectors (`.class` only — `{config}` is widget config, `::key` is for style application on text)
6. Emit widget code instead of a hyperlink

## Event Stream

For `[slider](volume){vol}`:
```
Start(Link(Inline, "volume", ""))
  Text("slider")
End(Link(...))
Text("{vol}")          <- consumed by lookahead
```

The event loop uses index-based iteration (`while event_idx < events.len()`) instead of `for event in events` to enable this lookahead. A `skip_next` flag consumes the `{config}` text event.

## Widget Types

| Widget | Syntax | State Type | Default |
|--------|--------|-----------|---------|
| Button | `[button.class](label){config}` | `u32` (click count) | `0` |
| Progress | `[progress](0.75)` or `[progress](field){config}` | None or `f64` | `0.0` |
| Log | `[log](field){config}` | `Vec<String>` | `Vec::new()` |
| Spinner | `[spinner]()` | None (stateless) | — |
| Slider | `[slider](field){config}` | `f64` | `0.0` |
| Double Slider | `[double_slider](field){config}` | `f64` x2 | `0.0`, `1.0` |
| Checkbox | `[checkbox](field)` | `bool` | `false` |
| TextEdit | `[textedit](field){config}` | `String` | `String::new()` |
| DragValue | `[dragvalue](field){config}` | `f64` | `0.0` |
| Radio | `[radio](field){config}` | `usize` | `0` |
| ComboBox | `[combobox](field){config}` | `usize` | `0` |
| ColorPicker | `[color](field)` | `[u8; 4]` | `[255,255,255,255]` |
| TextArea | `[textarea](field){config}` | `String` | `String::new()` |
| Password | `[password](field){config}` | `String` | `String::new()` |
| Toggle | `[toggle](field){config}` | `bool` | `false` |
| Selectable | `[selectable](field){config}` | `usize` | `0` |
| Select | `[select](index){list_field}` | `usize` + `Vec<String>` | `0`, `Vec::new()` |
| Display | `[display](field){config}` | `String` (self-declares) | `String::new()` |
| Foreach | `::: foreach field` ... `:::` | `Vec<RowStruct>` | `Vec::new()` |

## Double Slider (3rd-party: egui_double_slider)

Range slider with two handles. Requires `egui_double_slider` crate in the consumer's dependencies.

```markdown
[double_slider](frequency){freq_range}
```

Generates TWO state fields: `frequency_low: f64` and `frequency_high: f64`. Config uses the same `min`/`max` fields as regular slider.

```yaml
widgets:
  freq_range:
    min: 20
    max: 20000
```

The macro emits code referencing `egui_double_slider::DoubleSlider` — if the consumer doesn't have this crate, the Rust compiler gives an "unresolved import" error.

See `examples/third_party_widgets/` for a working example.

## Display Widget

Read-only widget that displays a value from `AppState`.

```markdown
[display](volume){vol_fmt}
```

Config supports `format` field for format strings:
```yaml
widgets:
  vol_fmt:
    format: "{:.1}"
```

Works inside table cells for grid-like state monitoring layouts.

Display widgets work in both `define_markdown_app!` and `include_markdown_ui!`.

### Display-only fields (self-declaration)

If a `[display]` references a field that no input widget declares, it **self-declares** the field as `String` on the generated state struct. This enables display-only pages where all data comes from code:

```markdown
| Stat | Value |
|------|-------|
| **Name** | [display](monster_name) |
| **HP** | [display](hp) |
| **AC** | [display](ac) |
```

This generates `monster_name: String`, `hp: String`, `ac: String` on `AppState`. Populate them from code:

```rust
state.monster_name = "Goblin".to_string();
state.hp = format!("{}/{}", hp.current, hp.max);
```

If an input widget on another page already declares the field (e.g., `[slider](volume)` declares `volume: f64`), the input widget's type wins — display reads the existing field.

Pages with only display-self-declared fields get a read-only render function (`state: &AppState`), not `&mut AppState`.

## Radio Button

Renders a group of radio buttons. Requires `options` in widget config.

```markdown
[radio](choice){opts}
```

```yaml
widgets:
  opts:
    options: ["Small", "Medium", "Large"]
```

State is `usize` (index into the options array). Generates `ui.radio_value()` for each option.

## ComboBox

Dropdown selection widget. Requires `options` in widget config.

```markdown
[combobox](selection){opts}
```

```yaml
widgets:
  opts:
    options: ["Red", "Green", "Blue"]
    label: "Color"
```

State is `usize` (index). Config supports `label` (display label) and `options` (choices). Generates `egui::ComboBox::show_index()`.

## ColorPicker

Color selection button. No config needed.

```markdown
[color](tint)
```

State is `[u8; 4]` (RGBA). Default is `[255, 255, 255, 255]`. The macro generates `egui::color_picker::color_edit_button_srgba()` with conversion to/from `[u8; 4]` since `Color32` can't be stored directly in the generated state struct.

## TextArea

Multi-line text editor. Like `textedit` but generates `TextEdit::multiline()`.

```markdown
[textarea](notes){cfg}
```

Config supports `hint` (placeholder text) and `rows` (desired visible row count, default 4).

```yaml
widgets:
  cfg:
    hint: "Write your notes here..."
    rows: 5
```

State is `String`.

## Password

Masked text input. Like `textedit` but with `.password(true)`.

```markdown
[password](secret){cfg}
```

Config supports `hint`. State is `String`. Characters are replaced with bullets in the UI.

## Toggle Switch

iOS-style animated toggle. Visually distinct from checkbox — a sliding pill shape.

```markdown
[toggle](dark_mode){cfg}
```

Config supports `label` (displayed next to the toggle). State is `bool`, default `false`. Uses the `toggle_switch()` helper function in `markdown_to_egui_helpers`.

## Selectable Labels

Tab-like toggle buttons in a horizontal row. Visually distinct from radio buttons — looks like a segmented control.

```markdown
[selectable](view){opts}
```

Config requires `options` list. State is `usize` (index). Generates `ui.selectable_value()` for each option in a `ui.horizontal()` layout.

## Stateful Progress Bar

`[progress]` accepts either a literal float or a state field name:

```markdown
[progress](0.75)              <!-- stateless: literal value -->
[progress](hp_frac){hp_bar}   <!-- stateful: reads f64 from AppState -->
```

When the content parses as a float, it's stateless (backwards compatible). Otherwise it creates a `f64` field on state.

Config supports `fill` for bar color:
```yaml
widgets:
  hp_bar:
    fill: "#8B0000"
```

Generated code: `ui.add(egui::ProgressBar::new(state.hp_frac as f32).show_percentage().fill(Color32::from_rgb(...)))`

## Log Widget

Scrollable message list that sticks to the bottom (newest messages visible):

```markdown
[log](messages){msg_cfg}
```

```yaml
widgets:
  msg_cfg:
    max_height: 200.0
```

State: `messages: Vec<String>`. Populate by pushing strings:
```rust
state.messages.push("The goblin hits you!".into());
```

Unlike `[foreach]`, log takes plain strings — no row struct, no `{field}` references. Uses `egui::ScrollArea::vertical().stick_to_bottom(true)`.

## Dynamic Styling

### ::: style block

Runtime color override using `ui.visuals_mut().override_text_color`:

```markdown
::: style hp_style

**HP:** [display](hp_text)

:::
```

State: `hp_style: String` — set to a frontmatter style name at runtime. The macro generates a `__resolve_style_color()` match table from all frontmatter styles. Only `color` is dynamically applied (bold, size, etc. are compile-time only).

Style blocks nest — inner overrides outer.

## Container Directives

Pages can specify their egui container via `panel:` in frontmatter:

```yaml
page:
  name: Stats
  label: Stats
  panel: right
  width: 180
```

Panel values: `left`, `right`, `top`, `bottom`, `window`. Omit for central panel (default).

When any page has `panel:`, `MdApp` gains `show_all(&egui::Context)`:
- Side panels are always visible (persist across page switches)
- Top/bottom panels are always visible
- Windows appear when the current page matches
- Central panel dispatches non-container pages
- Navigation bar auto-generated as a top panel
- Non-breaking: `show_page(&mut Ui)` still works

## Slider Suffix/Prefix

Sliders support `suffix` and `prefix` config fields for unit display:

```yaml
widgets:
  angle: { min: 0, max: 360, suffix: "°", label: Angle }
  price: { min: 0, max: 1000, prefix: "$", label: Price }
```

These map directly to egui's `Slider::suffix()` and `Slider::prefix()` methods.

## Select (Runtime Selectable List)

Scrollable selection list populated from a `Vec<String>` at runtime. Like `[selectable]` but with dynamic options instead of compile-time YAML.

```markdown
[select](chosen_species){species_list}
```

- `chosen_species` — `usize` field (selection index)
- `species_list` — `Vec<String>` field (labels, populated from code)

Config supports `max_height` (pixels, default 200):

```yaml
widgets:
  species_list:
    max_height: 150.0
```

Populate from code:
```rust
state.species_list = vec!["Human".into(), "Elf".into(), "Dwarf".into()];
```

Generated code: `egui::ScrollArea` with `ui.selectable_label()` for each item.

## Foreach (Collection Iteration Block)

Block-level directive that iterates a `Vec<RowStruct>` and renders each row. The row struct is auto-generated at compile time from `{field}` references in the body. Body supports tables and lists.

### Table template

```markdown
::: foreach items

| {letter} | {name} | {quantity} |
|-----------|--------|------------|

:::
```

Generates:
```rust
pub struct ItemsRow {
    pub letter: String,
    pub name: String,
    pub quantity: String,
}
// On state: pub items: Vec<ItemsRow>
```

Inside a foreach block, `{field}` resolves to row fields (not frontmatter style keys). The struct name is derived from the field name: `items` → `ItemsRow`.

Populate from code:
```rust
state.items.push(ItemsRow {
    letter: "a".into(),
    name: "Iron Sword".into(),
    quantity: "1".into(),
});
```

### List template

```markdown
::: foreach effects
- **{name}**: {description}
:::
```

### Important: blank lines are required

CommonMark requires a blank line before block-level elements (tables, lists). The foreach body MUST have blank lines around the table:

```markdown
::: foreach items
                          ← blank line required
| {letter} | {name} |
|-----------|--------|
                          ← blank line required
:::
```

Without blank lines, pulldown-cmark treats the table syntax as literal text. litui detects this and emits a compile error: `"foreach body contains no {field} references"`.

### Row struct naming convention

The generated struct name is `capitalize_first(field_name) + "Row"`:
- `items` → `ItemsRow`
- `effects` → `EffectsRow`
- `inv_items` → `Inv_itemsRow`

The struct is `pub` with `#[derive(Clone, Debug)]` and `Default`. External code references it to populate the Vec:

```rust
// Direct construction
state.items.push(ItemsRow {
    letter: "a".into(),
    name: "Iron Sword".into(),
    quantity: "1".into(),
});

// Or via Default + field assignment
let mut row = ItemsRow::default();
row.name = "Health Potion".into();
state.items.push(row);
```

### Key constraints (v1)

- Body must contain exactly one table or one list
- Blank lines required around the table/list (CommonMark parsing)
- All `{field}` references resolve to row struct fields (String)
- Style suffixes (`::key`) are not available inside foreach blocks
- Grid IDs are unique per iteration (no egui ID collisions)
- `foreach` inside a table cell is NOT supported (it's a block-level directive)

## Table Cell Support

Most widgets work inside table cells (the macro emits them as `Fragment::Widget` within the Grid closure). Supported: `[button]`, `[slider]`, `[checkbox]`, `[display]`, `[progress]`, `[spinner]`, `[select]`, `[combobox]`, `[radio]`, `[toggle]`, `[color]`, `[textedit]`, `[textarea]`, `[password]`, `[dragvalue]`, `[selectable]`.

NOT supported in table cells: `foreach` (block-level directive that generates a `for` loop — cannot nest inside a Grid cell closure).

## Advanced Button Response

Buttons with `{config}` always generate a `{config}_count: u32` click counter. Additional response tracking can be enabled via widget config.

### Generated field names

| Config | Field | Type | Condition |
|--------|-------|------|-----------|
| `{cfg}` | `cfg_count` | `u32` | Always when button has `{config}` |
| `{cfg}` | `cfg_hovered` | `bool` | When `track_hover: true` |
| `{cfg}` | `cfg_secondary_count` | `u32` | When `track_secondary: true` |

Example: `[button](Submit){on_submit}` with `track_hover: true` generates `on_submit_count: u32` and `on_submit_hovered: bool`.

### Config:

```yaml
widgets:
  on_submit:
    track_hover: true
    track_secondary: true
```

This generates additional state fields:
- `on_submit_hovered: bool` (when `track_hover: true`) — updated every frame
- `on_submit_secondary_count: u32` (when `track_secondary: true`) — incremented on right-click

## Image Widget

Standard markdown image syntax renders via `egui::Image`:

```markdown
![alt text](image.png)
![](https://example.com/logo.png)
```

- Relative paths are resolved against `CARGO_MANIFEST_DIR` at compile time and converted to `file://` URIs
- Absolute URLs (`http://`, `https://`, `file://`) are passed through unchanged
- Alt text is set via `.alt_text()` if provided
- Works inside table cells
- **Requires `egui_extras::install_image_loaders(ctx)`** in the app to actually load images at runtime

## Multi-Word Content

Spaces in link URLs break pulldown-cmark parsing. Use:
- Angle brackets: `[button](<Click me>)` — CommonMark spec, spaces allowed
- Underscores: `[button](Click_me)` — macro converts `_` to spaces

## Generated State

### `include_markdown_ui!` (single file)

- No stateful widgets: returns `impl FnMut(&mut egui::Ui)`
- Has stateful widgets: returns `(fn(&mut Ui, &mut MdFormState), MdFormState)`

### `define_markdown_app!` (multi-page)

All widget fields across all pages merge into a single flat `AppState` struct. Two pages can declare the same field if the types match (shared state). Conflicting types produce a compile error.

Render function signatures depend on state usage:
- Pages with mutable widgets (slider, checkbox, etc.): `render_x(ui: &mut Ui, state: &mut AppState)`
- Pages with only display widgets (read-only): `render_x(ui: &mut Ui, state: &AppState)`
- Stateless pages: `render_x(ui: &mut Ui)`
