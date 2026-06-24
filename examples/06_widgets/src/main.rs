use eframe::egui;
use litui::*;

fn main() -> eframe::Result {
    let (render, mut state) = include_litui_ui!("content.md");

    eframe::run_ui_native("06 Widgets", Default::default(), move |ui, _| {
        egui_extras::install_image_loaders(ui.ctx());
        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                render(ui, &mut state);
            });
        });
    })
}
