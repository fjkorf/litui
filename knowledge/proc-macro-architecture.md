# Proc-Macro Architecture

## Module Layout

Parsing lives in a standalone crate (`litui_parser`) with no proc-macro dependencies, enabling independent testing. Code generation lives in the macro crate.

```
crates/litui_parser/src/
├── lib.rs           — module declarations
├── ast.rs           — pure-data AST types (Inline, Block, Document, WidgetDirective, etc.)
├── frontmatter.rs   — Frontmatter/StyleDef/WidgetDef/ThemeDef types, YAML parsing,
│                      style merging, selector parsing, hex colors, semantic colors
├── parse.rs         — parse_document(): pulldown-cmark event loop → Document AST
└── error.rs         — ParseError type (no proc-macro2 Span)

crates/litui_macro/src/
├── lib.rs           — #[proc_macro] entry points, load_and_parse_md
├── parse.rs         — Bridge types (ParsedMarkdown, WidgetField, WidgetType with to_tokens())
├── codegen_ast.rs   — Document AST → ParsedMarkdown (TokenStream generation)
└── codegen.rs       — parsed_to_include_tokens, define_litui_app_impl,
                       AppInput parser, to_snake_case
```

All items are `pub(crate)` except the two `#[proc_macro]` functions in `lib.rs`.

## Pipeline

```
.md file → strip_frontmatter() → YAML + markdown
         → serde_yaml::from_str() → Frontmatter
         → litui_parser::parse_document(markdown, frontmatter) → Document (AST)
         → codegen_ast::document_to_parsed(doc, frontmatter) → ParsedMarkdown (TokenStream)
         → codegen::parsed_to_include_tokens(parsed) → final TokenStream
```

## AST Types (litui_parser::ast)

The AST is pure data — no `TokenStream`, `quote`, `syn`, or `proc-macro2` dependencies.

### Inline (within paragraphs, cells, list items)

```rust
enum Inline {
    Text { text, bold, italic, strikethrough },
    InlineCode(String),
    Link { text, url, bold, italic, strikethrough },
    Image { alt, url },
    StyledSpan { class, text, bold, italic, strikethrough },  // ::class(text)
    ClassSpan { classes, text, bold, italic, strikethrough },  // [.class](text)
    Widget(WidgetDirective),
    ForeachField(String),
}
```

### Block (document-level nodes)

```rust
enum Block {
    Heading { level, text, style_suffix },
    Paragraph { fragments: Vec<Inline>, style_suffix },
    List { kind, items: Vec<ListItem> },
    Table { headers, rows, num_columns, table_index },
    CodeBlock { text },
    BlockQuote { depth, blocks: Vec<Block> },
    HorizontalRule,
    Directive(Directive),  // foreach, if, style, frame, horizontal, columns
    Image { alt, url },
    Widget(WidgetDirective),
    Spacing(f32),
    ItemSpacingOverride(f32),
}
```

### ListItem

Each item carries its nesting `depth` (1 = top-level) and `kind` (ordered/unordered with number) for correct prefix rendering. Items are stored flat (not nested trees) — the depth field provides indentation.

### WidgetDirective

```rust
struct WidgetDirective {
    widget_type: WidgetKind,  // 18 variants: Button, Slider, Display, etc.
    field: String,             // URL portion (field name or literal)
    id: Option<String>,        // #id selector
    classes: Vec<String>,      // .class selectors
    config_key: String,        // {key} config reference
}
```

## What the Macro Generates

The macro resolves the `.md` file path at compile time, parses it with pulldown-cmark (via litui_parser), and emits a Rust closure. The generated code looks like:

```rust
|ui: &mut egui::Ui| {
    // Each paragraph -> ui.horizontal_wrapped(|ui| { styled_label(ui, ...); });
    // Each list item -> ui.horizontal_wrapped(|ui| { emit_bullet_prefix(ui, depth); styled_label(ui, ...); });
    // Each heading -> h1(ui, "text");
    // Each code block -> code(ui, "text");
}
```

## Key Constraint: Proc-Macro Cannot Use egui Types

The macro crate is `proc-macro = true`. It can only emit `TokenStream` — it cannot import or use `egui::Ui`, `RichText`, etc. at compile time. All egui types are referenced by name in the generated code and resolved by the consumer crate.

## Fragment Accumulation Pattern

The parser accumulates text fragments at compile time, then flushes them at block boundaries:

1. `Text("some ")` -> appends to `pending_text`
2. `Start(Strong)` -> flushes `pending_text` as `Inline::Text{bold:false,...}`, sets `bold = true`
3. `Text("bold")` -> appends to `pending_text`
4. `End(Strong)` -> flushes `pending_text` as `Inline::Text{bold:true,...}`, sets `bold = false`
5. `End(Paragraph)` -> flushes remaining `pending_text`, creates `Block::Paragraph` with all `Inline` fragments

The codegen then converts each `Block::Paragraph` into `ui.horizontal_wrapped(|ui| { styled_label(ui, text, bold, italic, strikethrough); ... })` calls.

Style flags (`bold`, `italic`, `strikethrough`) are compile-time booleans baked into the generated code as literal `true`/`false` arguments.

## Flush Points (Where Fragments Become AST Nodes)

- `End(Paragraph)` — creates `Block::Paragraph` or appends to list
- `End(Item)` — fallback flush for tight lists (no Paragraph wrapper)
- `Start(List)` — flushes parent item text before nesting (tight list fix)
- `End(Heading)` — creates `Block::Heading` with accumulated text
- `End(CodeBlock)` — creates `Block::CodeBlock`
- `End(Link)` — creates `Inline::Widget` for widget directives, `Inline::Link` for hyperlinks, `Inline::ClassSpan` for styled spans
- `End(Image)` — creates `Block::Image` or `Inline::Image` (table context)

## egui Layout Patterns

### `ui.horizontal_wrapped(|ui| { ... })`

Used for paragraph content. Widgets inside flow left-to-right and wrap to new lines when width is exceeded. Each call creates its own row in the parent vertical layout.

### Row-prefix pattern for list items

Each list item is its own `ui.horizontal_wrapped(|ui| { ... })` call. The prefix (bullet circle or number) is drawn via `allocate_exact_size` + painter calls at the start of the row.

```rust
ui.horizontal_wrapped(|ui| {
    emit_bullet_prefix(ui, depth);
    styled_label(ui, "item text", false, false, false);
});
```

### Block-level elements

Headings, code blocks, and separators break out of inline flow. They call `ui.end_row()` before and after to ensure they occupy their own vertical space.

## Table-Aware Widget Emission

Widgets inside table cells are stored as `Inline::Widget` in the cell's fragment list, ensuring they render inside the `egui::Grid` closure. Outside tables, widgets become `Block::Widget` and render at the top level.

## `End(Link)` Processing Flow

The link handler is the most complex event handler. It processes widgets, styled text spans, and regular hyperlinks:

1. Parse selectors from link text: `parse_selectors("button#id.class")` -> `ParsedSelector { base_name, id, classes }`
2. If `base_name` is a known widget name:
   - Lookahead for `{config}` text event
   - Record widget field in `widget_fields` based on type
   - Create `WidgetDirective` with parsed name/id/classes/config
   - Push as `Inline::Widget` (table) or `Block::Widget` (non-table)
3. If `base_name` is empty and classes exist: create `Inline::ClassSpan` (styled inline text)
4. Otherwise: create `Inline::Link` (regular hyperlink), applying class bold/italic/strikethrough

## `define_litui_app!` Code Generation

The multi-page macro generates several interconnected types:

1. **`AppInput` parsing**: optional `parent: "path"` keyword + comma-separated page paths
2. **Parent loading**: if specified, load parent `.md`, validate no `page:` section, merge frontmatter into each child
3. **Widget field collection**: gather all `WidgetField`s across all pages into a flat list, check for name/type collisions. `WidgetField` is an enum: `Stateful { name, ty: WidgetType }` for regular widgets, `Foreach { name, row_fields }` for collection iteration. `WidgetType` is an enum (`F64`, `Bool`, `U32`, `Usize`, `String`, `ByteArray4`, `VecString`) with `to_tokens()` and `default_tokens()` methods. Same-name fields across pages are allowed if types match; conflicting types error.
4. **Display widget validation**: `[display]` self-declares fields as `String` if no input widget declares them. Verify remaining display refs exist in the collected field set.
5. **Code generation**:
   - `Page` enum with one variant per file, `Default` impl, `ALL` const, `NAV_PAGES` const (navigable pages only), `label()` method
   - `AppState` struct (if any widgets exist) with all fields + `Default` impl
   - Per-page `render_*` functions: `&mut AppState` for mutable pages, `&AppState` for read-only, none for stateless
   - `render_shared(ui)` if parent has markdown body
   - `LituiApp` struct with `current_page: Page`, `state: AppState`, `show_nav()`, `show_page()`, `show_all()`
   - `show_nav()` iterates `Page::NAV_PAGES` by default (or `Page::ALL` when `nav: { show_all: true }`)
   - Nav bar position controlled by `nav: { position: "top" | "bottom" | "none" }` in parent frontmatter
   - `__setup_theme(ctx)` if parent frontmatter has a `theme:` section (called in `show_all()`)

## Color Resolution

Style colors are either hex (`#RRGGBB`) or semantic keywords (`strong`, `error`, `weak`, etc.):

- **Hex**: Parsed at compile time into `Color32::from_rgb(r, g, b)` — fixed across themes
- **Semantic**: Generates `ui.visuals().{field}` — resolved at runtime, adapts to dark/light mode

The `color_value_tokens()` function in `codegen_ast.rs` dispatches between the two paths. All color fields (`color`, `background`, `stroke_color`) in `StyleDef` accept both forms.
