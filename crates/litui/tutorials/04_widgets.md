# Widgets

> Run it: `cargo run -p widgets_example`

Widgets turn your markdown into interactive forms. Sliders, checkboxes, text fields, buttons — all declared inline with link syntax.

## The widget syntax

```markdown
[widget_type](field_name){config}
```

- `widget_type` — one of the recognized widget names
- `field_name` — becomes a field on the generated state struct
- `{config}` — references a key in the YAML `widgets:` section (optional for some widgets)

## State and rendering

When your markdown contains stateful widgets, the macro return type changes:

```rust,ignore
// No widgets — returns a closure
let render = include_markdown_ui!("content.md");
render(ui);

// With widgets — returns a tuple
let (render, mut state) = include_markdown_ui!("form.md");
render(ui, &mut state);
```

The `state` is a generated `MdFormState` struct with a field for each widget. You read and write these fields from Rust just like any other struct.

## YAML widget config

Widget parameters live in the frontmatter `widgets:` section:

```yaml
widgets:
  vol:
    min: 0
    max: 100
    label: Volume
  name_cfg:
    hint: "Enter your name"
```

Reference them with `{key}` after the widget link. This is different from `::key` on text — on widgets, `{key}` means config, not style. Use `.class` selectors for widget styling.

## Widget catalog

### Slider

```markdown
[slider](volume){vol}
```

```yaml
widgets:
  vol:
    min: 0
    max: 100
    label: Volume
```

State type: `f64`. Default: `0.0`. The label appears next to the slider.

#### Suffix and prefix

Sliders support `suffix` and `prefix` config fields to annotate the displayed value:

```yaml
widgets:
  angle:
    min: 0
    max: 360
    suffix: "°"
  price:
    min: 0
    max: 999
    prefix: "$"
```

```text
[slider](heading){angle}
[slider](cost){price}
```

The suffix/prefix text is appended or prepended to the slider's value label.

### Checkbox

```markdown
[checkbox](muted)
```

State type: `bool`. Default: `false`. Optional config for a label:

```yaml
widgets:
  mute_cfg:
    label: "Mute audio"
```

```markdown
[checkbox](muted){mute_cfg}
```

### TextEdit

```markdown
[textedit](username){cfg}
```

```yaml
widgets:
  cfg:
    hint: "Enter name"
```

State type: `String`. Default: `String::new()`. The hint text shows as placeholder when the field is empty.

### TextArea

```text
[textarea](notes){cfg}
```

```yaml
widgets:
  cfg:
    hint: "Write your notes here..."
    rows: 6
```

State type: `String`. Default: `String::new()`. Multi-line text editor. `rows` sets the visible height (default: 4). The `hint` works the same as TextEdit.

### Password

```text
[password](secret){cfg}
```

```yaml
widgets:
  cfg:
    hint: "Enter password"
```

State type: `String`. Default: `String::new()`. Masked single-line input — characters render as dots.

### DragValue

```markdown
[dragvalue](count){cfg}
```

```yaml
widgets:
  cfg:
    speed: 0.5
```

State type: `f64`. Default: `0.0`. Drag left/right to change the value. `speed` controls sensitivity.

### Radio

```markdown
[radio](choice){opts}
```

```yaml
widgets:
  opts:
    options: ["Small", "Medium", "Large"]
```

State type: `usize` (index into the options array). Default: `0`. Renders a vertical group of `ui.radio_value()` buttons.

### Toggle Switch

```text
[toggle](dark_mode){cfg}
```

```yaml
widgets:
  cfg:
    label: "Dark mode"
```

State type: `bool`. Default: `false`. iOS-style animated toggle, visually distinct from checkbox. The `label` config is optional.

### Selectable Labels

```text
[selectable](view){opts}
```

```yaml
widgets:
  opts:
    options: ["Inventory", "Stats", "Map"]
```

State type: `usize` (index into the options array). Default: `0`. Renders as horizontal tab-like toggle buttons, visually distinct from radio.

### ComboBox

```markdown
[combobox](selection){opts}
```

```yaml
widgets:
  opts:
    options: ["Red", "Green", "Blue"]
    label: "Color"
```

State type: `usize` (index). Default: `0`. Dropdown menu via `egui::ComboBox::show_index()`. The `label` appears next to the dropdown.

### ColorPicker

```markdown
[color](tint)
```

State type: `[u8; 4]` (RGBA). Default: `[255, 255, 255, 255]`. No config needed. Renders a color button that opens egui's color picker. The macro handles conversion between `[u8; 4]` and `Color32` internally.

![Widget form](img/widgets_form.png)

## Stateless widgets

These don't create state fields. They render inline and that's it.

### Button

```markdown
[button](Submit)
```

A plain button with no config is stateless — it renders but you can't detect clicks. To track clicks, add a config key:

```markdown
[button](Submit){on_click}
```

This generates `state.on_click_count: u32`, incremented each time the button is clicked.

Multi-word labels use underscores or angle brackets:

```markdown
[button](Save_Changes)
[button](<Save Changes>)
```

### Progress bar

```markdown
[progress](0.75)
```

Renders a progress bar at 75%. The value is a float literal, not a state field.

### Spinner

```markdown
[spinner]()
```

An animated loading spinner. No state, no config, just spin.

## Advanced button tracking

Buttons can track more than clicks. Enable hover and secondary click detection via widget config:

```yaml
widgets:
  on_submit:
    track_hover: true
    track_secondary: true
```

```markdown
[button](Submit){on_submit}
```

This generates three state fields:

- `on_submit_count: u32` — left-click count
- `on_submit_hovered: bool` — true while the cursor is over the button (updated every frame)
- `on_submit_secondary_count: u32` — right-click count

## Reading state in Rust

The generated state struct has public fields. Read them directly:

```rust,ignore
let (render, mut state) = include_markdown_ui!("form.md");

egui::CentralPanel::default().show(ctx, |ui| {
    render(ui, &mut state);

    if state.on_submit_count > 0 {
        println!("Volume: {}", state.volume);
        println!("Username: {}", state.username);
        println!("Muted: {}", state.muted);
    }
});
```

Field names match the `field_name` in `[widget](field_name)`. Types match the widget catalog above.

![New widgets](img/widgets_new.png)

## Stateful progress bar

The literal `[progress](0.75)` is static. For runtime values, use a field name:

```text
[progress](hp_frac){hp_bar}
```

```yaml
widgets:
  hp_bar:
    fill: "#8B0000"
```

This generates `hp_frac: f64` on state. The `fill` config sets the bar color. Without `fill`, the default egui color is used.

Populate from code:
```rust,ignore
state.hp_frac = player.hp as f64 / player.max_hp as f64;
```

## Log widget

Scrollable message list that sticks to the bottom (newest messages visible):

```text
[log](messages){msg_cfg}
```

```yaml
widgets:
  msg_cfg:
    max_height: 200.0
```

State: `messages: Vec<String>`. Populate by pushing strings:
```rust,ignore
state.messages.push("The goblin hits you!".into());
state.messages.push("You take 5 damage.".into());
```

Unlike `[foreach]`, log takes plain strings — no row struct, no `{field}` references.

## Conditional sections — ::: if

Show or hide content based on a bool in AppState:

```text
::: if has_orb

**ORB OF ZOT** ::gold

:::
```

State: `has_orb: bool` — self-declared if no other widget declares it. When `false`, the content is completely absent (no layout space taken). When `true`, it renders normally.

Set from code:
```rust,ignore
state.has_orb = player.inventory.contains(&Item::OrbOfZot);
```

Conditional blocks can contain any content — paragraphs, headings, widgets, tables, even nested blocks.

## Display widget

Read-only widget that shows a value from shared state. Only works inside `define_markdown_app!`:

```text
[display](volume){vol_fmt}
```

```yaml
widgets:
  vol_fmt:
    format: "{:.1}"
```

### Self-declaration

Display widgets **self-declare** their field as `String` when no input widget elsewhere in the app already declares it. This means a page can be purely display-only — no sliders, no text fields, just `[display]` references — and the macro still generates the state struct.

If an input widget on another page already declares the field (e.g., a slider declares `volume` as `f64`), display uses that widget's type. If nothing else declares it, display claims it as `String`.

This enables display-only pages where all data comes from code — ECS systems, API responses, computed values — rather than user input.

### Example: monster stat card

A stat card page with no input widgets at all:

```text
# [display](monster_name)

| Stat | Value |
|------|-------|
| HP | [display](hp) |
| ATK | [display](atk) |
| DEF | [display](def) |
| Type | [display](element) |
```

Every field self-declares as `String`. Populate them from Rust:

```rust,ignore
// In a bevy_ecs system, or wherever you have access to AppState
state.monster_name = "Fire Drake".into();
state.hp = "240".into();
state.atk = "18".into();
state.def = "12".into();
state.element = "Fire".into();
```

No input widgets needed. The page is a pure read-only view driven entirely by code.

## Select — runtime selectable list

Like `[selectable]` but with dynamic options from a `Vec<String>` populated at runtime:

```text
[select](chosen_species){species_list}
```

```yaml
widgets:
  species_list:
    max_height: 150.0
```

This generates two state fields: `chosen_species: usize` (selection index) and `species_list: Vec<String>` (labels). Populate the list from code:

```rust,ignore
state.species_list = vec!["Human".into(), "Elf".into(), "Dwarf".into()];
```

Renders as a scrollable `egui::ScrollArea` with `selectable_label` for each item.

## Foreach — iterating dynamic collections

For variable-length lists (inventories, spell lists, status effects), use `::: foreach` to iterate a `Vec` and render each row:

```text
::: foreach items

| {letter} | {name} | {quantity} |
|-----------|--------|------------|

:::
```

**Important:** The blank lines before and after the table are required. CommonMark needs paragraph
separation for block-level elements like tables. Without them, the table syntax becomes literal
text and litui emits a compile error.

The macro discovers `{letter}`, `{name}`, `{quantity}` and generates a row struct:

```rust,ignore
pub struct ItemsRow {
    pub letter: String,
    pub name: String,
    pub quantity: String,
}
// On state: pub items: Vec<ItemsRow>
```

Populate from an ECS system or app code:

```rust,ignore
let mut row = ItemsRow::default();
row.letter = "a".into();
row.name = "Iron Sword".into();
row.quantity = "1".into();
state.items.push(row);
```

The struct name follows the convention: `capitalize_first(field_name) + "Row"` — so `items` becomes
`ItemsRow`, `effects` becomes `EffectsRow`. The struct is `pub` with `Default`, so external code
can construct and populate it.

Inside a foreach block, `{field}` resolves to row struct fields — not frontmatter style keys.

Foreach also works with lists:

```text
::: foreach effects
- **{name}**: {description}
:::
```

**Constraints (v1):** Body must contain one table or one list. All row fields are `String`.

## Previous / Next

Previous: [Tables](crate::_tutorial::_03_tables)

Next: [Selectors](crate::_tutorial::_05_selectors_and_spans) — CSS-like class and ID selectors for fine-grained styling.
