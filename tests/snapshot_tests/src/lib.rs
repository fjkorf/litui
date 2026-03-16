//! Snapshot test utilities for visual regression testing of markdown rendering.
//!
//! Uses `egui_kittest` with wgpu to render markdown UI headlessly and compare
//! against baseline PNG files. See `knowledge/testing-patterns.md` for the full
//! workflow including fixture creation, baseline generation, and comparison config.

use egui_kittest::Harness;

/// Render a markdown UI closure and snapshot it at 800px width.
pub fn snapshot_markdown(name: &str, ui_fn: impl FnMut(&mut egui::Ui) + 'static) {
    let width = 800.0;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(width, 600.0))
        .with_max_steps(16)
        .wgpu()
        .build_ui(ui_fn);
    harness.run_steps(2);
    harness.fit_contents();
    #[expect(deprecated)]
    let height = harness.ctx.screen_rect().height().max(600.0);
    harness.set_size(egui::vec2(width, height));
    harness.run_steps(2);
    harness.snapshot(name);
}
