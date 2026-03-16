# Frontmatter Style System

The macro supports YAML frontmatter for defining reusable style presets. This is an **immediate-mode GUI** feature — there is no CSS, no DOM, no runtime style lookup. The macro resolves `::key` references at compile time into literal `RichText` method chains baked into the generated code.

## Pipeline

1. `strip_frontmatter()` splits the file into YAML + markdown before pulldown-cmark sees it
2. `serde_yaml::from_str()` deserializes into `Frontmatter { page, styles, widgets }`
3. pulldown-cmark emits `::key` as literal `Text("::key")` — it's plain text, not special markup
4. At flush points (`End(Heading)`, `emit_paragraph`), `detect_style_suffix()` checks for trailing `::key` pattern
5. If found, the key is looked up in the frontmatter `styles` map
6. The `StyleDef` is resolved into a `styled_label_rich()` call with all properties as compile-time literals
7. For headings, the style merges with heading defaults (e.g., H1's 28pt size is preserved unless overridden)

## Key Functions

- `strip_frontmatter(content) -> (yaml_str, markdown)` — string scan for `---` delimiters
- `parse_hex_color("#RRGGBB") -> [u8; 3]` — compile-time hex color parsing
- `detect_style_suffix(text) -> (trimmed_text, Option<key>)` — finds trailing `::key` pattern
- `style_def_to_label_tokens(text, style, ...) -> TokenStream` — generates `styled_label_rich()` call
- `merge_style_defs(base, overlay) -> StyleDef` — merges two styles, overlay's `Some` fields override base
- `merge_frontmatter(parent, child) -> Frontmatter` — child styles/widgets override parent on collision
- `resolve_classes(classes, frontmatter) -> Option<StyleDef>` — folds class names into merged style

## Parent Frontmatter Inheritance

`define_litui_app!` supports a `parent:` keyword to specify a shared frontmatter file:

```rust
define_litui_app! {
    parent: "content/_app.md",
    "content/about.md",
    "content/form.md",
}
```

- Parent must NOT have `page:` section
- Parent must NOT contain stateful widgets
- Child styles override parent on key collision
- Widget configs are also inherited/overridable
- Optional markdown body generates `render_shared(ui)`

## ID/Class Selectors

CSS-like selectors on link text: `[button#submit.premium.large](Click_me)`

- `button` — base name (widget type or link text)
- `#submit` — ID (used as `egui::Id` via `ui.push_id()`)
- `.premium.large` — classes (reference frontmatter styles, composed left-to-right)

Classes compose via `merge_style_defs`. Last class wins on property conflicts.

Coexistence with `::key`: `.class` applies styles from frontmatter, `{config}` applies widget config only (min/max/label/format/etc.). The grammar is: `::key` = compile-time style, `{config}` = widget config, `::$field` = runtime style.

## Inline Styled Text Spans

Empty base name with classes creates a styled text fragment instead of a hyperlink:

```markdown
::accent(orange bold text)
::subtle(gray italic note)
```

The URL content becomes the display text, with class styles applied. Angle brackets are required for multi-word content (same as multi-word button labels).

## Styled Containers (Blockquotes and Lists)

The `::key` syntax also applies to blockquotes and list items. When detected, the style's `color` field is used to color the container element (quote bar or bullet/number), and the full style is applied to the text content.

```markdown
> Warning: proceed with caution. ::danger

- All systems operational ::success
- Build failed ::danger

1. First step ::success
2. Blocked step ::warning
```

Implementation: `emit_paragraph()` passes the resolved style color to `emit_quote_bars_colored()`, and `emit_list_item()` passes it to `emit_bullet_prefix_colored()` / `emit_numbered_prefix_colored()`. The `_colored` helper variants accept `Option<[u8; 3]>` and fall back to the default egui color when `None`.

## Spacing Configuration

The `spacing:` section overrides default vertical spacing values. All fields are optional:

```yaml
spacing:
  paragraph: 12.0     # after paragraphs (default 8)
  table: 12.0         # after tables (default 8)
  heading_h1: 20.0    # before H1 (default 16)
  heading_h2: 16.0    # before H2 (default 12)
  heading_h3: 10.0    # before H3 (default 8)
  heading_h4: 6.0     # before H4+ (default 4)
  item: 4.0           # ui.spacing_mut().item_spacing.y
```

Values resolve at compile time. The `item` field emits `ui.spacing_mut().item_spacing.y = X` at the start of the render function.

Spacing participates in parent/child frontmatter merging — child values override parent on field collision.

## Runtime Styles (`::$field`)

When a paragraph or list item ends with `::$field_name` (note the `$` prefix), the field is auto-declared as `String` on `AppState` and the emitted content is wrapped in a `__resolve_style_color()` override block. Set the field to a frontmatter style name at runtime to change text color.

## Panel/Window Visibility Control

`PageDef` supports an `open:` field on all panel types (`left`, `right`, `top`, `bottom`, `window`). When present, the named `bool` field is auto-declared on `AppState` (default `false`). For windows, this enables `egui::Window::open()` with an X close button. For side/top/bottom panels, the panel is hidden when the bool is `false`.

### `navigable:` field

`PageDef` also supports `navigable:` to control whether a page appears in `show_nav()`. Default: `true` for central pages (no `panel:`), `false` for panel/window pages. Override explicitly with `navigable: true` or `navigable: false`.

### `nav:` parent config

Top-level `nav:` in the parent `_app.md` controls navigation bar behavior:

```yaml
nav:
  position: top      # "top" | "bottom" | "none"
  show_all: false    # if true, show ALL pages in nav including panels/windows
```

`position: "none"` disables the auto-generated nav panel — call `show_nav(ui)` manually. `show_all: true` makes `show_nav()` iterate `Page::ALL` instead of `Page::NAV_PAGES`.

## Semantic Color Keywords

Color fields (`color`, `background`, `stroke_color`) accept either hex `#RRGGBB` or a semantic keyword referencing an egui Visuals field. Semantic colors resolve at **runtime** via `ui.visuals()` and automatically adapt to dark/light mode.

| Keyword | egui Field | Typical Use |
|---------|-----------|-------------|
| `text` | `widgets.noninteractive.fg_stroke.color` | Default text color |
| `strong` | `strong_text_color()` | Bold/heading text |
| `weak` | `weak_text_color()` | Dimmed/secondary text |
| `hyperlink` | `hyperlink_color` | Link-colored text |
| `warn` | `warn_fg_color` | Warning indicators |
| `error` | `error_fg_color` | Error indicators |
| `code_bg` | `code_bg_color` | Code block backgrounds |
| `faint_bg` | `faint_bg_color` | Subtle backgrounds |
| `extreme_bg` | `extreme_bg_color` | High-contrast backgrounds |
| `panel_fill` | `panel_fill` | Panel backgrounds |
| `window_fill` | `window_fill` | Window backgrounds |
| `selection` | `selection.bg_fill` | Selection highlight |

```yaml
styles:
  danger:
    color: error           # adapts to dark/light mode
    bold: true
  muted:
    color: weak            # dimmed in both themes
    italic: true
  panel:
    background: panel_fill # matches egui panel background
    inner_margin: 8
  custom:
    color: "#FF6B00"       # fixed — same in both themes
```

Hex literals generate `Color32::from_rgb(r, g, b)` at compile time. Semantic keywords generate `ui.visuals().{field}` at runtime. Both mix freely within the same frontmatter.

## Global Theme Configuration

The `theme:` frontmatter section customizes egui's Visuals globally. Place it in the root `_app.md` for `define_litui_app!`. Generates a `__setup_theme(ctx)` function called at the start of `show_all()`.

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

Base values apply to both themes. `dark:` and `light:` sub-sections override based on `visuals.dark_mode`. Fields: `hyperlink_color`, `warn_fg_color`, `error_fg_color`, `code_bg_color`, `panel_fill`, `window_fill`, `faint_bg_color`, `extreme_bg_color`, `selection_color`.

Semantic color keywords in styles reference the active Visuals — if you customize `panel_fill` via `theme:`, a style with `background: panel_fill` picks up the custom value.

## Why Some Colors Are Compile-Time

egui rebuilds the entire UI every frame. There's no retained style state. Hex colors like `{ color: "#FF6B00" }` become `egui::Color32::from_rgb(255, 107, 0)` in generated code — zero runtime cost. Semantic keywords like `{ color: strong }` become `ui.visuals().strong_text_color()` — one field access per frame, also near-zero cost.
