use eframe::egui;
use litui::*;

mod pages {
    use egui;
    use litui::*;

    define_litui_app! {
        parent: "content/_app.md",
        "content/home.md",
        "content/settings.md",
    }
}

use pages::*;

fn main() -> eframe::Result {
    eframe::run_native(
        "09 Dynamic",
        Default::default(),
        Box::new(|_cc| {
            let mut md = LituiApp::default();
            populate(&mut md.state);
            Ok(Box::new(AppWrapper { md }))
        }),
    )
}

fn populate(state: &mut AppState) {
    state.show_details = true;
    state.status_text = "All systems operational".into();
    state.status_style = "success".into();

    state.items = vec![
        ItemsRow {
            name: "Iron Sword".into(),
            quantity: "1".into(),
            weight: "3.5 lb".into(),
        },
        ItemsRow {
            name: "Health Potion".into(),
            quantity: "5".into(),
            weight: "0.5 lb".into(),
        },
        ItemsRow {
            name: "Torch".into(),
            quantity: "3".into(),
            weight: "1.0 lb".into(),
        },
    ];
}

struct AppWrapper {
    md: LituiApp,
}

impl eframe::App for AppWrapper {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);
        egui::TopBottomPanel::top("nav").show(ctx, |ui| {
            ui.horizontal(|ui| {
                for &p in Page::ALL {
                    if ui
                        .selectable_label(self.md.current_page == p, p.label())
                        .clicked()
                    {
                        self.md.current_page = p;
                    }
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.md.show_page(ui);
            });
        });
    }
}
