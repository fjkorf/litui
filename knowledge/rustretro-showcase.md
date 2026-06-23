# RustRetro Showcase & Driver Requirements — Design Research

## Status: Planned. Drives two unbuilt litui features.

RustRetro (a from-scratch Rust libretro frontend / 68000 debugging instrument) is litui's first
**real-world specialized-app showcase** — not a demo, but a brownfield app with a demanding
custom core. Its consumer-side plan lives in `../rustretro/ROADMAP.md` ("litui integration").

## The thesis litui proves here

Specialized tools are ~80% rote UI (menus, forms, settings, lists, status readouts) wrapped
around a small bespoke core. litui's pitch: **own the UI framework and every standard screen
with a little Rust + Markdown, so the app spends its hand-written effort only on the few
surfaces that truly matter.** RustRetro is the proof: litui owns the window frame, navigation,
and ~6 standard screens; RustRetro keeps five custom-painted inspectors (framebuffer, tile
grid, hex dump, disassembly listing, PC heatmap).

## The boundary principle (sharper than "generic vs. domain-specific")

The cut line that predicts litui-fit is **shape, not subject matter**:

> List / form / display surfaces → litui. Custom-painted, spatial surfaces → bespoke.

A *watch-variable table* and a *breakpoint manager* are deeply domain-specific yet are pure
list/form/display — litui owns them. A *framebuffer view* is generic ("show an image") yet is
spatially custom-painted — it stays bespoke. Stating the boundary this way matters for showcase
integrity: it shows litui handling domain-specific screens, not just the boring chrome.

## Two features RustRetro drives

### 1. The `[custom]` escape hatch — the linchpin — BUILT

A directive that invokes a user-supplied closure stored on the generated state struct:

```markdown
[custom](framebuffer_slot)
```
```rust
app.framebuffer_slot = Some(Box::new(|ui| { /* raw egui: ui.image(...) etc. */ }));
```

It is **doubly load-bearing**:
- *Inside a page* — drop a framebuffer/sparkline into otherwise-Markdown content.
- *As a whole page* — let a bespoke inspector live as a **page inside litui's navigation**.
  Without this, litui cannot own the shell while hosting an app's custom panels; with it, the
  bespoke panels are just slots in the nav and litui owns the frame.

**Resolved (it works).** The slot is stored as
`Option<Box<dyn FnMut(&mut egui::Ui) + Send + Sync>>` on the generated state struct. The
`Send + Sync` bound is required because the state struct is meant to live in a Bevy `Resource`;
the closure is `FnMut` (not `Fn`) so it can mutate its captures. Because a boxed closure is
neither `Clone` nor `Debug`, those derives are dropped from the generated state whenever a
custom slot is present. At runtime the macro uses a **take/replace** calling pattern — take the
closure out of the `Option`, call it, put it back — to satisfy the borrow checker across the
proc-macro boundary. The earlier lifetime concern is closed.

Proven on the full Bevy stack (bevy 0.19 / bevy_egui 0.40 / egui 0.34) via
`examples/13_custom/`, with 2 passing headless tests, including the **whole-page-as-slot** case:
`examples/13_custom/content/panel.md` is a page whose entire body is just
`[custom](panel_slot)`. Relates to
[`third-party-widgets.md`](third-party-widgets.md) (manual-integration pattern).

### 2. Live-resource binding in the Bevy path

`11_bevy` only renders static content; `12_game`'s per-frame `populate_data` (live data → 
`[display]`/`[progress]`/`::$field`) is the pattern an inspector needs, but it's only shown on
eframe. RustRetro needs that **in Bevy**: a `Resource`'s changing data (registers, watch
values, log lines) flowing into litui widgets every frame. A Bevy example demonstrating this
would both unblock RustRetro and become litui's strongest "dashboard/instrument" showcase.
See [`bevy-ecs-integration.md`](bevy-ecs-integration.md).

## State-coupling contract

RustRetro's truth is `Arc<Mutex<DebugState>>`; litui's `AppState` must be a **pure projection**
(one `sync(debug, app)` per frame: values down, widget outputs up). litui's "widgets mutate
state fields, no event enum" model maps cleanly onto RustRetro's existing "UI flips a bool, the
run loop consumes it" signals (`create_bookmark`, `save_regions`). Document this as the
recommended pattern for any litui-in-a-real-app integration.

## Adoption risk litui must address: version policy

RustRetro currently lags litui (egui 0.31 vs 0.34; bevy 0.15 vs 0.19). Binding a real app to
litui couples it to litui's egui cadence. litui should ship a stable-ish release with a stated
**minimum supported egui** (and ideally CI against two versions) so a litui egui bump doesn't
force-march every consumer. This is a prerequisite for litui being credible beyond demos.

## Deferred: parser-crate refactor

A parser-crate extraction was started and then **intentionally deferred** — it is future work,
**not** part of the shippable baseline. The following artifacts are parked, not live:

- `crates/markdown_to_egui_parser/` — the extracted parser crate. It is a workspace member but
  is **not consumed** by the shippable pipeline.
- `crates/markdown_to_egui_macro/src/_codegen.rs` — the refactor-target codegen, parked and
  underscore-prefixed so it is not in the module tree.
- the grammar test harness (`tests/grammar_harness.rs` + `tests/grammar_cases/`) — removed from
  the tree because it tests a public `parse_markdown` parser API that can only exist after the
  refactor (a proc-macro crate can't export callable `pub fn`s).

Finishing the refactor would take: implement the stubbed pieces (`lib.rs` currently returns an
empty `ParsedMarkdown`; `_codegen.rs`'s `show_all` are stubs), port the `WidgetField::CustomSlot`
variant into the parser crate's `WidgetField` (it currently has only `Stateful`), reconstruct the
wiring that swaps `parse.rs` over to emit parser-crate types (the original WIP stash was scrapped),
then restore the grammar harness against the now-public `markdown_to_egui_parser::parse_markdown`.

## Showcase integrity

Keep the win **measurable**: report "litui owns the frame + N standard screens in ~X lines of
Rust+Markdown; the app keeps K bespoke spatial panels." If the sync glue or escape hatches
balloon, the "little Rust" claim weakens — keep the glue small and visible. The final artifact
is a writeup: *how litui powers the chrome of a 68000 debugger.*
