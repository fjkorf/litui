use eframe::egui;
use litui::*;

fn main() -> eframe::Result {
    let (render, mut state) = include_markdown_ui!("content.md");

    eframe::run_simple_native("07 Layout", Default::default(), move |ctx, _| {
        egui_extras::install_image_loaders(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                render(ui, &mut state);
            });
        });
    })
}
