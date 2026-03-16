# API Reference

Generated from source by `python3 scripts/generate-doc-markdown.py`.
Contains full type definitions, function signatures, and module dependencies.

---

## `crates/markdown_to_egui_macro/src/lib.rs`

litui — literate UI for egui.

This proc-macro crate reads `.md` files at compile time, parses them with
[`pulldown_cmark`] 0.9, and emits Rust code that calls
[`markdown_to_egui_helpers`] functions to render the content in egui.

# Entry Points

- [`include_markdown_ui!`] -- single-file inclusion. Returns a closure
`|ui: &mut egui::Ui| { ... }` for static markdown, or a
`(fn(&mut Ui, &mut MdFormState), MdFormState)` tuple when stateful
widgets (slider, checkbox, etc.) are present.

- [`define_markdown_app!`] -- multi-page app skeleton. Generates a `Page`
enum, per-page render functions, an `AppState` struct (if any page has
widgets), and an `MdApp` struct with navigation and dispatch.

# Module Structure

- `frontmatter` -- YAML frontmatter types (`Frontmatter`, `StyleDef`,
`WidgetDef`, `PageDef`), parsing, parent/child merging, CSS-like selector
parsing, hex color parsing, and `{key}` detection.
- `parse` -- pulldown-cmark event loop that converts markdown into
`ParsedMarkdown` (accumulated `TokenStream` fragments + widget fields).
- `codegen` -- converts `ParsedMarkdown` into final `TokenStream` output
for both macro entry points.

# Usage

```rust,ignore
use markdown_to_egui_helpers::*;
use markdown_to_egui_macro::include_markdown_ui;

// Static markdown (no widgets):
let render = include_markdown_ui!("content.md");
render(ui);

// Markdown with stateful widgets:
let (render, mut state) = include_markdown_ui!("form.md");
render(ui, &mut state);
```

```rust,ignore
use markdown_to_egui_helpers::*;
use markdown_to_egui_macro::define_markdown_app;

define_markdown_app! {
parent: "content/_app.md",
"content/about.md",
"content/form.md",
}
// Generates: Page enum, AppState, render_about(), render_form(), MdApp
```

### Functions

#### `load_and_parse_md` (line 69)

```rust
pub(crate) fn load_and_parse_md(
    path: &str,
    parent: Option<&Frontmatter>,
) -> Result<(Frontmatter, ParsedMarkdown), proc_macro2::TokenStream>
```

Read a markdown file and parse it into structured data.
Returns (frontmatter, parsed_markdown) or a compile error.

#### `include_markdown_ui` (line 110)

```rust
pub fn include_markdown_ui(input: proc_macro::TokenStream) -> proc_macro::TokenStream
```

Macro to include markdown as egui UI code.

#### `define_markdown_app` (line 140)

```rust
pub fn define_markdown_app(input: proc_macro::TokenStream) -> proc_macro::TokenStream
```

Item-position macro that generates a full app skeleton from markdown files.

Each `.md` file must include a `page:` section in its YAML frontmatter:
```yaml
---
page:
name: About
label: About
default: true
---
```

Generates:
- `Page` enum with one variant per file
- State structs for pages with stateful widgets (named `{PageName}State`)
- Render functions (`render_{snake_name}`)
- `MdApp` struct with navigation and dispatch

### Module Dependencies

```rust
use crate::codegen::{define_markdown_app_impl, parsed_to_include_tokens};
use crate::frontmatter::{Frontmatter, merge_frontmatter, strip_frontmatter};
use crate::parse::{ParsedMarkdown, markdown_to_egui};
```

## `crates/markdown_to_egui_macro/src/frontmatter.rs`

YAML frontmatter parsing and style resolution.

Markdown files may begin with a `---`-delimited YAML block that defines
reusable style presets, widget configurations, and page metadata. This
module handles the full pipeline:

1. [`strip_frontmatter()`] splits raw file content into YAML + markdown
(must happen before pulldown-cmark sees the file, since `---` would
be parsed as `ThematicBreak`).
2. `serde_yaml` deserializes the YAML into a [`Frontmatter`] struct.
3. [`merge_frontmatter()`] merges a parent's styles/widgets into a child
(used by `define_markdown_app!` with the `parent:` keyword).
4. At flush points in the event loop, [`detect_style_suffix()`] checks for
trailing `{key}` references, and [`style_def_to_label_tokens()`]
emits the corresponding `styled_label_rich()` call.

See `knowledge/frontmatter-and-styles.md` for the full style system design.

### Structs

#### `Frontmatter` (line 32)

```rust
pub(crate) struct Frontmatter {
    #[serde(default)]
    pub(crate) page: Option<PageDef>,
    #[serde(default)]
    pub(crate) styles: HashMap<String, StyleDef>,
    #[serde(default)]
    pub(crate) widgets: HashMap<String, WidgetDef>,
}
```

Top-level frontmatter deserialized from the YAML block at the start of a `.md` file.

Contains optional page metadata (for `define_markdown_app!`), a map of named
style presets referenced via `::key` or `.class` in the markdown body, and a
map of widget configurations referenced via `{config}` after widget directives.

#### `PageDef` (line 47)

```rust
pub(crate) struct PageDef {
    pub(crate) name: String,
    pub(crate) label: String,
    #[serde(default)]
    pub(crate) default: bool,
    /// Container type: "left", "right", "top", "bottom", "window", or absent (central panel).
    #[serde(default)]
    pub(crate) panel: Option<String>,
    /// Default width for side panels or windows.
    #[serde(default)]
    pub(crate) width: Option<f32>,
    /// Default height for top/bottom panels or windows.
    #[serde(default)]
    pub(crate) height: Option<f32>,
}
```

Page metadata from the `page:` section of frontmatter.

Required for each file passed to `define_markdown_app!`. Provides the
enum variant name, the UI label for navigation, and whether this page
is the default (exactly one page must set `default: true`).

#### `WidgetDef` (line 73)

```rust
pub(crate) struct WidgetDef {
    pub(crate) min: Option<f64>,
    pub(crate) max: Option<f64>,
    pub(crate) speed: Option<f64>,
    pub(crate) label: Option<String>,
    pub(crate) hint: Option<String>,
    /// Format string for display widgets (e.g., `"{:.1}"`)
    pub(crate) format: Option<String>,
    /// Options list for radio/combobox widgets
    pub(crate) options: Option<Vec<String>>,
    /// Track hover state for buttons (generates `{name}_hovered: bool` field)
    pub(crate) track_hover: Option<bool>,
    /// Track secondary click for buttons (generates `{name}_secondary_count: u32` field)
    pub(crate) track_secondary: Option<bool>,
    /// Tooltip text shown on hover for any widget
    pub(crate) tooltip: Option<String>,
    /// Suffix appended to slider display (e.g., `"°"`)
    pub(crate) suffix: Option<String>,
    /// Prefix prepended to slider display (e.g., `"$"`)
    pub(crate) prefix: Option<String>,
    /// Desired row count for textarea widgets
    pub(crate) rows: Option<usize>,
    /// Max height in pixels for scrollable select widgets
    pub(crate) max_height: Option<f64>,
    /// Fill color for progress bars (e.g., `"#8B0000"`)
    pub(crate) fill: Option<String>,
}
```

Widget-specific configuration from the `widgets:` section of frontmatter.

Referenced by `{key}` after a widget directive (e.g., `[slider](volume){vol}`).
Not all fields apply to every widget type:
- `min`/`max` -- slider, double_slider, dragvalue range bounds
- `speed` -- dragvalue drag sensitivity
- `label` -- slider/checkbox display label
- `hint` -- textedit placeholder hint text
- `format` -- display widget format string (e.g., `"{:.1}"`)

#### `StyleDef` (line 112)

```rust
pub(crate) struct StyleDef {
    pub(crate) bold: Option<bool>,
    pub(crate) italic: Option<bool>,
    pub(crate) strikethrough: Option<bool>,
    pub(crate) underline: Option<bool>,
    pub(crate) color: Option<String>,
    pub(crate) background: Option<String>,
    pub(crate) size: Option<f32>,
    pub(crate) monospace: Option<bool>,
    pub(crate) weak: Option<bool>,
}
```

A named style preset that controls how text is rendered.

Each field is `Option` so that styles can be composed via [`merge_style_defs()`]:
an overlay's `Some` values override the base, while `None` inherits.

- `bold`/`italic`/`strikethrough`/`underline` -- text decoration flags
- `color`/`background` -- hex color strings (`"#RRGGBB"`), parsed at compile time
- `size` -- font size in points (overrides heading defaults when applied to headings)
- `monospace` -- use monospace font instead of body font
- `weak` -- render with weak (dimmed) text color

#### `ParsedSelector` (line 170)

```rust
pub(crate) struct ParsedSelector {
    pub(crate) base_name: String,
    pub(crate) id: Option<String>,
    pub(crate) classes: Vec<String>,
}
```

Result of parsing CSS-like selectors from link text.

Link text like `"button#submit.premium.large"` is split into:
- `base_name` -- `"button"` (the widget type or link display text)
- `id` -- `Some("submit")` (used as `egui::Id` via `ui.push_id()`)
- `classes` -- `["premium", "large"]` (resolved against frontmatter styles,
composed left-to-right via [`merge_style_defs()`])

### Functions

#### `merge_frontmatter` (line 127)

```rust
pub(crate) fn merge_frontmatter(parent: &Frontmatter, child: Frontmatter) -> Frontmatter
```

Merge parent and child frontmatter. Child values override parent on key collision.

#### `merge_style_defs` (line 144)

```rust
pub(crate) fn merge_style_defs(base: &StyleDef, overlay: &StyleDef) -> StyleDef
```

Merge two StyleDefs. Overlay's `Some` fields override base.

#### `parse_selectors` (line 178)

```rust
pub(crate) fn parse_selectors(link_text: &str) -> ParsedSelector
```

Parse CSS-like selectors from link text: `"button#id.class1.class2"` →
`ParsedSelector { base_name: "button", id: Some("id"), classes: ["class1", "class2"] }`

#### `resolve_classes` (line 223)

```rust
pub(crate) fn resolve_classes(classes: &[String], frontmatter: &Frontmatter) -> Option<StyleDef>
```

Resolve class names into a merged StyleDef. Panics at compile time on undefined classes.

#### `strip_frontmatter` (line 240)

```rust
pub(crate) fn strip_frontmatter(content: &str) -> (&str, &str)
```

Split content into (yaml_frontmatter, remaining_markdown).
Returns ("", content) if no frontmatter is present.

#### `parse_hex_color` (line 277)

```rust
pub(crate) fn parse_hex_color(s: &str) -> Result<[u8; 3], String>
```

Parse "#RRGGBB" hex color to [r, g, b].

#### `detect_style_suffix` (line 292)

```rust
pub(crate) fn detect_style_suffix(text: &str) -> (&str, Option<&str>)
```

Check if text ends with a `::key` style suffix. Returns (trimmed_text, Some(key)) or (text, None).
The key may start with `$` for runtime style references (e.g., `::$hp_style`).

#### `style_def_to_label_tokens` (line 309)

```rust
pub(crate) fn style_def_to_label_tokens(
    text: &str,
    style: &StyleDef,
    base_bold: bool,
    base_italic: bool,
    base_strikethrough: bool,
) -> proc_macro2::TokenStream
```

Emit tokens for a `styled_label_rich()` call using a resolved `StyleDef`.

## `crates/markdown_to_egui_macro/src/parse.rs`

Markdown event loop: pulldown-cmark events to `ParsedMarkdown`.

This module walks the pulldown-cmark 0.9 event stream and accumulates
[`Fragment`]s (styled text, inline code, links, widgets) which are flushed
into generated `TokenStream` code at block boundaries.

Key design choices:
- **Index-based iteration** (`while event_idx < events.len()`) instead of
`for event in events`, enabling one-event lookahead for widget `{config}`
consumption.
- **Fragment accumulation** -- inline content is buffered as [`Fragment`]
values, then flushed at flush points into `ui.horizontal_wrapped(|ui| { ... })`
calls.
- **Flush points**: `End(Paragraph)`, `End(Item)` (tight list fallback),
`Start(List)` (parent item before nesting), `End(Heading)`, `End(CodeBlock)`,
`End(TableCell)`.
- **Table-aware widget emission**: widgets inside table cells are pushed to
`fragments` as `Fragment::Widget` so they render inside the `egui::Grid`
closure; outside tables they go directly to `code_body`.

See `knowledge/pulldown-cmark-0.9.md` for the event model and
`knowledge/proc-macro-architecture.md` for the fragment accumulation pattern.

### Structs

#### `BlockFrame` (line 144)

```rust
struct BlockFrame {
    directive: BlockDirective,
    field_name: String,
    saved_code_body: Vec<proc_macro2::TokenStream>,
}
```

A stack frame for a `:::` block directive.

#### `ParsedMarkdown` (line 153)

```rust
pub(crate) struct ParsedMarkdown {
    pub(crate) code_body: Vec<proc_macro2::TokenStream>,
    pub(crate) widget_fields: Vec<WidgetField>,
    /// True if the generated code references `state` (e.g., display widgets).
    pub(crate) references_state: bool,
    /// Field names referenced by display widgets (for validation against AppState).
    pub(crate) display_refs: Vec<String>,
    /// Generated style lookup function tokens (when dynamic styling is used).
    pub(crate) style_table: Option<proc_macro2::TokenStream>,
}
```

Structured output from parsing a single markdown file.

### Enums

#### `WidgetType` (line 70)

```rust
pub(crate) enum WidgetType {
    F64,
    Bool,
    U32,
    Usize,
    String,
    ByteArray4,
    VecString,
}
```

The Rust type of a widget's state field.

#### `WidgetField` (line 109)

```rust
pub(crate) enum WidgetField {
    /// Standard stateful widget (slider, checkbox, textedit, etc.)
    Stateful { name: String, ty: WidgetType },
    /// Foreach collection — generates a row struct + `Vec<RowStruct>`
    Foreach {
        name: String,
        row_fields: Vec<String>,
    },
}
```

A widget field discovered during parsing. Collected into a generated
state struct (`MdFormState` or `AppState`).

#### `BlockDirective` (line 137)

```rust
enum BlockDirective {
    Foreach { row_fields: Vec<String> },
    If,
    Style,
}
```

The type of block directive opened by `:::`.

### Functions

#### `parse_inline_styled_spans` (line 177)

```rust
fn parse_inline_styled_spans(
    text: &str,
    fragments: &mut Vec<Fragment>,
    frontmatter: &Frontmatter,
    bold: bool,
    italic: bool,
    strikethrough: bool,
) -> bool
```

Scan text for `::class(text)` inline styled span patterns.
Splits into alternating Styled and Widget (styled_label_rich) fragments.
Returns true if any spans were found and processed.

#### `markdown_to_egui` (line 263)

```rust
pub(crate) fn markdown_to_egui(content: &str, frontmatter: &Frontmatter) -> ParsedMarkdown
```

Parse markdown content into a [`ParsedMarkdown`] using the pulldown-cmark event loop.

This is the core of the macro: it walks every pulldown-cmark event, tracks
inline style state (bold/italic/strikethrough), list nesting, blockquote depth,
table structure, and widget directives. Text fragments are accumulated and
flushed into `TokenStream` code at block boundaries.

The `frontmatter` parameter provides style and widget definitions for
`::key` / `.class` resolution during code generation.

#### `emit_paragraph` (line 365)

```rust
    fn emit_paragraph(
        fragments: &mut Vec<Fragment>,
        code_body: &mut Vec<proc_macro2::TokenStream>,
        blockquote_depth: usize,
        frontmatter: &Frontmatter,
    )
```

Emit accumulated fragments as an inline-wrapped paragraph.
If the last fragment ends with `::key`, apply the frontmatter style to all fragments.

#### `emit_list_item` (line 480)

```rust
    fn emit_list_item(
        fragments: &mut Vec<Fragment>,
        code_body: &mut Vec<proc_macro2::TokenStream>,
        list_stack: &mut [Option<usize>],
        blockquote_depth: usize,
        frontmatter: &Frontmatter,
    )
```

Emit accumulated fragments as a list item (bullet or numbered).
Each item is a separate top-level `ui.horizontal_wrapped(...)`.
If the last fragment ends with `::key`, the style's color is applied to the
bullet/number prefix and all text fragments get the full style treatment.

#### `parse_foreach_text` (line 708)

```rust
    fn parse_foreach_text(
        text: &str,
        fragments: &mut Vec<Fragment>,
        row_fields: &mut Vec<String>,
        bold: bool,
        italic: bool,
        strikethrough: bool,
    )
```

Parse text containing `{field}` references into alternating Styled and ForeachField
fragments. Used inside foreach blocks for field substitution.

### Constants

#### `WIDGET_NAMES` (line 759)

```rust
const WIDGET_NAMES: &[&str] = &[
        "button",
        "progress",
        "spinner",
        "slider",
        "double_slider",
        "checkbox",
        "textedit",
        "textarea",
        "password",
        "dragvalue",
        "display",
        "radio",
        "combobox",
        "color",
        "toggle",
        "selectable",
        "select",
        "log",
    ];
```

Known widget names that intercept link syntax.
To add a new widget: add its name here, then add a match arm in the
`End(Link)` handler below.

#### `OPTIONS` (line 1474)

```rust
const OPTIONS: &[&str] = &[#(#options),*];
```


### Module Dependencies

```rust
use crate::frontmatter::{ Frontmatter, WidgetDef, detect_style_suffix, parse_hex_color, parse_selectors, resolve_classes, style_def_to_label_tokens, };
```

## `crates/markdown_to_egui_macro/src/codegen.rs`

Code generation: `ParsedMarkdown` to final `TokenStream`.

This module contains the two code generation paths:

- [`parsed_to_include_tokens()`] -- for `include_markdown_ui!`. Returns either
a closure (no stateful widgets) or a `(fn, MdFormState)` tuple (has widgets).

- [`define_markdown_app_impl()`] -- for `define_markdown_app!`. Generates a
`Page` enum, optional `AppState` struct, per-page `render_*()` functions,
and an `MdApp` struct with `show_nav()` and `show_page()` methods.

### Structs

#### `AppInput` (line 168)

```rust
struct AppInput {
    parent_path: Option<LitStr>,
    page_paths: Vec<LitStr>,
}
```

Parsed input for `define_markdown_app!`: optional `parent: "path"` followed
by comma-separated page file paths.

### Functions

#### `widget_field_tokens` (line 20)

```rust
fn widget_field_tokens(
    f: &WidgetField,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    Option<proc_macro2::TokenStream>,
)
```

Generate field definition tokens for a WidgetField, handling foreach row structs.
Returns (field_def, field_default, optional_row_struct).

#### `parsed_to_include_tokens` (line 88)

```rust
pub(crate) fn parsed_to_include_tokens(parsed: ParsedMarkdown) -> proc_macro2::TokenStream
```

Convert a [`ParsedMarkdown`] into the `TokenStream` returned by `include_markdown_ui!`.

When `widget_fields` is empty, emits a simple closure `|ui: &mut egui::Ui| { ... }`.
When stateful widgets are present, emits a `MdFormState` struct with `Default`,
a render function, and returns the tuple `(__md_render, MdFormState::default())`.

#### `to_snake_case` (line 151)

```rust
fn to_snake_case(s: &str) -> String
```

Convert a PascalCase name to snake_case.

#### `define_markdown_app_impl` (line 214)

```rust
pub(crate) fn define_markdown_app_impl(
    input: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream>
```

Implementation of `define_markdown_app!`.

Loads all page files (with optional parent frontmatter merging), validates
page metadata, collects widget fields across all pages, validates display
widget references, and generates:
- `Page` enum with `Default`, `ALL` const, and `label()` method
- `AppState` struct (if any page has stateful widgets)
- `render_shared(ui)` (if parent has markdown body)
- Per-page `render_{snake_name}()` functions
- `MdApp` struct with `show_nav()` and `show_page()`

#### `show_all` (line 670)

```rust
            pub fn show_all(&mut self, ctx: &egui::Context)
```

Render all pages in their designated containers.
Side panels are always visible. Windows appear for the current page.
Central panel pages use standard page dispatch.

### Module Dependencies

```rust
use crate::frontmatter::{Frontmatter, PageDef};
use crate::parse::{ParsedMarkdown, WidgetField, WidgetType, capitalize_first};
```

## `crates/markdown_to_egui_helpers/src/lib.rs`

Helper functions for standardized heading and text styles in egui.
Used by the markdown_to_egui_macro crate and consumers.

The macro generates a closure whose body runs inside a single
`left_to_right` wrapping layout (the easy_mark pattern). These helpers
emit widgets into that flow: `ui.label()` for inline text,
`ui.end_row()` for line breaks, and `allocate_exact_size()` for
indentation prefixes (bullets, numbers, quote bars).

### Structs

#### `StyleContext` (line 14)

```rust
pub struct StyleContext {
    pub heading_level: Option<u8>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub code: bool,
    pub small: bool,
    pub raised: bool,
    pub weak: bool,
    pub blockquote: bool,
    pub link: Option<String>,
    pub list_number: Option<usize>,
    pub bullet: bool,
}
```

Represents the current style context for markdown rendering.

### Functions

#### `h1` (line 33)

```rust
pub fn h1(ui: &mut Ui, text: &str)
```

Render an H1 heading (28pt, bold, strong color) as a block-level element.

#### `h2` (line 46)

```rust
pub fn h2(ui: &mut Ui, text: &str)
```

Render an H2 heading (22pt, bold) as a block-level element.

#### `h3` (line 59)

```rust
pub fn h3(ui: &mut Ui, text: &str)
```

Render an H3 heading (18pt, bold) as a block-level element.

#### `body` (line 72)

```rust
pub fn body(ui: &mut Ui, text: &str)
```

Render body text with the default body text style.

#### `code` (line 77)

```rust
pub fn code(ui: &mut Ui, text: &str)
```

Block-level fenced code.

#### `hyperlink` (line 93)

```rust
pub fn hyperlink(ui: &mut Ui, text: &str, url: &str)
```

Render a clickable hyperlink with underline and hyperlink color.

#### `separator` (line 103)

```rust
pub fn separator(ui: &mut Ui)
```

Render a horizontal separator as a block-level element.

#### `bullet_point` (line 112)

```rust
pub fn bullet_point(ui: &mut Ui, text: &str)
```

Legacy: render a bullet point with single-level indent. Prefer `emit_bullet_prefix`.

#### `numbered_point` (line 127)

```rust
pub fn numbered_point(ui: &mut Ui, number: &str, text: &str)
```

Legacy: render a numbered list item with single-level indent. Prefer `emit_numbered_prefix`.

#### `quote_indent` (line 147)

```rust
pub fn quote_indent(ui: &mut Ui, text: &str)
```

Legacy: render a blockquote with single-level indent. Prefer `emit_quote_bars`.

#### `italic` (line 160)

```rust
pub fn italic(ui: &mut Ui, text: &str)
```

Legacy: render italic text. Prefer `styled_label`.

#### `underline` (line 165)

```rust
pub fn underline(ui: &mut Ui, text: &str)
```

Legacy: render underlined text. Prefer `styled_label_rich`.

#### `strikethrough` (line 170)

```rust
pub fn strikethrough(ui: &mut Ui, text: &str)
```

Legacy: render strikethrough text. Prefer `styled_label`.

#### `small` (line 179)

```rust
pub fn small(ui: &mut Ui, text: &str)
```

Legacy: render small text. Prefer `styled_label_rich` with a size override.

#### `raised` (line 184)

```rust
pub fn raised(ui: &mut Ui, text: &str)
```

Legacy: render raised (superscript-like) text.

#### `weak` (line 189)

```rust
pub fn weak(ui: &mut Ui, text: &str)
```

Legacy: render weak (dimmed) text. Prefer `styled_label_rich` with `weak: true`.

#### `styled_label` (line 196)

```rust
pub fn styled_label(ui: &mut Ui, text: &str, bold: bool, is_italic: bool, is_strikethrough: bool)
```

Inline text with composable style flags.

#### `styled_hyperlink` (line 211)

```rust
pub fn styled_hyperlink(
    ui: &mut Ui,
    text: &str,
    url: &str,
    bold: bool,
    is_italic: bool,
    is_strikethrough: bool,
)
```

Clickable hyperlink with composable inline styles.

#### `inline_code` (line 235)

```rust
pub fn inline_code(ui: &mut Ui, text: &str)
```

Inline monospace code (not block-level).

#### `styled_label_rich` (line 246)

```rust
pub fn styled_label_rich(
    ui: &mut Ui,
    text: &str,
    bold: bool,
    is_italic: bool,
    is_strikethrough: bool,
    is_underline: bool,
    color: Option<[u8; 3]>,
    background: Option<[u8; 3]>,
    size: Option<f32>,
    monospace: bool,
    is_weak: bool,
)
```

Fully parameterized label for frontmatter-styled text.
Color/background are `Option<[u8; 3]>` RGB tuples resolved at compile time.

#### `end_paragraph` (line 297)

```rust
pub fn end_paragraph(ui: &mut Ui)
```

End the current row and add paragraph spacing.

#### `emit_quote_bars` (line 304)

```rust
pub fn emit_quote_bars(ui: &mut Ui, depth: usize)
```

Emit blockquote vertical bars for `depth` levels at the start of a row.

#### `emit_quote_bars_colored` (line 309)

```rust
pub fn emit_quote_bars_colored(ui: &mut Ui, depth: usize, bar_color_override: Option<[u8; 3]>)
```

Emit blockquote vertical bars with an optional custom color.

#### `emit_bullet_prefix` (line 328)

```rust
pub fn emit_bullet_prefix(ui: &mut Ui, depth: usize)
```

Emit a bullet prefix with depth-based indentation.

#### `emit_bullet_prefix_colored` (line 333)

```rust
pub fn emit_bullet_prefix_colored(ui: &mut Ui, depth: usize, color_override: Option<[u8; 3]>)
```

Emit a bullet prefix with an optional custom color.

#### `emit_numbered_prefix` (line 348)

```rust
pub fn emit_numbered_prefix(ui: &mut Ui, depth: usize, number: &str)
```

Emit a numbered prefix with depth-based indentation.

#### `emit_numbered_prefix_colored` (line 353)

```rust
pub fn emit_numbered_prefix_colored(
    ui: &mut Ui,
    depth: usize,
    number: &str,
    color_override: Option<[u8; 3]>,
)
```

Emit a numbered prefix with an optional custom color.

#### `toggle_switch` (line 387)

```rust
pub fn toggle_switch(ui: &mut Ui, on: &mut bool) -> eframe::egui::Response
```

iOS-style toggle switch widget. Click to flip the boolean.

## `examples/demo_content/src/lib.rs`

Shared demo content for the eframe and Bevy demo apps.

Contains the 7-page markdown demo generated by `define_markdown_app!`,
plus the `auto_unmute` ECS system. Both `eframe_demo` and `bevy_demo`
depend on this crate for the generated types and render functions.

### Functions

#### `auto_unmute` (line 32)

```rust
pub fn auto_unmute(mut state: ResMut<'_, AppState>)
```

ECS system: automatically uncheck "muted" when volume exceeds 80%.

