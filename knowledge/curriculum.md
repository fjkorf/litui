# Curriculum Design

Blueprint for structuring litui's tutorials, examples, and documentation into a progressive learning path.

## Design Principles

1. **Progressive** — each tutorial builds on the previous; no concept is used before it's taught
2. **1:1 example mapping** — each tutorial has a dedicated runnable example
3. **Show source + result** — every tutorial shows the `.md` source and a screenshot of the rendered UI
4. **Expert tips** — each tutorial includes one technical deep-dive sourced from knowledge docs
5. **Shortest to longest** — tutorial 01 is the minimal hello world; tutorial 12 is a full game vertical slice

## Tutorial Sequence

| # | Tutorial | New Concept | Example | Builds On |
|---|---------|-------------|---------|-----------|
| 01 | Hello Markdown | Markdown → egui (headings, paragraphs, bold, italic, lists, blockquotes, code, links, separators) | `01_hello` | — |
| 02 | Frontmatter Styles | YAML `styles:` block, `::key` suffix, `::key(text)` inline spans, style properties (color, bold, size, etc.) | `02_styles` | 01 |
| 03 | Tables | GFM table syntax, `egui::Grid` rendering, inline formatting in cells, striped rows | `03_tables` | 02 |
| 04 | Images | `![alt](url)` syntax, `file://` paths, `egui_extras::install_image_loaders()` | `04_images` | 03 |
| 05 | Styled Containers | Styled blockquotes (colored bars), styled lists (colored bullets/numbers), `::: frame panel` with frame properties | `05_containers` | 04 |
| 06 | Widgets | `[slider]`, `[checkbox]`, `[textedit]`, `[button]`, `[display]`, `{config}` widget configs, state struct generation | `06_widgets` | 05 |
| 07 | Layout | `::: horizontal` for side-by-side, `::: columns N` with `::: next`, `spacing:` frontmatter config | `07_layout` | 06 |
| 08 | Multi-Page Apps | `define_litui_app!`, `Page` enum, `AppState`, parent frontmatter, `panel:` containers (left/right/top/bottom/window), `show_all()` | `08_multi_page` | 07 |
| 09 | Dynamic Content | `::: foreach` iteration, `::: if` conditionals, `::: style` runtime color, `::$field` shorthand | `09_dynamic` | 08 |
| 10 | Advanced Widgets | `[radio]`, `[combobox]`, `[select]`, `[toggle]`, `[color]`, `[progress]`, `[log]`, `[textarea]`, `[password]`, `[dragvalue]`, `[spinner]`, `[selectable]`, advanced button tracking (`track_hover`, `track_secondary`), `.class` and `#id` selectors on widgets | `10_advanced` | 09 |
| 11 | Bevy Integration | `bevy_egui` plugin, `EguiContexts` system param, `Page`/`AppState` as `Resource`, render in `EguiPrimaryContextPass`, business logic in `Update` | `11_bevy` | 10 |
| 12 | Game UI | Full vertical slice: character creation with `[select]`, inventory with `::: foreach`, monster info cards, stat panel (`panel: right`), dynamic HP coloring, window visibility control (`open:`), 3rd-party widget integration | `12_game` | 11 |

## Per-Tutorial Spec

### 01 — Hello Markdown
- **Introduces:** `include_litui_ui!`, headings (H1-H3), paragraphs, **bold**, *italic*, ~~strikethrough~~, bullet lists, numbered lists, nested lists, blockquotes, fenced code blocks, `inline code`, [links](url), horizontal rules
- **Assumes:** Rust basics, eframe setup
- **Example:** `cargo run -p 01_hello`
- **Screenshot:** Headings + inline formatting; lists + blockquotes
- **Expert tip:** Compile-time codegen — the macro emits `fn` calls, no runtime markdown parsing (source: `proc-macro-architecture.md`)
- **Length target:** ~80 lines

### 02 — Frontmatter Styles
- **Introduces:** YAML frontmatter (`---` delimiters), `styles:` section, style properties (bold, italic, color, background, size, monospace, weak, underline, strikethrough), `::key` line suffix, `::key(text)` inline spans, style composition
- **Assumes:** 01
- **Example:** `cargo run -p 02_styles`
- **Screenshot:** Styled text with colors and sizes
- **Expert tip:** Style resolution pipeline — `strip_frontmatter()` → `detect_style_suffix()` → `style_def_to_label_tokens()` → `styled_label_rich()` (source: `frontmatter-and-styles.md`)
- **Length target:** ~120 lines

### 03 — Tables
- **Introduces:** GFM pipe table syntax, header row, alignment, bold headers, striped rows, inline formatting in cells
- **Assumes:** 01, 02
- **Example:** `cargo run -p 03_tables`
- **Screenshot:** Basic table; table with formatting
- **Expert tip:** pulldown-cmark table events → `egui::Grid::new().striped(true)` (source: `pulldown-cmark-0.9.md`)
- **Length target:** ~80 lines

### 04 — Images
- **Introduces:** `![alt](url)` syntax, `file://` URI for local images, `egui_extras::install_image_loaders()` setup, images in tables, alt text as fallback
- **Assumes:** 01-03
- **Example:** `cargo run -p 04_images`
- **Screenshot:** Image rendering with alt text
- **Expert tip:** `egui::Image` loader initialization and supported formats (source: helpers code comments)
- **Length target:** ~80 lines

### 05 — Styled Containers
- **Introduces:** `::key` on blockquotes (colored vertical bars), `::key` on list items (colored bullets/numbers), frame style properties (`inner_margin`, `stroke`, `stroke_color`, `corner_radius`, `background`), `::: frame panel` block directive
- **Assumes:** 01-04
- **Example:** `cargo run -p 05_containers`
- **Screenshot:** Colored blockquotes; colored lists; frame containers
- **Expert tip:** CSS box model → egui Frame mapping (source: `layout-and-spacing.md`)
- **Length target:** ~120 lines

### 06 — Widgets
- **Introduces:** `[slider](field)`, `[checkbox](field)`, `[textedit](field)`, `[button](label)`, `[display](field)`, `{config}` widget config syntax, `widgets:` frontmatter section (min, max, label, hint, format), state struct generation (`LituiFormState`), click counters, display self-declaration
- **Assumes:** 01-05
- **Example:** `cargo run -p 06_widgets`
- **Screenshot:** Form with slider, checkbox, text input, buttons
- **Expert tip:** Widget detection via markdown link interception — `[slider]` is just a link where the text matches `WIDGET_NAMES` (source: `widget-directives.md`)
- **Length target:** ~200 lines

### 07 — Layout
- **Introduces:** `::: horizontal` (side-by-side elements), `::: columns N` with `::: next` (multi-column), `spacing:` frontmatter (paragraph, table, heading_h1-h4, item), parent inheritance of spacing
- **Assumes:** 01-06
- **Example:** `cargo run -p 07_layout`
- **Screenshot:** Side-by-side buttons; two-column layout
- **Expert tip:** Block directive stack — `code_body` swap pattern for nested containers (source: `proc-macro-architecture.md`)
- **Length target:** ~100 lines

### 08 — Multi-Page Apps
- **Introduces:** `define_litui_app!` macro, `page:` frontmatter (name, label, default), `Page` enum, shared `AppState`, per-page `render_*()` functions, `LituiApp` struct, `show_nav()`, `show_page()`, parent frontmatter (`parent:` keyword), `panel:` containers (left, right, top, bottom, window), `show_all()` auto-dispatch, window `open:` visibility control
- **Assumes:** 01-07
- **Example:** `cargo run -p 08_multi_page`
- **Screenshot:** Navigation bar with page tabs; side panel layout
- **Expert tip:** `Page` enum generation and `AppState` field merging across pages (source: `codegen.rs`)
- **Length target:** ~200 lines

### 09 — Dynamic Content
- **Introduces:** `::: foreach field` with `{field}` row references, auto-generated row structs (`FieldRow`), `::: if field` conditional rendering, `::: style field` runtime color override, `::$field` paragraph shorthand, `__resolve_style_color()` match table
- **Assumes:** 01-08
- **Example:** `cargo run -p 09_dynamic`
- **Screenshot:** Dynamic list from Vec; conditional content; colored text
- **Expert tip:** `__resolve_style_color()` — compile-time match table generated from frontmatter styles (source: `parse.rs` style table generation)
- **Length target:** ~180 lines

### 10 — Advanced Widgets
- **Introduces:** `[radio]`, `[combobox]`, `[select]`, `[toggle]`, `[color]`, `[progress]`, `[log]`, `[textarea]`, `[password]`, `[dragvalue]`, `[spinner]`, `[selectable]`, `.class` selectors on widgets, `#id` selectors, `track_hover`/`track_secondary`, `suffix`/`prefix` on sliders, `fill` on progress bars, widgets in table cells, 3rd-party widget integration (Pattern A: built-in directive, Pattern B: manual alongside macro)
- **Assumes:** 01-09
- **Example:** `cargo run -p 10_advanced`
- **Screenshot:** Advanced widget showcase
- **Expert tip:** Pattern A vs Pattern B for third-party widgets (source: `third-party-widgets.md`)
- **Length target:** ~250 lines

### 11 — Bevy Integration
- **Introduces:** `bevy_egui` crate and `EguiPlugin`, `EguiContexts` system param, `Page`/`AppState` as bevy `Resource`, render systems in `EguiPrimaryContextPass` schedule, business logic in `Update` schedule, identical output to eframe
- **Assumes:** 01-10, Bevy basics
- **Example:** `cargo run -p 11_bevy`
- **Screenshot:** Bevy window with litui UI
- **Expert tip:** `EguiContexts` and render pass scheduling — why `EguiPrimaryContextPass` not `Update` (source: `bevy-ecs-integration.md`)
- **Length target:** ~150 lines

### 12 — Game UI
- **Introduces:** Character creation screen with `[select]` + display fields, inventory management with `::: foreach`, monster info cards with display-only pages, persistent stat panel (`panel: right`), dynamic HP color via `::$field`, window visibility with `open:`, combining all prior concepts into a cohesive game UI
- **Assumes:** All previous
- **Example:** `cargo run -p 12_game`
- **Screenshot:** Character creation; inventory table; stat sidebar
- **Expert tip:** Module stratification — which litui modules are foundational vs leaf, and how the dependency DAG informs architecture decisions (source: `api/API.md` stratification table)
- **Length target:** ~200 lines

## Feature Coverage Matrix

| Feature | Tutorial | Widget/Directive |
|---------|----------|-----------------|
| Headings | 01 | — |
| Paragraphs, bold, italic, strikethrough | 01 | — |
| Lists (bullet, numbered, nested) | 01 | — |
| Blockquotes | 01 | — |
| Code blocks, inline code | 01 | — |
| Links | 01 | — |
| Horizontal rules | 01 | — |
| Frontmatter `styles:` | 02 | — |
| `::key` line suffix | 02 | — |
| `::key(text)` inline spans | 02 | — |
| Style properties (color, bold, size, etc.) | 02 | — |
| GFM tables | 03 | — |
| Images `![alt](url)` | 04 | — |
| Styled blockquotes (colored bars) | 05 | — |
| Styled lists (colored bullets) | 05 | — |
| `::: frame` containers | 05 | Frame |
| `[slider]` | 06 | Slider |
| `[checkbox]` | 06 | Checkbox |
| `[textedit]` | 06 | TextEdit |
| `[button]` | 06 | Button |
| `[display]` | 06 | Display |
| Widget `{config}` | 06 | — |
| `::: horizontal` | 07 | Horizontal |
| `::: columns` + `::: next` | 07 | Columns |
| `spacing:` config | 07 | — |
| `define_litui_app!` | 08 | — |
| `page:` frontmatter | 08 | — |
| Parent frontmatter inheritance | 08 | — |
| `panel:` containers | 08 | — |
| Window `open:` control | 08 | — |
| `::: foreach` | 09 | Foreach |
| `::: if` | 09 | If |
| `::: style` | 09 | Style |
| `::$field` runtime styles | 09 | — |
| `[radio]` | 10 | Radio |
| `[combobox]` | 10 | ComboBox |
| `[select]` | 10 | Select |
| `[toggle]` | 10 | Toggle |
| `[color]` | 10 | ColorPicker |
| `[progress]` | 10 | Progress |
| `[log]` | 10 | Log |
| `[textarea]` | 10 | TextArea |
| `[password]` | 10 | Password |
| `[dragvalue]` | 10 | DragValue |
| `[spinner]` | 10 | Spinner |
| `[selectable]` | 10 | Selectable |
| `.class` / `#id` selectors | 10 | — |
| `track_hover` / `track_secondary` | 10 | — |
| 3rd-party widget integration | 10 | — |
| Bevy integration | 11 | — |
| Game vertical slice | 12 | — |

## Gap Analysis

Features not covered by any tutorial:
- `[double_slider]` — requires `egui_double_slider` dependency; covered in 3rd-party section of 10
- Widget layout control (inline vs block) — partially addressed by `::: horizontal`
- Task list checkboxes — not yet implemented
- Footnotes — not yet implemented
- HTML passthrough — not yet implemented
