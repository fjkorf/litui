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

`define_markdown_app!` supports a `parent:` keyword to specify a shared frontmatter file:

```rust
define_markdown_app! {
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

## Window Visibility Control

`PageDef` supports an `open:` field for `panel: window` pages. When present, the named `bool` field is auto-declared on `AppState`, and the window renders with `egui::Window::open()` for state-driven visibility with an X close button.

## Why No Runtime Lookup

egui rebuilds the entire UI every frame. There's no retained style state. The macro must emit all styling as literal code. A frontmatter style `{ color: "#FF6B00" }` becomes `egui::Color32::from_rgb(255, 107, 0)` in the generated Rust code.
