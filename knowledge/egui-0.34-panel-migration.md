# egui 0.34 panel migration — status, rationale, and handoff

**Status (2026-06-24):** litui is on **egui/eframe/egui_extras/egui_kittest 0.34.3, bevy 0.19,
bevy_egui 0.40, MSRV 1.95**, with all egui-0.34 deprecation warnings cleared (build emits 0). The
macro keeps a `Context`-based panel codegen behind `#[allow(deprecated)]` as a **deliberate stopgap**
because the "proper" fix (egui 0.34's `show_inside(ui)` panel model) is **blocked upstream on
bevy_egui not exposing a root `Ui`**. This doc records *why* egui deprecated the old API, *why* we
can't fully migrate yet, and two ready-to-run handoff prompts (one litui-side, one bevy_egui upstream).

`main` is at the post-#6 merge; CI (fmt + clippy + test) and the GitHub Pages docs deploy are green.

---

## 1. Why egui 0.34 deprecated `Panel::show(ctx)` → `show_inside(ui)`

It is the surface of a deliberate **architectural shift from Context-centric to Ui-centric rendering**,
not a cosmetic rename. Three reinforcing reasons:

1. **The root `Ui`, not `Context`, is now the per-frame entrypoint.** egui's changelog: *"we switch
   from having `Context` be the main entrypoint, and instead provide whole-app `Ui`. In egui we've
   replaced `Context::run` with `Context::run_ui`, and changed viewports to be given a `&mut Ui`
   instead of `Context`."* In eframe, `App::update(&Context)` was deprecated in favor of
   `App::ui(&mut Ui, …)`. Panels-on-`Context` belonged to the old model.

2. **`show(ctx)` was provably redundant.** In egui 0.34 source, the deprecated `Panel::show_dyn(ctx)`
   builds a throwaway root Ui and renders into it:
   ```rust
   let mut panel_ui = Ui::new(ctx.clone(), self.id,
       UiBuilder::new().layer_id(LayerId::background()).max_rect(ctx.available_rect()));
   ```
   and `Context::run_ui` (what `App::ui` uses) builds the **same** root Ui once per frame (`"__top_ui"`,
   background layer, `available_rect`) and shares it across all panels, tracking
   `root_ui_available_rect`/`root_ui_min_rect` in pass state. So `show(ctx)` ≡ "make a root Ui, then
   `show_inside`" — pure duplication once a shared root Ui exists, and the per-call version couldn't
   share layout state. (`show(ctx)`'s own doc reads literally: *"Show the panel at the top level."*)

3. **Unification + composability (stated goal).** Changelog: *"SidePanel and TopBottomPanel are
   deprecated, replaced by a single `Panel` … This unification and simplification will make it easier
   to maintain and improve panels going forward."* (precursor: egui issue #5643). Three panel types
   collapse into one `Panel{left,right,top,bottom}` with one entry, `show_inside(ui)`. Because it takes
   any `Ui`, panels can now nest **inside other panels/windows/areas**, not just at the top level.

**Sources:** egui CHANGELOG (`github.com/emilk/egui/blob/main/CHANGELOG.md`), egui issue #5643, and
egui 0.34.3 source `src/containers/panel.rs` (`show`/`show_dyn`/`show_inside`, module docs) +
`src/context.rs` (`run_ui`/`run_ui_dyn`).

---

## 2. The bevy_egui blocker + current litui mitigation

egui 0.34 moved "produce the root `Ui`" up to the **framework-integration layer**: eframe adopted it
(`App::ui` / `run_ui_native`), so eframe apps get a root `&mut Ui` for free. **bevy_egui 0.40 has not
made that move** — it exposes only `EguiContexts::ctx_mut() -> &mut egui::Context` (the old
entrypoint). In Bevy, the only way to obtain a `Ui` is `CentralPanel::show(ctx, |ui| …)`, which is the
deprecated method. So Bevy code is structurally stuck on the deprecated `Context` path until bevy_egui
exposes a root Ui (via egui's `Context::run_ui`). This is **not** litui working around a quirk — it's
bevy_egui lagging egui's intended integration contract.

**What is on `main` today (the stopgap):**
- `crates/litui_macro/src/codegen.rs`: the generated `define_litui_app!` `show_all(&mut self, ctx:
  &egui::Context)` is wrapped in `#[allow(deprecated)]` (both emission points — `has_containers` and
  the simple-layout branch), with a comment explaining the bevy_egui constraint. `__setup_theme` uses
  the clean rename `ctx.global_style_mut(...)`.
- `examples/01_hello`..`07_layout` (eframe single-page): **genuinely de-deprecated** —
  `eframe::run_simple_native` → `run_ui_native` (closure gets `&mut Ui`), and
  `CentralPanel::default().show(ctx, …)` → `.show_inside(ui, …)`; `install_image_loaders(ui.ctx())`.
- `examples/11_bevy`, `examples/13_custom`: `#[allow(deprecated)]` on `render_nav`/`render_page`
  (forced — bevy_egui 0.40 gives only `&Context`).
- `examples/08/09/10/12` (eframe multi-page): already covered (macro `#[allow]` for `show_all`;
  existing `#[allow(deprecated)]` on their `impl eframe::App` blocks for the `App::update` bridge).

`cargo build --workspace` → 0 deprecation warnings; `cargo test --workspace` green. No behavioral
change (the `show_all` body is unchanged; `show_inside` on the root ui preserves central-panel layout),
so snapshots are unaffected.

**Trigger to revisit:** bevy_egui exposing a root `Ui` (or egui offering a non-deprecated way to build
a full-area root `Ui` from a `&Context` outside eframe).

---

## 3. Handoff A — litui-side `show_inside` migration (do this once unblocked)

> Investigate and (if viable) implement the `show_inside` panel migration for litui's macro.
>
> Repo `/Users/frankkorf/Playspaces/litui` (branch `main`). The blocker is described above: litui's
> `define_litui_app!` generates `show_all(&mut self, ctx: &egui::Context)` using deprecated
> `*Panel::show(ctx)`, kept behind `#[allow(deprecated)]` because bevy_egui 0.40 exposes no root `Ui`.
>
> Steps:
> 1. **Is it unblocked?** Check whether a newer bevy_egui exposes a root `Ui` (or whether egui itself
>    now offers a non-deprecated full-area root `Ui` from a `&Context` — read egui `ui.rs` `Ui::new`/
>    `UiBuilder` + `context.rs` `run_ui`). If still blocked, stop and keep the stopgap.
> 2. **If unblocked:** migrate `crates/litui_macro/src/codegen.rs` — `show_all(&mut self, ui: &mut
>    egui::Ui)` using `egui::Panel::left/right/top/bottom(id)…show_inside(ui, …)` and
>    `CentralPanel::default().show_inside(ui, …)`; theme via `ui.ctx().global_style_mut()`. Then flip
>    eframe examples 08/09/10/12 from `App::update(ctx)`+`show_all(ctx)` to `App::ui(ui)`+`show_all(ui)`
>    (clears their `App::update` `#[allow]`), and update bevy examples 11/13 to get a root ui and call
>    `show_all(ui)`. Drop the now-unneeded `#[allow(deprecated)]`s.
> 3. Treat the `show_all` signature change (`ctx` → `ui`) as a deliberate breaking change; note it for
>    RustRetro (Bevy consumer).
> 4. Verify: `cargo build --workspace` (0 deprecation warnings), `cargo test --workspace` (green incl.
>    snapshots), `cargo test -p tut_13_custom`. Land via PR (direct push to `main` is blocked).

---

## 4. Handoff B — bevy_egui upstream fix (the real unblock)

> Contribute an upstream fix to **bevy_egui** that exposes a root `egui::Ui`, so egui 0.34's
> `Panel::show_inside(ui)` model is usable in Bevy. Work in a fresh clone of the bevy_egui repo
> (`github.com/vladbat00/bevy_egui` — confirm canonical/maintained), NOT the local app or the
> read-only `~/.cargo/registry` copy.
>
> **Problem:** see §1–§2. egui 0.34 deprecated `Context`-based panel `.show(ctx)` for `show_inside(ui)`,
> which needs a root `Ui`. eframe provides it via `App::ui`, internally
> `egui::Context::run_ui(raw_input, |ui| …)` (see `eframe-0.34.3/src/native/epi_integration.rs`).
> bevy_egui 0.40 exposes only `EguiContexts::ctx_mut() -> &mut egui::Context` and drives egui through a
> `Context`-based pass — no root Ui — so Bevy users have no non-deprecated panel path.
>
> **Goal:** add a non-deprecated, ergonomic way for Bevy systems to render into a **root, full-area
> `egui::Ui`** for an egui context, mirroring eframe's `App::ui`, so users can call
> `egui::Panel::top(id).show_inside(ui, …)` etc. without deprecation warnings.
>
> **Investigate first:** (a) is it already solved in a newer bevy_egui (changelog/issues/PRs)? if so,
> the downstream fix is just a version bump — report and stop. (b) how bevy_egui drives the frame
> (`ctx.run`/`begin_pass`/`end_pass`, the `EguiPrimaryContextPass`/`EguiContextPass` schedules,
> single- vs multi-pass). (c) egui's `Context::run_ui` and `Ui::new`/`UiBuilder` (`context.rs`,
> `ui.rs`) — the mechanism to replicate/expose.
>
> **Design (align with maintainer — open an issue first):** provide a root `&mut Ui` to systems —
> e.g. an `EguiContexts::root_ui_mut()` / a system-param accessor, or a closure entry analogous to
> `App::ui` run inside `Context::run_ui`. Must be **additive** (don't break `ctx_mut()` users), work in
> **single- and multi-pass** modes, and use correct full-screen rect/scaling/input. Add an example
> (port an existing panel example to root-Ui + `show_inside`) + tests.
>
> **Validate downstream:** litui (`/Users/frankkorf/Playspaces/litui`) is the motivating consumer —
> its macro is pinned to deprecated `Context`-based `show(ctx)` *because* Bevy can't supply a root Ui
> (see §2/§3). With your change, a Bevy system should obtain a root `Ui` and drive panels via
> `show_inside` with zero deprecation warnings; confirm it unblocks Handoff A.
>
> **Deliverable:** determination (viable now / still blocked) + a fork/branch with implementation,
> example, tests, changelog entry, and a PR-ready description (problem → egui 0.34 Context→Ui shift →
> `run_ui` mechanism → new API → backward compatibility). Run the repo's fmt/clippy/test/CI. This is a
> real OSS contribution — match conventions, keep it additive, follow the maintainer's steer.

---

## 5. Operational notes (build environment & git)

- **Build artifacts live on an external APFS image** (the internal disk filled during this work). The
  cargo `target-dir` is redirected via a **gitignored** `/.cargo/config.toml` to
  `/Volumes/litui-build/litui-target`. **Before building in a fresh shell / after a reboot or replug,
  mount it:** `hdiutil attach /Volumes/Samsung_T5/litui-build.sparsebundle` (mounts at
  `/Volumes/litui-build`). If absent, cargo errors clearly rather than scattering artifacts. To
  reclaim space: `hdiutil detach /Volumes/litui-build && hdiutil compact
  /Volumes/Samsung_T5/litui-build.sparsebundle`.
- **CI** (`.github/workflows/ci.yml`) gates fmt + clippy + `cargo test --workspace --exclude
  snapshot_tests` on PRs and pushes to `main`. Snapshot baselines are macOS-rendered (kept local). The
  `litui` `generate_screenshots` test is a GPU-backed PNG generator and is skipped in CI via
  `LITUI_SKIP_SCREENSHOTS=1`. Direct pushes to `main` are blocked by the harness — **land via PR**.
- **gh-pages** docs deploy (`.github/workflows/gh-pages.yml`) was fixed (contents:write +
  `publish_dir: target/docs/doc` + root redirect); both CI and docs-deploy are green on `main`.
- **Branches:** `main` (canonical, `litui_*` crate names), `litui-shippable` (preserved older
  `markdown_to_egui_*` lineage — staging branch, do not delete), `origin/gh-pages` (published docs).
- The `[custom]` escape hatch, the egui-0.34/bevy-0.19 bump, CI, and this deprecation cleanup are all
  landed on `main`. Remaining non-deprecation cleanup: `unused variable: i` in
  `crates/litui_macro/src/codegen_ast.rs` and the broader clippy style backlog.
