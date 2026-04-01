# API Reference

Generated from source by `python3 scripts/generate-doc-markdown.py`.
Contains full type definitions, function signatures, and module dependencies.

---

## `crates/litui_macro/src/lib.rs`

litui — literate UI for egui.

This proc-macro crate reads `.md` files at compile time, parses them with
[`pulldown_cmark`] 0.9, and emits Rust code that calls
[`litui_helpers`] functions to render the content in egui.

# Entry Points

- [`include_litui_ui!`] -- single-file inclusion. Returns a closure
`|ui: &mut egui::Ui| { ... }` for static markdown, or a
`(fn(&mut Ui, &mut LituiFormState), LituiFormState)` tuple when stateful
widgets (slider, checkbox, etc.) are present.

- [`define_litui_app!`] -- multi-page app skeleton. Generates a `Page`
enum, per-page render functions, an `AppState` struct (if any page has
widgets), and an `LituiApp` struct with navigation and dispatch.

# Module Structure

Parsing is handled by the [`litui_parser`] crate which produces a pure-data
AST (no `TokenStream` dependencies). Code generation lives in this crate:

- `parse` -- bridge types (`ParsedMarkdown`, `WidgetField`, `WidgetType`)
that connect parser output to codegen.
- `codegen_ast` -- converts `litui_parser::ast::Document` into
`ParsedMarkdown` by walking the AST and emitting `TokenStream` code.
- `codegen` -- converts `ParsedMarkdown` into final `TokenStream` output
for both macro entry points.

# Usage

```rust,ignore
use litui_helpers::*;
use litui_macro::include_litui_ui;

// Static markdown (no widgets):
let render = include_litui_ui!("content.md");
render(ui);

// Markdown with stateful widgets:
let (render, mut state) = include_litui_ui!("form.md");
render(ui, &mut state);
```

```rust,ignore
use litui_helpers::*;
use litui_macro::define_litui_app;

define_litui_app! {
parent: "content/_app.md",
"content/about.md",
"content/form.md",
}
// Generates: Page enum, AppState, render_about(), render_form(), LituiApp
```

### Functions

#### `load_and_parse_md` (line 75)

```rust
pub(crate) fn load_and_parse_md(
    path: &str,
    parent: Option<&Frontmatter>,
    source_span: proc_macro2::Span,
) -> Result<(Frontmatter, ParsedMarkdown), proc_macro2::TokenStream>
```

Read a markdown file and parse it into structured data.
Returns (frontmatter, `parsed_markdown`) or a compile error.

Uses the new `litui_parser` for markdown → AST, then `codegen_ast` for AST → `TokenStream`.

#### `include_litui_ui` (line 117)

```rust
pub fn include_litui_ui(input: proc_macro::TokenStream) -> proc_macro::TokenStream
```

Macro to include markdown as egui UI code.

#### `define_litui_app` (line 169)

```rust
pub fn define_litui_app(input: proc_macro::TokenStream) -> proc_macro::TokenStream
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
- `LituiApp` struct with navigation and dispatch

### Module Dependencies

```rust
use crate::codegen::{define_litui_app_impl, parsed_to_include_tokens};
use crate::parse::ParsedMarkdown;
```

## `crates/litui_macro/src/parse.rs`

Bridge types for code generation.

These types bridge the `litui_parser` AST output to the codegen module.
`WidgetField` and `WidgetType` wrap the `litui_parser` equivalents with
`TokenStream` generation methods. `ParsedMarkdown` holds the generated
token output that `codegen.rs` consumes.

### Structs

#### `ParsedMarkdown` (line 111)

```rust
pub(crate) struct ParsedMarkdown {
    pub(crate) code_body: Vec<proc_macro2::TokenStream>,
    pub(crate) widget_fields: Vec<WidgetField>,
    /// True if the generated code references `state` (e.g., display widgets).
    pub(crate) references_state: bool,
    /// Field names referenced by display widgets (for validation against `AppState`).
    pub(crate) display_refs: Vec<String>,
    /// Generated style lookup function tokens (when dynamic styling is used).
    pub(crate) style_table: Option<proc_macro2::TokenStream>,
    /// Widget config keys referenced via `{key}` in widget directives.
    pub(crate) used_widget_configs: std::collections::HashSet<String>,
}
```

Structured output from parsing + codegen of a single markdown file.

Produced by `codegen_ast::document_to_parsed()`, consumed by
`codegen::parsed_to_include_tokens()` and `codegen::define_litui_app_impl()`.

### Enums

#### `WidgetType` (line 14)

```rust
pub(crate) enum WidgetType {
    F64,
    Bool,
    U32,
    Usize,
    String,
    ByteArray4,
    VecString,
    Date,
}
```

The Rust type of a widget's state field.

#### `RowField` (line 55)

```rust
pub(crate) enum RowField {
    /// `{field}` text reference — always String, display-only.
    Display(String),
    /// Widget inside foreach — typed, interactive.
    Widget { name: String, ty: WidgetType },
    /// Nested foreach — generates a child row struct + `Vec<ChildRow>`.
    Foreach {
        name: String,
        row_fields: Vec<RowField>,
        is_tree: bool,
    },
}
```

A field inside a foreach row struct (bridge type for codegen).

#### `WidgetField` (line 79)

```rust
pub(crate) enum WidgetField {
    /// Standard stateful widget (slider, checkbox, textedit, etc.)
    Stateful { name: String, ty: WidgetType },
    /// Foreach collection — generates a row struct + `Vec<RowStruct>`
    Foreach {
        name: String,
        row_fields: Vec<RowField>,
        is_tree: bool,
    },
}
```

A widget field discovered during parsing. Collected into a generated
state struct (`LituiFormState` or `AppState`).

## `crates/litui_macro/src/codegen.rs`

Code generation: `ParsedMarkdown` to final `TokenStream`.

This module contains the two code generation paths:

- [`parsed_to_include_tokens()`] -- for `include_litui_ui!`. Returns either
a closure (no stateful widgets) or a `(fn, LituiFormState)` tuple (has widgets).

- [`define_litui_app_impl()`] -- for `define_litui_app!`. Generates a
`Page` enum, optional `AppState` struct, per-page `render_*()` functions,
and a `LituiApp` struct with `show_nav()` and `show_page()` methods.

### Structs

#### `AppInput` (line 254)

```rust
struct AppInput {
    parent_path: Option<LitStr>,
    page_paths: Vec<LitStr>,
}
```

Parsed input for `define_litui_app!`: optional `parent: "path"` followed
by comma-separated page file paths.

### Functions

#### `panel_frame_tokens` (line 22)

```rust
fn panel_frame_tokens(background: &Option<String>) -> proc_macro2::TokenStream
```

Generate a `.frame(...)` call for a panel/window background.
Returns empty tokens if no background is set.

#### `generate_row_struct` (line 54)

```rust
fn generate_row_struct(
    struct_name: &str,
    row_fields: &[RowField],
    is_tree: bool,
    extra_structs: &mut Vec<proc_macro2::TokenStream>,
) -> (proc_macro2::TokenStream, syn::Ident)
```

Recursively generate a row struct from row fields.
Returns the struct TokenStream and its Ident. Nested `RowField::Foreach` entries
produce child structs appended to `extra_structs`.

#### `widget_field_tokens` (line 124)

```rust
fn widget_field_tokens(
    f: &WidgetField,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    Option<proc_macro2::TokenStream>,
)
```

Generate field definition tokens for a `WidgetField`, handling foreach row structs.
Returns (`field_def`, `field_default`, `optional_row_struct`).

#### `parsed_to_include_tokens` (line 173)

```rust
pub(crate) fn parsed_to_include_tokens(parsed: ParsedMarkdown) -> proc_macro2::TokenStream
```

Convert a [`ParsedMarkdown`] into the `TokenStream` returned by `include_litui_ui!`.

When `widget_fields` is empty, emits a simple closure `|ui: &mut egui::Ui| { ... }`.
When stateful widgets are present, emits a `LituiFormState` struct with `Default`,
a render function, and returns the tuple `(__md_render, LituiFormState::default())`.

#### `to_snake_case` (line 237)

```rust
fn to_snake_case(s: &str) -> String
```

Convert a `PascalCase` name to `snake_case`.

#### `define_litui_app_impl` (line 300)

```rust
pub(crate) fn define_litui_app_impl(
    input: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream>
```

Implementation of `define_litui_app!`.

Loads all page files (with optional parent frontmatter merging), validates
page metadata, collects widget fields across all pages, validates display
widget references, and generates:
- `Page` enum with `Default`, `ALL` const, and `label()` method
- `AppState` struct (if any page has stateful widgets)
- `render_shared(ui)` (if parent has markdown body)
- Per-page `render_{snake_name}()` functions
- `LituiApp` struct with `show_nav()` and `show_page()`

#### `show_all` (line 973)

```rust
            pub fn show_all(&mut self, ctx: &egui::Context)
```

Render all pages in their designated containers.
Side panels are always visible. Windows appear for the current page.
If any pages lack a `panel:` directive, a central panel dispatches them.
When all pages have explicit panels, no central panel is emitted.

#### `generate_theme_setup` (line 1073)

```rust
fn generate_theme_setup(theme: &Option<ThemeDef>) -> Option<proc_macro2::TokenStream>
```

Generate a `__setup_theme(ctx)` function from the frontmatter `theme:` section.

### Module Dependencies

```rust
use crate::parse::{ParsedMarkdown, RowField, WidgetField, WidgetType};
```

## `crates/litui_helpers/src/lib.rs`

Helper functions for standardized heading and text styles in egui.
Used by the litui_macro crate and consumers.

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

---

## Module Stratification

Stratification = (outgoing + 1) / (incoming + 1). Low = foundational, high = leaf.

| Module | Out | In | Strat | Role |
|--------|-----|-----|-------|------|
| `litui/lib` | 0 | 18 | 0.05 | foundation |
| `snap/lib` | 0 | 3 | 0.25 | foundation |
| `macro/parse` | 0 | 2 | 0.33 | foundation |
| `helpers/lib` | 0 | 0 | 1.00 | core |
| `macro/codegen` | 2 | 1 | 1.50 | connector |
| `macro/codegen_ast` | 1 | 0 | 2.00 | connector |
| `tut_01/main` | 1 | 0 | 2.00 | connector |
| `tut_02/main` | 1 | 0 | 2.00 | connector |
| `tut_03/main` | 1 | 0 | 2.00 | connector |
| `tut_04/main` | 1 | 0 | 2.00 | connector |
| `tut_05/main` | 1 | 0 | 2.00 | connector |
| `tut_06/main` | 1 | 0 | 2.00 | connector |
| `tut_07/main` | 1 | 0 | 2.00 | connector |
| `tut_08/main` | 1 | 0 | 2.00 | connector |
| `tut_09/main` | 1 | 0 | 2.00 | connector |
| `tut_10/main` | 1 | 0 | 2.00 | connector |
| `tut_11/main` | 1 | 0 | 2.00 | connector |
| `tut_12/main` | 1 | 0 | 2.00 | connector |
| `snap/tests/demo_pages` | 2 | 0 | 3.00 | leaf |
| `snap/tests/parent_body` | 2 | 0 | 3.00 | leaf |
| `snap/tests/snapshots` | 2 | 0 | 3.00 | leaf |
| `macro/lib` | 3 | 0 | 4.00 | leaf |

