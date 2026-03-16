use eframe::egui;
use litui::*;

mod pages {
    use egui;
    use litui::*;

    define_litui_app! {
        parent: "content/_app.md",
        "content/about.md",
        "content/form.md",
        "content/monitor.md",
    }
}

use pages::*;

fn main() -> eframe::Result {
    eframe::run_native(
        "08 Multi-Page",
        Default::default(),
        Box::new(|_cc| {
            Ok(Box::new(AppWrapper {
                md: LituiApp::default(),
            }))
        }),
    )
}

struct AppWrapper {
    md: LituiApp,
}

impl eframe::App for AppWrapper {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);
        self.md.show_all(ctx);
    }
}
