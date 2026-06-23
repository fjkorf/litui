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

## Two features RustRetro drives (and litui must build)

### 1. The `[custom]` escape hatch — the linchpin

A directive that invokes a user-supplied `FnMut(&mut egui::Ui)` stored on `AppState`:

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

Open question / risk: lifetimes of a `FnMut` held on a macro-generated `AppState` across the
proc-macro boundary. **Prototype before RustRetro commits to the migration** — if this is hard,
the whole "litui owns the frame" claim is at risk. Relates to
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

RustRetro currently lags litui (egui 0.31 vs 0.33; bevy 0.15 vs 0.18). Binding a real app to
litui couples it to litui's egui cadence. litui should ship a stable-ish release with a stated
**minimum supported egui** (and ideally CI against two versions) so a litui egui bump doesn't
force-march every consumer. This is a prerequisite for litui being credible beyond demos.

## Showcase integrity

Keep the win **measurable**: report "litui owns the frame + N standard screens in ~X lines of
Rust+Markdown; the app keeps K bespoke spatial panels." If the sync glue or escape hatches
balloon, the "little Rust" claim weakens — keep the glue small and visible. The final artifact
is a writeup: *how litui powers the chrome of a 68000 debugger.*
