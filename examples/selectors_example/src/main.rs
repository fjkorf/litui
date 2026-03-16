//! Example: CSS-like selectors, inline styled spans, and styled containers.
//!
//! Demonstrates `.class` selectors on buttons, `[.class](<text>)` inline spans,
//! and `{key}` styling on blockquotes and list items (coloring bars and bullets).

use eframe::egui;
use litui::*;

fn main() -> eframe::Result {
    eframe::run_native(
        "Selectors Example",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(MyApp))),
    )
}

struct MyApp;

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let render = include_markdown_ui!("content.md");
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                render(ui);
            });
        });
    }
}
