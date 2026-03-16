//! Example: stateful widgets with `include_markdown_ui!`.
//!
//! Demonstrates the tuple return pattern for markdown files containing
//! interactive widgets (slider, checkbox, textedit, dragvalue). The macro
//! returns `(render_fn, MdFormState)` which is destructured and used each
//! frame. Hand-coded submit/reset buttons and live state display show how
//! to mix macro-generated and manual UI code.

use eframe::egui;
use litui::*;

fn main() -> eframe::Result {
    eframe::run_native(
        "Widgets Example",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(build_app()))),
    )
}

fn build_app() -> impl eframe::App {
    let (render, state) = include_markdown_ui!("form.md");
    let mut state = state;
    let default_state = state.clone();
    let mut submit_count: u32 = 0;

    ClosureApp {
        f: move |ui: &mut egui::Ui| {
            // Render the markdown form — sliders, checkboxes, etc. mutate `state`
            render(ui, &mut state);

            // Buttons with click handling (outside markdown so we can check Response)
            ui.horizontal(|ui| {
                if ui
                    .button(
                        egui::RichText::new("Submit")
                            .strong()
                            .color(egui::Color32::from_rgb(0, 170, 0)),
                    )
                    .clicked()
                {
                    submit_count += 1;
                    state = default_state.clone();
                }
                if ui
                    .button(
                        egui::RichText::new("Reset All")
                            .strong()
                            .color(egui::Color32::from_rgb(255, 68, 68)),
                    )
                    .clicked()
                {
                    state = default_state.clone();
                    submit_count = 0;
                }
            });

            // --- Live state display ---
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);
            ui.heading("Live State");
            ui.add_space(4.0);

            // Volume slider → progress bar (muted checkbox zeroes it out)
            let effective_vol = if state.muted {
                0.0
            } else {
                state.volume / 100.0
            };
            ui.horizontal(|ui| {
                ui.label("Effective volume:");
                ui.add(
                    egui::ProgressBar::new(effective_vol as f32)
                        .show_percentage()
                        .animate(!state.muted),
                );
            });
            if state.muted {
                ui.label(
                    egui::RichText::new("MUTED")
                        .color(egui::Color32::from_rgb(255, 68, 68))
                        .strong(),
                );
            }

            ui.add_space(4.0);

            // Username text input → greeting
            if state.username.is_empty() {
                ui.label(egui::RichText::new("No username entered").weak().italics());
            } else {
                ui.horizontal(|ui| {
                    ui.label("Hello,");
                    ui.label(
                        egui::RichText::new(&state.username)
                            .strong()
                            .color(egui::Color32::from_rgb(68, 136, 255)),
                    );
                    ui.label("!");
                });
            }

            // DragValue → count display
            ui.label(format!("Count: {:.1}", state.count));

            ui.add_space(4.0);

            // New widgets state
            const QUALITY: &[&str] = &["Low", "Medium", "High", "Ultra"];
            const OUTPUTS: &[&str] = &["Speakers", "Headphones", "Bluetooth"];
            ui.label(format!("Quality: {}", QUALITY[state.quality]));
            ui.label(format!("Output: {}", OUTPUTS[state.output]));
            ui.horizontal(|ui| {
                ui.label("Accent color:");
                let [r, g, b, _] = state.accent_color;
                let color = egui::Color32::from_rgb(r, g, b);
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 2.0, color);
                ui.label(format!("#{r:02X}{g:02X}{b:02X}"));
            });

            // Tier 1+2 widgets
            const VIEWS: &[&str] = &["Grid", "List", "Board"];
            ui.label(format!("Dark mode: {}", state.dark_mode));
            ui.label(format!("View: {}", VIEWS[state.view_mode]));
            if !state.notes.is_empty() {
                ui.label(format!("Notes: {} chars", state.notes.len()));
            }
            if !state.api_key.is_empty() {
                ui.label(format!("API key: {} chars (hidden)", state.api_key.len()));
            }

            ui.add_space(4.0);
            ui.label(format!("Submitted {} time(s)", submit_count));
        },
    }
}

/// Minimal App wrapper that delegates to an FnMut closure.
struct ClosureApp<F: FnMut(&mut egui::Ui)> {
    f: F,
}

impl<F: FnMut(&mut egui::Ui)> eframe::App for ClosureApp<F> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                (self.f)(ui);
            });
        });
    }
}
