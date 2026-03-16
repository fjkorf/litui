# Testing Patterns

## Snapshot Tests

Visual regression tests use `egui_kittest` to render markdown headlessly via wgpu and compare against baseline PNGs. No window opens — works in CI.

### Quick Reference

```sh
cargo test -p snapshot_tests                                    # compare against baselines
UPDATE_SNAPSHOTS=true cargo test -p snapshot_tests              # update only failing
UPDATE_SNAPSHOTS=force cargo test -p snapshot_tests             # regenerate all
UPDATE_SNAPSHOTS=true cargo test -p snapshot_tests my_test      # generate single baseline
```

### Adding a New Test

1. Create fixture: `tests/snapshot_tests/fixtures/my_feature.md`
2. Add test function to `tests/snapshot_tests/tests/snapshots.rs`:
   ```rust
   #[test]
   fn my_feature() {
       snapshot_markdown("my_feature", include_markdown_ui!("fixtures/my_feature.md"));
   }
   ```
3. Generate baseline: `UPDATE_SNAPSHOTS=true cargo test -p snapshot_tests my_feature`
4. Verify visually by reading the PNG
5. Commit the `.png` as the baseline

For stateful widgets (slider/checkbox/textedit), destructure the tuple:
```rust
#[test]
fn my_form() {
    let (render, mut state) = include_markdown_ui!("fixtures/my_form.md");
    let mut harness = Harness::builder()
        .with_size(egui::vec2(800.0, 600.0))
        .with_max_steps(16)
        .wgpu()
        .build_ui(move |ui| { render(ui, &mut state); });
    harness.run_steps(2);
    harness.fit_contents();
    // ... same pattern as snapshot_markdown helper
}
```

### File Locations

| Purpose | Path |
|---------|------|
| Single-file tests | `tests/snapshot_tests/tests/snapshots.rs` |
| Multi-page demo tests | `tests/snapshot_tests/tests/demo_pages.rs` |
| Parent body tests | `tests/snapshot_tests/tests/parent_body.rs` |
| Fixtures | `tests/snapshot_tests/fixtures/*.md` |
| Baselines | `tests/snapshot_tests/snapshots/*.png` |
| Config | `kittest.toml` (workspace root) |
| Test Cargo.toml | `tests/snapshot_tests/Cargo.toml` |

### Current Fixtures (9 tests in snapshots.rs)

| Fixture | Tests |
|---------|-------|
| `example.md` | Full feature demo (headings, lists, quotes, code, links) |
| `headings.md` | H1-H3 only |
| `styled.md` | Frontmatter `::key` styles on paragraphs |
| `tables.md` | GFM tables with formatting |
| `widgets.md` | Buttons, progress bar, spinner |
| `stateful.md` | Sliders, checkbox, text input (state tuple) |
| `selectors.md` | Class selectors: `.action`, `.action.large` (composed), `.danger` |
| `widgets_in_table.md` | Buttons and progress bar inside table cells |
| `inline_styles.md` | Styled text spans: `::accent(text)`, link classes |

### Testing `define_markdown_app!` Pages

For multi-page apps with `AppState`, display widgets, and parent frontmatter, create a separate test file that invokes the macro in a module:

```rust
mod app {
    use markdown_to_egui_helpers::*;
    use markdown_to_egui_macro::define_markdown_app;

    define_markdown_app! {
        parent: "fixtures/parent.md",
        "fixtures/page1.md",
        "fixtures/page2.md",
    }
}

#[test]
fn page_with_state() {
    let mut state = app::AppState::default();
    state.volume = 50.0;
    snapshot_page("test_name", move |ui| app::render_page1(ui, &mut state));
}

#[test]
fn read_only_page() {
    let state = app::AppState::default();
    snapshot_page("test_name", move |ui| app::render_page2(ui, &state));
}
```

See `tests/demo_pages.rs` (8 tests covering all demo pages) and `tests/parent_body.rs` (parent body + style inheritance).

## How egui_kittest Rendering Works

### Harness Builder

```rust
Harness::builder()
    .with_size(egui::vec2(800.0, 600.0))  // Canvas size
    .with_max_steps(16)                     // Max frame iterations
    .wgpu()                                 // Headless GPU rendering
    .build_ui(|ui| { ... })
```

Defaults: 800x600, max_steps=4, dark theme, 1.0 DPI, OS=Nix (cross-platform consistent).

### Rendering Pipeline

1. **Layout** — `harness.run_steps(2)` runs 2 egui frames to stabilize layout
2. **Fit** — `harness.fit_contents()` measures actual content bounding box, resizes canvas
3. **Render** — `harness.run_steps(2)` re-renders at final size
4. **Snapshot** — `harness.snapshot("name")` captures:
   - egui tessellates UI to triangles
   - wgpu renders to intermediate texture (RGBA8, transparent clear)
   - DMA copy from GPU texture to CPU staging buffer
   - Extract RGBA pixels (handling row padding)
   - Compare against baseline PNG

### Snapshot Comparison (dify)

Per-pixel perceptual diff using the `dify` crate:
- `threshold = 0.6` (from `kittest.toml`) — per-pixel tolerance, 60% difference allowed
- `failed_pixel_count_threshold = 0` — zero pixels may exceed threshold (strict)
- Size must match exactly (no fuzzy sizing)

Output files on failure:
- `{name}.new.png` — current render
- `{name}.diff.png` — visual diff (red overlay)
- `{name}.old.png` — previous baseline (when updating)

### kittest.toml

```toml
output_path = "snapshots"   # relative to test crate's tests/ directory
threshold = 0.6             # per-pixel perceptual diff tolerance
```

Config is found by searching from CWD upward until `kittest.toml` is found.

### Limitations

- **Display widgets** (`[display](field)`) cannot be tested via `include_markdown_ui!` — they reference `state.field` which only exists in `define_markdown_app!`'s `AppState`. Use `define_markdown_app!` in a test module (see `tests/demo_pages.rs` and `tests/parent_body.rs`).
- **Spinner** continuously requests repaints. Must use `max_steps(16)` and `run_steps(N)` instead of `run()`.
- **`fit_contents()` uses deprecated `screen_rect()`** — suppressed with `#[expect(deprecated)]`.

## Event Dump Tests

When debugging markdown rendering issues, **always dump the event stream first**:

```sh
cargo test -p markdown_to_egui_macro dump_ -- --nocapture
```

Tests live in `crates/markdown_to_egui_macro/tests/dump_events.rs`. Current dump tests:

| Test | What it verifies |
|------|-----------------|
| `dump_nested_bullet_list_4space` | 4-space indent nesting |
| `dump_nested_bullet_list_2space` | 2-space indent nesting |
| `dump_nested_ordered_list` | Ordered list nesting |
| `dump_blockquote_with_list` | Blockquote + nested list |
| `dump_table_with_alignment` | Table column alignment |
| `dump_table_with_formatting` | Bold/code/links in table cells |
| `dump_widget_syntax` | Widget link syntax variants |
| `dump_display_widget` | Display widget event stream |
| `dump_widget_in_table_cell` | Widget directives inside table cells |
| `dump_inline_styled_span` | `::class(text)` styled span event stream |
| `dump_link_with_selectors` | `#id.class` selectors in link text |

## Macro Expansion Debugging

To inspect generated code:
```sh
cargo expand -p snapshot_tests --test snapshots 2>&1 | head -100
```

The test `crates/markdown_to_egui_macro/tests/deeply_nested.rs` verifies that macro expansion compiles for deeply nested markdown structures.

## Bevy Demo Testing

The Bevy demo (`examples/bevy_demo/`) renders identical content to the eframe demo, which is comprehensively tested by kittest. `egui_kittest` does not support Bevy's render pipeline.

Bevy 0.18 has `bevy_render::view::screenshot::Screenshot` for capturing frames — spawn a `Screenshot::primary_window()` entity with an observer to save to disk. However, this requires either a windowed app or a custom headless render graph setup (`ScheduleRunnerPlugin` + `WindowPlugin { primary_window: None }`).

**Current approach:** The Bevy demo is verified by `cargo check -p bevy_demo` (compile test) and manual smoke testing (`cargo run -p bevy_demo`). Content correctness is covered by the eframe snapshot tests since both apps render the same `demo_content` pages.
