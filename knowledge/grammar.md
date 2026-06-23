# Litui DSL Grammar Specification

Quick-reference for the litui markdown DSL processed by `include_markdown_ui!` and `define_markdown_app!`.

---

## 1. YAML Frontmatter Schema

Delimited by `---` fences at the start of a `.md` file. All top-level keys are optional. Unknown fields produce a compile error (`#[serde(deny_unknown_fields)]` on every struct).

```yaml
page:       # PageDef — required per file in define_markdown_app!
styles:     # HashMap<String, StyleDef> — named style presets
widgets:    # HashMap<String, WidgetDef> — named widget configs
spacing:    # SpacingDef — layout spacing overrides
```

### `page:`

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | `String` | yes | — | Enum variant name for the page |
| `label` | `String` | yes | — | UI label for navigation |
| `default` | `bool` | no | `false` | Exactly one page must be `true` |
| `panel` | `String` | no | central panel | `"left"`, `"right"`, `"top"`, `"bottom"`, `"window"` |
| `width` | `f32` | no | — | Default width for side panels or windows |
| `height` | `f32` | no | — | Default height for top/bottom panels or windows |
| `open` | `String` | no | — | `bool` field on `AppState` controlling window visibility (window panel only) |

### `styles:` (map of `StyleDef`)

Each key is a style name referenced by `::key`, `.class`, or `::: frame key`.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `bold` | `bool` | inherit | Bold text |
| `italic` | `bool` | inherit | Italic text |
| `strikethrough` | `bool` | inherit | Strikethrough text |
| `underline` | `bool` | `false` | Underline text |
| `color` | `String` | inherit | Hex color `"#RRGGBB"` |
| `background` | `String` | inherit | Background hex color `"#RRGGBB"` |
| `size` | `f32` | inherit | Font size in points |
| `monospace` | `bool` | `false` | Use monospace font |
| `weak` | `bool` | `false` | Dimmed text color |
| `inner_margin` | `f32` | — | Frame inner margin px (`::: frame` only) |
| `outer_margin` | `f32` | — | Frame outer margin px |
| `stroke` | `f32` | — | Frame border stroke width px |
| `stroke_color` | `String` | — | Frame border stroke hex `"#RRGGBB"` |
| `corner_radius` | `f32` | — | Frame corner radius px |

Styles compose left-to-right via `merge_style_defs`: overlay `Some` values override base; `None` inherits.

### `widgets:` (map of `WidgetDef`)

Each key is referenced by `{key}` after a widget link.

| Field | Type | Default | Applies to |
|-------|------|---------|------------|
| `min` | `f64` | — | slider, double_slider, dragvalue |
| `max` | `f64` | — | slider, double_slider, dragvalue |
| `speed` | `f64` | — | dragvalue |
| `label` | `String` | — | slider, checkbox, toggle, combobox |
| `hint` | `String` | — | textedit, textarea, password |
| `format` | `String` | — | display (e.g., `"{:.1}"`) |
| `options` | `Vec<String>` | — | radio, combobox, selectable |
| `track_hover` | `bool` | — | button (generates `{name}_hovered: bool`) |
| `track_secondary` | `bool` | — | button (generates `{name}_secondary_count: u32`) |
| `suffix` | `String` | — | slider (e.g., `"deg"`) |
| `prefix` | `String` | — | slider (e.g., `"$"`) |
| `rows` | `usize` | 4 | textarea |
| `max_height` | `f64` | 200.0 | select, log |
| `fill` | `String` | — | progress (hex color `"#RRGGBB"`) |

### `spacing:`

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `paragraph` | `f32` | 8.0 | Vertical gap after paragraphs |
| `table` | `f32` | 8.0 | Vertical gap after tables |
| `heading_h1` | `f32` | 16.0 | Top spacing before H1 |
| `heading_h2` | `f32` | 12.0 | Top spacing before H2 |
| `heading_h3` | `f32` | 8.0 | Top spacing before H3 |
| `heading_h4` | `f32` | 4.0 | Top spacing before H4+ |
| `item` | `f32` | — | `ui.spacing_mut().item_spacing.y` override |

---

## 2. Widget Link Syntax

```
[WIDGET_NAME#id.class1.class2](content){config}
```

- **`WIDGET_NAME`** -- one of the 18 names below (case-sensitive)
- **`#id`** -- optional, sets `egui::Id` via `ui.push_id()`
- **`.class`** -- optional, resolved against `styles:` in frontmatter; multiple classes compose left-to-right
- **`(content)`** -- state field name or literal value; spaces via `<angle brackets>` or underscores
- **`{config}`** -- optional, key into `widgets:` frontmatter section; consumed from the next text event via lookahead

| Widget | Name | State Type | Default | Notes |
|--------|------|-----------|---------|-------|
| Button | `button` | `u32` (click count) | `0` | With `{config}`: generates `{config}_count: u32` |
| Slider | `slider` | `f64` | `0.0` | |
| Double Slider | `double_slider` | `f64` x2 (`_low`, `_high`) | `0.0`, `1.0` | Requires `egui_double_slider` crate |
| Checkbox | `checkbox` | `bool` | `false` | |
| Toggle | `toggle` | `bool` | `false` | iOS-style animated switch |
| TextEdit | `textedit` | `String` | `String::new()` | Single-line |
| TextArea | `textarea` | `String` | `String::new()` | Multi-line |
| Password | `password` | `String` | `String::new()` | Masked input |
| DragValue | `dragvalue` | `f64` | `0.0` | |
| Radio | `radio` | `usize` | `0` | Requires `options` in config |
| ComboBox | `combobox` | `usize` | `0` | Requires `options` in config |
| Selectable | `selectable` | `usize` | `0` | Horizontal segmented control; requires `options` |
| Select | `select` | `usize` + `Vec<String>` | `0`, `Vec::new()` | Runtime-populated list; `{config}` names the `Vec<String>` field |
| ColorPicker | `color` | `[u8; 4]` | `[255,255,255,255]` | RGBA |
| Progress | `progress` | `f64` or none | `0.0` | Literal float = stateless; field name = stateful |
| Display | `display` | `String` (self-declares) | `String::new()` | Read-only; self-declares if no input widget owns the field |
| Spinner | `spinner` | none | — | Stateless |
| Log | `log` | `Vec<String>` | `Vec::new()` | Scrollable, stick-to-bottom |
| Custom | `custom` | `Option<Box<dyn FnMut(&mut egui::Ui) + Send + Sync>>` | `None` | Escape hatch; see below |

### `[custom](slot)` escape hatch

A markdown link whose **text is literally `custom`** and whose destination is the slot identifier: `[custom](slot_name)`. It emits a slot the app fills in with arbitrary egui rendering.

- Generates `pub slot_name: Option<Box<dyn FnMut(&mut egui::Ui) + Send + Sync>>` on the state struct, default `None`.
- `Send + Sync` is **required**: the generated state struct is intended to live in a Bevy `Resource`.
- `FnMut` (not `Fn`) lets the closure mutate its own captured state across frames.
- Render uses a **take/replace** pattern: the closure is taken out of the `Option`, called with the current `ui`, then put back, so the state struct stays movable and free of aliasing borrows.
- When any custom slot is present, the generated state struct **drops its `Clone`/`Debug` derives** (a boxed closure is neither).
- A page whose entire body is `[custom](slot)` works (whole-page-as-slot).

```rust
state.slot_name = Some(Box::new(|ui| { ui.label("raw egui"); }));
```

See `examples/13_custom/`.

---

## 3. Block Directives

Opened by `::: directive [arg]` on a line by itself (inside a paragraph text event). Closed by `:::` on its own line. Can nest (stack-based).

| Directive | Syntax | Arg | State Generated | Description |
|-----------|--------|-----|-----------------|-------------|
| `foreach` | `::: foreach field` | field name | `Vec<{Field}Row>` | Iterates a collection; body must contain exactly one table or list with `{field}` references |
| `if` | `::: if field` | field name | `field: bool` | Conditional rendering; auto-declares `bool` field |
| `style` | `::: style field` | field name | `field: String` | Runtime text color override; field value must be a style name from frontmatter |
| `frame` | `::: frame [style_name]` | optional style name | none | Wraps content in `egui::Frame`; resolves style from frontmatter for frame properties |
| `horizontal` | `::: horizontal` | none | none | Wraps content in `ui.horizontal()` |
| `columns` | `::: columns N` | integer N | none | Splits into N columns via `ui.columns()`; use `::: next` to advance columns |

### `::: next`

Column separator inside `::: columns`. Advances to the next column. Error if used outside columns or if column count exceeded.

### `foreach` body rules

- Body must contain exactly one table or one list
- Blank lines required around the table/list (CommonMark parsing requirement)
- `{field}` references resolve to row struct fields (all `String`)
- Row struct name: `capitalize_first(field_name) + "Row"` (e.g., `items` -> `ItemsRow`)
- Generated struct is `pub` with `#[derive(Clone, Debug, Default)]`
- Style suffixes (`::key`) are not available inside foreach blocks
- Cannot nest inside a table cell

---

## 4. Style Suffixes

Three forms for applying named styles:

| Form | Syntax | Context | Description |
|------|--------|---------|-------------|
| Block suffix | `text ::key` | End of paragraph, heading, or list item | Applies `styles.key` to the entire block; resolved at compile time |
| Runtime suffix | `text ::$field` | End of paragraph, heading, or list item | Auto-declares `field: String` on state; applies `color` at runtime via `override_text_color` |
| Inline span | `::class(text)` | Anywhere in text content | Applies `styles.class` to `text` only; parsed from text events |

### Suffix detection rules

- `::key` -- suffix key must match `[a-zA-Z0-9_$]+`
- `::$field` -- the `$` prefix signals a runtime style field; only `color` is applied dynamically, other properties are compile-time only
- `::class(text)` -- parsed greedily left-to-right; class must exist in `styles:` frontmatter

---

## 5. Validation Rules

All validation occurs at compile time (proc macro expansion). Errors are emitted as `compile_error!`.

| What | When | Error |
|------|------|-------|
| Unknown frontmatter fields | YAML deserialization | serde error (via `deny_unknown_fields`) |
| Undefined style `::key` | Flush point (paragraph/heading end) | `"Undefined style key '{key}' in frontmatter"` |
| Undefined `.class` on widget | Widget link processing | `"Undefined style class '.{class}' in frontmatter"` |
| Undefined `::class()` inline span | Text event processing | `"Undefined style class '::{class}' in inline span"` |
| Invalid hex color | Any color field resolution | `"Color must be #RRGGBB (6 hex digits)"` |
| Unused widget configs | After parsing all pages | `"Unused widget config(s) in frontmatter widgets: section: {names}"` |
| Conflicting field types | `define_markdown_app!` field merging | `"Widget field '{name}' declared with conflicting types"` |
| `foreach` body empty | Block close (`:::`) | `"foreach body contains no {field} references"` |
| `::: columns` non-integer arg | Directive open | `"::: columns requires a number"` |
| `::: next` outside columns | Text event processing | `"::: next can only be used inside ::: columns"` |
| `::: next` exceeds column count | Column advancement | `"too many ::: next separators"` |
| Unknown widget name in link | `End(Link)` processing | `"Unknown widget name '{name}'. Valid names: ..."` |
| Unknown directive name | Directive open | `"Unknown block directive '{name}'"` |

### Frontmatter merging (`define_markdown_app!` with `parent:`)

- Parent `styles:` and `widgets:` are inherited by child pages
- Child values override parent on key collision
- `spacing:` merges field-by-field (child `Some` overrides parent)
- `page:` is always from the child (not inherited)
