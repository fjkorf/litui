use eframe::egui;
use litui::*;

fn main() -> eframe::Result {
    eframe::run_simple_native("03 Tables", Default::default(), |ctx, _| {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let render = include_markdown_ui!("content.md");
                render(ui);
            });
        });
    })
}
