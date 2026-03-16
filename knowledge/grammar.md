# Litui DSL Grammar Specification

Quick-reference for the litui markdown DSL processed by `include_litui_ui!` and `define_litui_app!`.

---

## 1. YAML Frontmatter Schema

Delimited by `---` fences at the start of a `.md` file. All top-level keys are optional. Unknown fields produce a compile error (`#[serde(deny_unknown_fields)]` on every struct).

```yaml
page:       # PageDef — required per file in define_litui_app!
styles:     # HashMap<String, StyleDef> — named style presets
widgets:    # HashMap<String, WidgetDef> — named widget configs
spacing:    # SpacingDef — layout spacing overrides
theme:      # ThemeDef — egui Visuals overrides (parent only)
nav:        # NavDef — navigation bar configuration (parent only)
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
| `open` | `String` | no | — | `bool` field on `AppState` controlling panel/window visibility. Works on all panel types. |
| `navigable` | `bool` | no | `true` for central, `false` for panel/window | Whether this page appears in `show_nav()` |

### `styles:` (map of `StyleDef`)

Each key is a style name referenced by `::key`, `.class`, or `::: frame key`.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `bold` | `bool` | inherit | Bold text |
| `italic` | `bool` | inherit | Italic text |
| `strikethrough` | `bool` | inherit | Strikethrough text |
| `underline` | `bool` | `false` | Underline text |
| `color` | `String` | inherit | Hex `"#RRGGBB"` or semantic keyword (see below) |
| `background` | `String` | inherit | Hex `"#RRGGBB"` or semantic keyword (see below) |
| `size` | `f32` | inherit | Font size in points |
| `monospace` | `bool` | `false` | Use monospace font |
| `weak` | `bool` | `false` | Dimmed text color |
| `inner_margin` | `f32` | — | Frame inner margin px (`::: frame` only) |
| `outer_margin` | `f32` | — | Frame outer margin px |
| `stroke` | `f32` | — | Frame border stroke width px |
| `stroke_color` | `String` | — | Frame border stroke hex `"#RRGGBB"` or semantic keyword |
| `corner_radius` | `f32` | — | Frame corner radius px |

Styles compose left-to-right via `merge_style_defs`: overlay `Some` values override base; `None` inherits.

#### Semantic color keywords

Color fields (`color`, `background`, `stroke_color`) accept either hex `"#RRGGBB"` or a semantic keyword that references an egui Visuals field. Semantic colors resolve at runtime and automatically adapt to dark/light mode.

| Keyword | egui Field | Typical Use |
|---------|-----------|-------------|
| `text` | `widgets.noninteractive.fg_stroke.color` | Default text color |
| `strong` | `strong_text_color()` | Bold/heading text |
| `weak` | `weak_text_color()` | Dimmed secondary text |
| `hyperlink` | `hyperlink_color` | Link-colored text |
| `warn` | `warn_fg_color` | Warning indicators |
| `error` | `error_fg_color` | Error indicators |
| `code_bg` | `code_bg_color` | Code block backgrounds |
| `faint_bg` | `faint_bg_color` | Subtle backgrounds |
| `extreme_bg` | `extreme_bg_color` | High-contrast backgrounds |
| `panel_fill` | `panel_fill` | Panel backgrounds |
| `window_fill` | `window_fill` | Window backgrounds |
| `selection` | `selection.bg_fill` | Selection highlight |

Example mixing hex and semantic:
```yaml
styles:
  title:
    color: "#FFD700"       # fixed gold (same in both themes)
    bold: true
  danger:
    color: error           # adapts to dark/light mode
  panel:
    background: panel_fill # adapts to dark/light mode
```

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

### `theme:`

Global egui Visuals overrides. Typically placed in the root `_app.md` file for `define_litui_app!`. Generates a `__setup_theme(ctx)` function called at the start of `show_all()`.

| Field | Type | Description |
|-------|------|-------------|
| `hyperlink_color` | `"#RRGGBB"` | Link color |
| `warn_fg_color` | `"#RRGGBB"` | Warning text color |
| `error_fg_color` | `"#RRGGBB"` | Error text color |
| `code_bg_color` | `"#RRGGBB"` | Code block background |
| `panel_fill` | `"#RRGGBB"` | Panel background |
| `window_fill` | `"#RRGGBB"` | Window background |
| `faint_bg_color` | `"#RRGGBB"` | Subtle background (Grid stripes) |
| `extreme_bg_color` | `"#RRGGBB"` | TextEdit background |
| `selection_color` | `"#RRGGBB"` | Selection highlight |
| `dark` | `ThemeOverrides` | Dark-mode specific overrides |
| `light` | `ThemeOverrides` | Light-mode specific overrides |

The `dark:` and `light:` sub-sections accept the same fields. Base values apply to both themes; per-theme values override based on `visuals.dark_mode`.

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

### `nav:` (parent-level only)

Controls the auto-generated navigation bar in `define_litui_app!`.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `position` | `String` | `"top"` | `"top"`, `"bottom"`, or `"none"` |
| `show_all` | `bool` | `false` | If `true`, show ALL pages in nav including panels/windows |

- `position: "top"` — nav bar as a `TopBottomPanel::top`
- `position: "bottom"` — nav bar as a `TopBottomPanel::bottom`
- `position: "none"` — no auto nav; consumer calls `show_nav(ui)` manually

When `show_all` is `false` (default), `show_nav()` iterates `Page::NAV_PAGES` (only pages with `navigable: true` or defaulting to navigable). When `true`, iterates `Page::ALL`.

```yaml
nav:
  position: none
  show_all: false
```

---

## 2. Widget Link Syntax

```
[WIDGET_NAME#id.class1.class2](content){config}
```

- **`WIDGET_NAME`** -- one of the 19 names below (case-sensitive)
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
| Datepicker | `datepicker` | `chrono::NaiveDate` | `NaiveDate::default()` | Calendar popup; requires `egui_extras` with `datepicker` feature + `chrono` |

---

## 3. Block Directives

Opened by `::: directive [arg]` on a line by itself (inside a paragraph text event). Closed by `:::` on its own line. Can nest (stack-based).

| Directive | Syntax | Arg | State Generated | Description |
|-----------|--------|-----|-----------------|-------------|
| `foreach` | `::: foreach field` | field name | `Vec<{Field}Row>` | Iterates a collection; body must contain exactly one table or list with `{field}` references and/or widget directives |
| `if` | `::: if field` | field name | `field: bool` | Conditional rendering; auto-declares `bool` field |
| `style` | `::: style field` | field name | `field: String` | Runtime text color override; field value must be a style name from frontmatter |
| `frame` | `::: frame [style_name]` | optional style name | none | Wraps content in `egui::Frame`; resolves style from frontmatter for frame properties |
| `horizontal` | `::: horizontal [align]` | optional alignment | none | Wraps content in horizontal row; alignment: `center`, `right`, `space-between` |
| `columns` | `::: columns N` or `::: columns W:W:W` | integer or weights | none | Splits into columns; use `::: next` to advance; weights give proportional widths |
| `center` | `::: center` | none | none | Center-aligns block content |
| `right` | `::: right` | none | none | Right-aligns block content |
| `fill` | `::: fill` | none | none | Stretches content to fill available width |

### `::: next`

Column separator inside `::: columns` or `::: horizontal space-between`. Advances to the next section. Error if used outside columns or if section count exceeded.

### Alignment directives

`::: center`, `::: right`, and `::: fill` wrap their body in an egui `Layout` override:
- `center` → `Layout::top_down(Align::Center)` — block content is horizontally centered
- `right` → `Layout::top_down(Align::Max)` — block content is right-aligned
- `fill` → `Layout::top_down_justified(Align::Min)` — widgets stretch to fill available width

These nest inside any other directive.

### `::: horizontal` alignment

The `horizontal` directive accepts an optional alignment argument:
- `::: horizontal` — left-aligned (default, same as before)
- `::: horizontal center` — items centered in the row
- `::: horizontal right` — items right-aligned
- `::: horizontal space-between` — first items left, last items right; uses `::: next` to separate left/right groups

### `::: columns` weights

Columns accept either a plain integer or a colon-separated weight list:
- `::: columns 3` — three equal-width columns (uses `ui.columns()`)
- `::: columns 3:1` — two columns, 75%/25% (uses `StripBuilder` with relative sizes)
- `::: columns 3:1:1` — three columns, 60%/20%/20%
- `::: columns 1:2:1` — three columns, 25%/50%/25%

Weights are ratios: each weight is divided by the sum to get a fraction. When all weights are equal, falls back to `ui.columns()` for simplicity.

### Table column alignment (GFM standard)

Standard markdown table alignment is honored:
```markdown
| Left   | Center | Right  |
|:-------|:------:|-------:|
| text   |  text  |   text |
```

- `:---` or `---` → left-aligned (default)
- `:---:` → center-aligned
- `---:` → right-aligned

Non-left-aligned cells are wrapped in `ui.with_layout()` to apply the correct alignment.

### `foreach` body rules

- Body must contain exactly one table or one list
- Blank lines required around the table/list (CommonMark parsing requirement)
- Display `{field}` references resolve to row struct fields (`String`)
- Input widgets (checkbox, button, textedit, etc.) generate typed fields on the row struct (`bool`, `u32`, `String`, `f64`)
- Widget configs (`{cfg}`) reference the global frontmatter `widgets:` section
- Row struct name: `capitalize_first(field_name) + "Row"` (e.g., `items` -> `ItemsRow`)
- Generated struct is `pub` with `#[derive(Clone, Debug, Default)]`
- Static style suffixes (`::key`) work inside foreach; dynamic style suffixes (`::$field`) read from the row struct
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
| Conflicting field types | `define_litui_app!` field merging | `"Widget field '{name}' declared with conflicting types"` |
| `foreach` body empty | Block close (`:::`) | `"foreach body contains no {field} references"` |
| `::: columns` non-integer arg | Directive open | `"::: columns requires a number"` |
| `::: next` outside columns | Text event processing | `"::: next can only be used inside ::: columns"` |
| `::: next` exceeds column count | Column advancement | `"too many ::: next separators"` |
| Unknown widget name in link | `End(Link)` processing | `"Unknown widget name '{name}'. Valid names: ..."` |
| Unknown directive name | Directive open | `"Unknown block directive '{name}'"` |

### Frontmatter merging (`define_litui_app!` with `parent:`)

- Parent `styles:` and `widgets:` are inherited by child pages
- Child values override parent on key collision
- `spacing:` merges field-by-field (child `Some` overrides parent)
- `theme:` child overrides parent entirely (not field-merged)
- `nav:` child overrides parent entirely
- `page:` is always from the child (not inherited)
