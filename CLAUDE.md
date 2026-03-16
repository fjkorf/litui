# Project: litui

litui — literate UI for egui. A proc-macro reads `.md` files during compilation and emits egui widget code, with YAML frontmatter for styles, CSS-like selectors, and bevy_ecs state management.

## Documentation Hygiene

This project maintains two complementary documentation layers that MUST stay in sync:

1. **Rustdoc comments** (in-code `///` and `//!`) — the source of truth for what code does
2. **Knowledge markdown** (`knowledge/api/`) — generated from rustdoc, consumed by Claude and contributors

**After any code change that adds, removes, or modifies types, functions, or module docs, run:**

```sh
cargo fmt
cargo clippy
python3 scripts/generate-doc-markdown.py
```

The generated `knowledge/api/` files are how Claude understands the codebase's API. If rustdoc comments change but the markdown isn't regenerated, Claude works with stale information. Treat `python3 scripts/generate-doc-markdown.py` like `cargo fmt` — run it before considering a task complete.

The hand-written `knowledge/*.md` files (pitfalls, testing, architecture) are maintained manually and cover cross-cutting topics that don't belong on any single code item.

## Pre-Push Verification

**Run this checklist before every push.** Do not push if any step fails.

```sh
cargo fmt --all                                            # format
cargo clippy                                               # lint
cargo test --workspace                                     # all tests pass
cargo doc -p litui --no-deps 2>&1 | grep "warning:.*litui" # no doc warnings
python3 scripts/generate-doc-markdown.py                   # regen API docs
cargo test -p litui --test generate_screenshots            # regen tutorial PNGs
```

**Also verify:**

1. **Tutorial ↔ Example 1:1 mapping** — every tutorial must have a `> Run it: \`cargo run -p <example>\`` line pointing to a real example, and every example must be referenced by at least one tutorial. Check `crates/litui/src/_tutorial/mod.rs` index table.
2. **No stale references** — grep for old crate names (`md_demo_lib`, `md_demo_app`) and dead links. Zero hits expected.
3. **Knowledge docs current** — if you changed features, check that `knowledge/widget-directives.md`, `knowledge/frontmatter-and-styles.md`, and other relevant knowledge files reflect the change.
4. **README accurate** — if you added features, examples, or changed the project structure, update `README.md` (it's also the `cargo doc` landing page for litui).

## Knowledge Base

| File | When to read |
|------|-------------|
| [`pulldown-cmark-0.9.md`](knowledge/pulldown-cmark-0.9.md) | Modifying the event loop, adding markdown features, debugging rendering |
| [`proc-macro-architecture.md`](knowledge/proc-macro-architecture.md) | Understanding fragment accumulation, flush points, code generation |
| [`frontmatter-and-styles.md`](knowledge/frontmatter-and-styles.md) | Style system, parent inheritance, ID/class selectors, merge logic |
| [`widget-directives.md`](knowledge/widget-directives.md) | Widget detection, state generation, display widgets, AppState |
| [`testing-patterns.md`](knowledge/testing-patterns.md) | Snapshot tests, event dump tests, macro expansion debugging |
| [`common-pitfalls.md`](knowledge/common-pitfalls.md) | **Start here when debugging** — 18 gotchas with solutions |
| [`third-party-widgets.md`](knowledge/third-party-widgets.md) | Integrating external egui widget crates (built-in vs manual patterns) |
| [`bevy-ecs-integration.md`](knowledge/bevy-ecs-integration.md) | ECS state management, DemoApp architecture, future Bevy plans |
| [`dynamic-styling-design.md`](knowledge/dynamic-styling-design.md) | **Design research** — `[style]` block, `{$field}`, container directives (not yet implemented) |
| [`layout-and-spacing.md`](knowledge/layout-and-spacing.md) | Spacing defaults, CSS→egui mapping, future Frame/column plans |
| [`api/API.md`](knowledge/api/API.md) | **Generated** — full struct fields, fn signatures, enum variants, consts, imports |

## Architecture

### Crates

- **`litui`** (`crates/litui/`) — Facade crate. Re-exports macros and helpers. Hosts user-facing tutorials in `_tutorial` module rendered via `cargo doc`.
  - `src/_tutorial/` — Doc-only modules with `#[doc = include_str!()]` tutorial content
  - `tutorials/` — Tutorial markdown files (01-09)
  - `fixtures/` — Tutorial-specific markdown snippets for screenshot generation
  - `tests/generate_screenshots.rs` — Renders fixtures to `src/_tutorial/img/` PNGs
- **`litui_parser`** (`crates/litui_parser/`) — Standalone parser crate. No proc-macro dependencies. Independently testable.
  - `src/ast.rs` — Pure-data AST types (Inline, Block, Document, WidgetDirective, etc.)
  - `src/frontmatter.rs` — Frontmatter/StyleDef/WidgetDef types, YAML parsing, style merging, selector parsing
  - `src/parse.rs` — `parse_document()`: pulldown-cmark event loop → Document AST
  - `src/error.rs` — ParseError type
- **`litui_macro`** (`crates/litui_macro/`) — Proc-macro crate. Exports `include_litui_ui!()` and `define_litui_app!()`.
  - `src/lib.rs` — Entry points, `load_and_parse_md`
  - `src/parse.rs` — Bridge types (ParsedMarkdown, WidgetField, WidgetType)
  - `src/codegen_ast.rs` — Document AST → ParsedMarkdown (TokenStream generation)
  - `src/codegen.rs` — `parsed_to_include_tokens`, `define_litui_app_impl`, `AppInput`
- **`litui_helpers`** (`crates/litui_helpers/`) — Runtime rendering functions the macro-generated code calls.

### Examples

- **`01_hello`** through **`07_layout`** — Progressive single-page examples using `include_litui_ui!`
- **`08_multi_page`** — Multi-page app with `define_litui_app!`, panels, navigation
- **`09_dynamic`** — Dynamic content: foreach, if, runtime styles
- **`10_advanced`** — All widget types, selectors, advanced button tracking
- **`11_bevy`** — Bevy integration via bevy_egui
- **`12_game`** — Full game UI vertical slice: chargen, inventory, monsters, stats panel
- **`snapshot_tests`** (`tests/snapshot_tests/`) — Headless visual regression tests.

### Dependency Graph

```
litui -> { litui_macro, litui_helpers }
tut_01..07 -> { litui, eframe }
tut_08..10, tut_12 -> { litui, eframe, egui }
tut_11 -> { litui, bevy, bevy_egui }
litui_parser -> { pulldown-cmark 0.9, serde, serde_yaml }
litui_macro -> { litui_parser, quote, syn, serde_yaml }
snapshot_tests -> { litui_macro, litui_helpers, egui_kittest }
```

### egui

- Source: crates.io `egui`/`eframe` v0.33.3 (migrated from fjkorf/egui fork)
- Standard upstream API: `eframe::App::update()`, `CentralPanel::default().show(ctx, |ui| { ... })`
- 3rd-party egui crates (egui_double_slider, etc.) work without `[patch.crates-io]`

## Build & Run

```sh
cargo check
cargo fmt
cargo clippy
cargo test                                                 # run all 31 tests
cargo run -p tut_01_hello                                  # run the hello world example
cargo run -p tut_08_multi_page                             # run the multi-page demo
cargo run -p tut_12_game                                   # run the game UI demo
cargo test -p snapshot_tests                               # run visual regression tests
UPDATE_SNAPSHOTS=true cargo test -p snapshot_tests         # regenerate baseline PNGs
cargo test -p litui_macro dump_ -- --nocapture  # dump event streams
cargo doc -p litui --no-deps --open                        # tutorials and API reference
cargo doc --no-deps --workspace                            # generate HTML docs
cargo test -p litui --test generate_screenshots            # regenerate tutorial PNGs
python3 scripts/generate-doc-markdown.py                   # regenerate knowledge/api/ markdown
```

## Not Yet Implemented

- Task list checkboxes
- Footnotes
- HTML passthrough
