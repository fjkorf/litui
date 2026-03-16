use eframe::egui;
use litui::*;

fn main() -> eframe::Result {
    eframe::run_simple_native("04 Images", Default::default(), |ctx, _| {
        egui_extras::install_image_loaders(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let render = include_litui_ui!("content.md");
                render(ui);
            });
        });
    })
}
