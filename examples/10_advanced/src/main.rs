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
            let mut md = MdApp::default();
            populate(&mut md.state);
            Ok(Box::new(AppWrapper { md }))
        }),
    )
}

fn populate(state: &mut AppState) {
    state.hp_frac = 0.65;
    state.species_list = vec![
        "Human".into(),
        "Elf".into(),
        "Dwarf".into(),
        "Halfling".into(),
    ];
    state.messages = vec!["App started.".into(), "Loading complete.".into()];
}

struct AppWrapper {
    md: MdApp,
}

impl eframe::App for AppWrapper {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);

        // Update derived display fields
        let size_opts = ["Small", "Medium", "Large"];
        self.md.state.size_label = size_opts
            .get(self.md.state.size)
            .unwrap_or(&"?")
            .to_string();
        self.md.state.chosen_species_label = self
            .md
            .state
            .species_list
            .get(self.md.state.chosen_species)
            .cloned()
            .unwrap_or_else(|| "None".into());
        self.md.state.dark_label = if self.md.state.dark_mode {
            "ON".into()
        } else {
            "OFF".into()
        };
        self.md.state.on_submit_hovered_label = if self.md.state.on_submit_hovered {
            "yes".into()
        } else {
            "no".into()
        };
        self.md.state.angle_label = format!("{:.0}°", self.md.state.angle);

        self.md.show_all(ctx);
    }
}
