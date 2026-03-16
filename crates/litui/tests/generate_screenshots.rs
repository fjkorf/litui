//! Generate tutorial screenshot PNGs from fixtures.
//!
//! Run with: `cargo test -p litui --test generate_screenshots`
//!
//! Renders each fixture markdown file headlessly via egui_kittest and saves
//! the result as a PNG in `src/_tutorial/img/`. These PNGs are referenced
//! by the tutorial markdown files and rendered inline by rustdoc.

use egui_kittest::Harness;
use litui::*;

const IMG_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/_tutorial/img");

fn render_and_save(name: &str, ui_fn: impl FnMut(&mut egui::Ui) + 'static) {
    let width = 600.0;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(width, 400.0))
        .with_max_steps(16)
        .wgpu()
        .build_ui(ui_fn);
    harness.run_steps(2);
    harness.fit_contents();
    #[expect(deprecated)]
    let height = harness.ctx.screen_rect().height().max(100.0).min(800.0);
    harness.set_size(egui::vec2(width, height));
    harness.run_steps(2);

    let img = harness.render().expect("render failed");
    let path = format!("{IMG_DIR}/{name}.png");
    img.save(&path)
        .unwrap_or_else(|e| panic!("Failed to save {path}: {e}"));
}

#[test]
fn hello_headings() {
    render_and_save(
        "hello_headings",
        include_markdown_ui!("fixtures/hello_headings.md"),
    );
}

#[test]
fn hello_lists() {
    render_and_save(
        "hello_lists",
        include_markdown_ui!("fixtures/hello_lists.md"),
    );
}

#[test]
fn styles_basic() {
    render_and_save(
        "styles_basic",
        include_markdown_ui!("fixtures/styles_basic.md"),
    );
}

#[test]
fn tables_basic() {
    render_and_save(
        "tables_basic",
        include_markdown_ui!("fixtures/tables_basic.md"),
    );
}

#[test]
fn tables_formatted() {
    render_and_save(
        "tables_formatted",
        include_markdown_ui!("fixtures/tables_formatted.md"),
    );
}

#[test]
fn tables_widgets() {
    render_and_save(
        "tables_widgets",
        include_markdown_ui!("fixtures/tables_widgets.md"),
    );
}

#[test]
fn widgets_form() {
    let (render, mut state) = include_markdown_ui!("fixtures/widgets_form.md");
    render_and_save("widgets_form", move |ui| render(ui, &mut state));
}

#[test]
fn widgets_new() {
    let (render, mut state) = include_markdown_ui!("fixtures/widgets_new.md");
    render_and_save("widgets_new", move |ui| render(ui, &mut state));
}

#[test]
fn selectors() {
    render_and_save("selectors", include_markdown_ui!("fixtures/selectors.md"));
}

#[test]
fn styled_blockquote() {
    render_and_save(
        "styled_blockquote",
        include_markdown_ui!("fixtures/styled_blockquote.md"),
    );
}

#[test]
fn styled_list() {
    render_and_save(
        "styled_list",
        include_markdown_ui!("fixtures/styled_list.md"),
    );
}

#[test]
fn selectors_full() {
    render_and_save(
        "selectors_full",
        include_markdown_ui!("fixtures/selectors_full.md"),
    );
}

#[test]
fn image_widget() {
    let render = include_markdown_ui!("fixtures/image_widget.md");
    let mut loaders_installed = false;
    render_and_save("image_widget", move |ui| {
        if !loaders_installed {
            egui_extras::install_image_loaders(ui.ctx());
            loaders_installed = true;
        }
        render(ui);
    });
}
