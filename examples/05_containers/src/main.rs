use eframe::egui;
use litui::*;

fn main() -> eframe::Result {
    eframe::run_ui_native("05 Containers", Default::default(), |ui, _| {
        egui_extras::install_image_loaders(ui.ctx());
        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let render = include_litui_ui!("content.md");
                render(ui);
            });
        });
    })
}
