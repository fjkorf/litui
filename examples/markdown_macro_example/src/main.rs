//! Minimal example: render a single markdown file with `include_markdown_ui!`.
//!
//! Demonstrates the simplest usage -- static markdown with no widgets.
//! The macro returns a closure that is called each frame inside a scroll area.

use eframe::egui;
use litui::*;

fn main() -> eframe::Result {
    eframe::run_native(
        "Markdown Macro Example",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(MyApp))),
    )
}

struct MyApp;

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);
        let generated = include_markdown_ui!("example.md");
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                generated(ui);
            });
        });
    }
}
