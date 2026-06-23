use eframe::egui;
use litui::*;

mod pages {
    use egui;
    use litui::*;

    define_markdown_app! {
        parent: "content/_app.md",
        "content/showcase.md",
        "content/monitor.md",
    }
}

use pages::*;

fn main() -> eframe::Result {
    eframe::run_native(
        "10 Advanced",
        Default::default(),
        Box::new(|_cc| {
            let md = MdApp::default();
            Ok(Box::new(AppWrapper { md }))
        }),
    )
}


struct AppWrapper {
    md: MdApp,
}

impl eframe::App for AppWrapper {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);

        self.md.show_all(ctx);
    }
}
