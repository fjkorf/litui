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
| Datepicker | `[datepicker](field)` | `chrono::NaiveDate` | `NaiveDate::default()` |
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

See `examples/10_advanced/` and tutorial 10 for widget integration patterns.

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

Display widgets work in both `define_litui_app!` and `include_litui_ui!`.

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

Config supports `label` (displayed next to the toggle). State is `bool`, default `false`. Uses the `toggle_switch()` helper function in `litui_helpers`.

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

## Datepicker

Calendar date picker using `egui_extras::DatePickerButton`. Shows the selected date and opens a popup calendar on click.

```markdown
[datepicker](due_date)
```

State: `due_date: chrono::NaiveDate`. Consumer must add dependencies:

```toml
egui_extras = { version = "0.33", features = ["datepicker"] }
chrono = "0.4"
```

The widget emits `ui.add(egui_extras::DatePickerButton::new(&mut state.due_date))`.

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

## Collapsing Header (`::: collapsing`)

Wraps content in `egui::CollapsingHeader` — a clickable header that toggles a collapsible body.

### Syntax

```markdown
::: collapsing "Section Title"

Hidden content, widgets, tables, etc.

:::
```

### Title forms

| Form | Example | Description |
|------|---------|-------------|
| Quoted string | `::: collapsing "Details"` | Static title text |
| Unquoted word | `::: collapsing Details` | Single-word static title |
| Field reference | `::: collapsing {name}` | Title from `state.name` or `__row.name` in foreach |

### State tracking (optional)

Append `{bool_field}` to track open/closed state in `AppState`:

```markdown
::: collapsing "Advanced" {show_advanced}

Content here.

:::
```

Auto-declares `show_advanced: bool` on `AppState` (default `false`). Enables bidirectional sync — the app can programmatically open/close the section, and user clicks update the field.

Field title + state tracking also works:

```markdown
::: collapsing {bone_name} {bone_open}
```

### Generated code

**Without state tracking** — egui manages open/close internally:

```rust
egui::CollapsingHeader::new("Section Title")
    .id_salt("litui_collapsing_0")
    .default_open(false)
    .show(ui, |ui| {
        // body content
    });
```

**With state tracking** — bidirectional sync via `CollapsingState`. Uses `show_toggle_button` + `show_body_indented` (both `&mut self`) instead of `show_header` (which consumes `self`):

```rust
{
    let __collapsing_id = ui.make_persistent_id("litui_collapsing_0");
    let mut __cs = egui::collapsing_header::CollapsingState::load_with_default_open(
        ui.ctx(), __collapsing_id, state.show_advanced,
    );
    if state.show_advanced != __cs.is_open() {
        __cs.set_open(state.show_advanced);
    }
    let __toggle = ui.horizontal(|ui| {
        let __btn = __cs.show_toggle_button(ui, paint_default_icon);
        ui.label("Advanced");
        __btn
    });
    __cs.show_body_indented(&__toggle.inner, ui, |ui| {
        // body content
    });
    state.show_advanced = __cs.is_open();
}
```

### Nesting

Collapsing inside collapsing works — each gets a unique `id_salt` from its `collapsing_index`:

```markdown
::: collapsing "Outer"

::: collapsing "Inner"

Deeply nested content.

:::

:::
```

### In foreach

Use field references for per-row titles. The title field value provides natural ID uniqueness:

```markdown
::: foreach bones

::: collapsing {name}

| Property | Value |
|----------|-------|
| {length} | {weight} |

:::

:::
```

### Key details

- Default state: always starts collapsed (`default_open: false`)
- ID stability: each collapsing instance gets `litui_collapsing_N` salt (N is parse order)
- All litui content works inside the body: widgets, tables, lists, nested directives
- In foreach context, `{field}` references in the title resolve to `__row.field`

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

Optional `background` field sets the panel/window frame fill:

```yaml
page:
  name: Shapes
  panel: left
  width: 220
  background: transparent           # fully transparent
  # background: "#1A1A2E"           # opaque hex
  # background: "#1A1A2E80"         # semi-transparent (RRGGBBAA)
```

When set, emits `.frame(Frame::NONE.fill(...))` on the container. Useful for Bevy apps where the 3D viewport should show through panels.

When any page has `panel:`, `LituiApp` gains `show_all(&egui::Context)`:
- Side panels are always visible (persist across page switches), unless gated by `open:`
- Top/bottom panels are always visible, unless gated by `open:`
- Windows appear when the current page matches
- Central panel dispatches non-container pages
- Navigation bar auto-generated (position controlled by `nav:` config)
- Non-breaking: `show_page(&mut Ui)` still works

### `navigable:` field

Controls whether a page appears in `show_nav()`. Default: `true` for central pages (no `panel:`), `false` for panel/window pages. Override explicitly:

```yaml
page:
  name: Stats
  label: Stats
  panel: right
  navigable: true   # force this panel page into the nav bar
```

The generated `Page` enum has both `ALL` (every page) and `NAV_PAGES` (only navigable pages). `show_nav()` iterates `NAV_PAGES` by default.

### `nav:` parent config

The parent `_app.md` can configure navigation bar behavior:

```yaml
nav:
  position: top      # "top" | "bottom" | "none"
  show_all: false    # if true, show ALL pages in nav including panels/windows
```

- `position: "top"` (default) — nav bar rendered as a top panel
- `position: "bottom"` — nav bar rendered as a bottom panel
- `position: "none"` — no auto nav; call `show_nav(ui)` manually
- `show_all: true` — `show_nav()` iterates `Page::ALL` instead of `Page::NAV_PAGES`

## Frame Styles

Styles with frame properties (`inner_margin`, `stroke`, `corner_radius`, etc.) wrap the paragraph in an `egui::Frame`:

```yaml
styles:
  panel:
    inner_margin: 8
    background: "#1A1A2E"
    corner_radius: 4
```

Applied with `::: frame panel`. Maps to:
- `inner_margin` → `Frame::inner_margin()`
- `outer_margin` → `Frame::outer_margin()`
- `stroke` + `stroke_color` → `Frame::stroke()`
- `corner_radius` → `Frame::corner_radius()`
- `background` → `Frame::fill()`

## Slider Suffix/Prefix

Sliders support `suffix` and `prefix` config fields for unit display:

```yaml
widgets:
  angle: { min: 0, max: 360, suffix: "°", label: Angle }
  price: { min: 0, max: 1000, prefix: "$", label: Price }
```

These map directly to egui's `Slider::suffix()` and `Slider::prefix()` methods.

## Slider Integer Mode, Step, and Decimals

Additional slider config options for numeric precision:

```yaml
widgets:
  octaves: { min: 0, max: 6, integer: true, label: "Octaves" }
  rotation: { min: -180, max: 180, step: 5.0, suffix: "°", label: "Rotation" }
  smooth_k: { min: 0.0, max: 0.1, decimals: 3, label: "Smooth K" }
```

| Field | egui method | Description |
|-------|------------|-------------|
| `integer: true` | `.integer()` | Snaps to whole numbers (sets `fixed_decimals(0)`, `smallest_positive(1.0)`, `step_by(1.0)`) |
| `step: 5.0` | `.step_by(5.0)` | Quantize to discrete steps |
| `decimals: 3` | `.fixed_decimals(3)` | Fixed decimal places in display |

## DragValue Enhanced Config

DragValue now supports range clamping, suffix/prefix, and decimal control:

```yaml
widgets:
  roughness: { min: 0.0, max: 1.0, speed: 0.01, label: "Roughness" }
  angle: { speed: 0.5, suffix: "°", decimals: 1 }
  precise: { speed: 0.001, decimals: 4 }
```

| Field | egui method | Description |
|-------|------------|-------------|
| `min`/`max` | `.range(min..=max)` | Clamp drag range (both optional, one-sided supported) |
| `suffix` | `.suffix("°")` | Appended to display |
| `prefix` | `.prefix("$")` | Prepended to display |
| `decimals` | `.fixed_decimals(N)` | Fixed decimal places |
| `speed` | `.speed(0.1)` | Drag sensitivity (default 0.1) |

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

### Tree mode (`foreach ... children`)

Add `children` after the field name to render a recursive tree:

```markdown
::: foreach bones children

::: collapsing {name}

{description}

:::

:::
```

This generates a row struct with `children: Vec<Self>`:

```rust
pub struct BonesRow {
    pub name: String,
    pub description: String,
    pub children: Vec<BonesRow>,
}
```

The body renders recursively — for each node, the body is rendered, then `row.children` is rendered with the same template. A `__tree_depth: usize` variable is available in the generated code (0 = root level).

Populate from code:

```rust
let mut arm = BonesRow::default();
arm.name = "Arm".into();
arm.children.push(BonesRow {
    name: "Hand".into(),
    ..Default::default()
});
state.bones.push(arm);
```

**Key details:**
- `::: collapsing {name}` inside tree foreach gets dynamic ID salts (depth + pointer) to avoid collisions
- `::: collapsing {name} {is_open}` works inside foreach — `is_open: bool` is added to the row struct, not AppState
- All standard body content works: text, tables, widgets, nested collapsing

### Inner foreach (nested collections)

A foreach inside another foreach iterates a `Vec` on the parent row struct:

```markdown
::: foreach bones children

::: collapsing {name} {is_open}

::: foreach shapes

| {shape_name} | {shape_type} |
|---|---|

:::

:::

:::
```

This generates nested row structs:

```rust
pub struct BonesRow {
    pub name: String,
    pub is_open: bool,          // from collapsing {is_open}
    pub shapes: Vec<ShapesRow>, // from inner foreach
    pub children: Vec<BonesRow>,
}
pub struct ShapesRow {
    pub shape_name: String,
    pub shape_type: String,
}
```

The inner foreach uses `__row.shapes` as its collection source (Rust's lexical scoping with shadowing handles the variable binding naturally). All standard foreach features work inside: `{field}` references, widgets, tables.

**Constraints:**
- Tree foreach (`::: foreach X children`) cannot be nested inside another foreach
- Regular (non-tree) inner foreach nests to arbitrary depth via shadowing

### Key constraints

- Body must contain exactly one table or one list
- Blank lines required around the table/list (CommonMark parsing)
- Display `{field}` references resolve to row struct fields (`String`)
- Style suffixes work inside foreach: `::key` (static) and `::$field` (dynamic, reads from row struct)
- Grid IDs are unique per iteration (no egui ID collisions)
- `foreach` inside a table cell is NOT supported (it's a block-level directive)

### Widgets Inside Foreach

Input widgets work inside `::: foreach` blocks. They generate typed fields on the row struct instead of the main state struct:

```markdown
::: foreach items

| [checkbox](done) | {name} | [button](delete){on_delete} |
|---|---|---|

:::
```

Generated row struct:
```rust
pub struct ItemsRow {
    pub done: bool,           // from [checkbox](done)
    pub name: String,         // from {name}
    pub on_delete_count: u32, // from [button](delete){on_delete}
}
```

Supported widgets: checkbox, toggle, button, textedit, textarea, slider, dragvalue, display, progress.

Widget configs (`{cfg}`) reference the global frontmatter `widgets:` section, not per-row config. The foreach loop iterates `&mut state.items`, so widgets can modify per-row values. Widget IDs are hashed with the row pointer to avoid egui ID collisions across rows.

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

## Runtime Style on Paragraphs (`::$field`)

Paragraphs, headings, and list items can use runtime styles via a `$`-prefixed key:

```markdown
Some status text. ::$status_style
```

This auto-declares `status_style: String` on `AppState` and wraps the emitted content in a `__resolve_style_color()` override block. Set the field to a style name from frontmatter at runtime:

```rust
state.status_style = "danger".into();
```

Only the `color` property of the resolved style is applied at runtime (via `ui.visuals_mut().override_text_color`). All other style properties (bold, size, etc.) are compile-time only.

## Horizontal Layout (`::: horizontal`)

Wraps content in `ui.horizontal()` for side-by-side rendering:

```markdown
::: horizontal

[button](Save) [button](Cancel)

:::
```

Content inside still flushes through `ui.horizontal_wrapped()` per paragraph. The outer `ui.horizontal()` makes them flow left-to-right on one row.

## Column Layout (`::: columns N`)

Splits content into N equal-width columns using `ui.columns()`:

```markdown
::: columns 2

Left column content.

::: next

Right column content.

:::
```

Use `::: next` to advance to the next column. Each column gets its own `Ui` — all standard markdown elements (paragraphs, headings, lists, tables, widgets) work inside columns.

## Spacing Configuration

Frontmatter `spacing:` section overrides default spacing values:

```yaml
spacing:
  paragraph: 12.0     # gap after paragraphs (default 8)
  table: 12.0         # gap after tables (default 8)
  heading_h1: 20.0    # top spacing before H1 (default 16)
  heading_h2: 16.0    # before H2 (default 12)
  heading_h3: 10.0    # before H3 (default 8)
  heading_h4: 6.0     # before H4+ (default 4)
  item: 4.0           # egui item_spacing.y override
```

All fields are optional — omitted values use built-in defaults. The `item` field maps to `ui.spacing_mut().item_spacing.y` emitted at the start of the render function.

In `define_litui_app!` with a parent file, spacing defined in the parent propagates to all child pages. Child pages can override individual values.

## Panel/Window Visibility Control

All panel types and windows support `open:` for state-driven visibility:

```yaml
page:
  name: Inventory
  label: Items
  panel: window
  open: show_inventory
```

```yaml
page:
  name: Stats
  label: Stats
  panel: right
  open: show_stats
```

When `open:` is specified:
- A `bool` field (e.g., `show_inventory`, `show_stats`) is auto-declared on `AppState` (default `false`)
- For windows: the window gets an X close button via `egui::Window::open()`
- For side/top/bottom panels: the panel is hidden when the bool is `false`
- Visibility is state-driven, independent of page navigation

When `open:` is absent, panels are always visible and windows use page-navigation behavior (visible when navigated to).

## Generated State

### `include_litui_ui!` (single file)

- No stateful widgets: returns `impl FnMut(&mut egui::Ui)`
- Has stateful widgets: returns `(fn(&mut Ui, &mut LituiFormState), LituiFormState)`

### `define_litui_app!` (multi-page)

All widget fields across all pages merge into a single flat `AppState` struct. Two pages can declare the same field if the types match (shared state). Conflicting types produce a compile error.

Render function signatures depend on state usage:
- Pages with mutable widgets (slider, checkbox, etc.): `render_x(ui: &mut Ui, state: &mut AppState)`
- Pages with only display widgets (read-only): `render_x(ui: &mut Ui, state: &AppState)`
- Stateless pages: `render_x(ui: &mut Ui)`
