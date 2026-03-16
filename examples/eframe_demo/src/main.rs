//! eframe demo: litui multi-page app with bevy_ecs state management.
//!
//! This is the eframe counterpart to `bevy_demo`. Same 7 pages, same content,
//! same `auto_unmute` ECS system — running in eframe's app loop with a manually
//! owned bevy_ecs::World for state management.

use bevy_ecs::prelude::*;
use demo_content::{self, AppState, Page};
use eframe::egui;

fn main() -> eframe::Result {
    eframe::run_native(
        "litui eframe Demo",
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(DemoApp::new()))
        }),
    )
}

struct DemoApp {
    world: World,
    schedule: Schedule,
}

impl DemoApp {
    fn new() -> Self {
        let mut world = World::new();
        world.insert_resource(Page::default());
        world.insert_resource(AppState::default());

        let mut schedule = Schedule::default();
        schedule.add_systems(demo_content::auto_unmute);

        Self { world, schedule }
    }
}

impl eframe::App for DemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.schedule.run(&mut self.world);

        egui::TopBottomPanel::top("demo_top_bar").show(ctx, |ui| {
            let current = *self.world.resource::<Page>();
            ui.horizontal_wrapped(|ui| {
                ui.visuals_mut().button_frame = false;
                for &page in Page::ALL {
                    if ui.selectable_label(current == page, page.label()).clicked() {
                        *self.world.resource_mut::<Page>() = page;
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let current = *self.world.resource::<Page>();
                match current {
                    Page::About => demo_content::render_about(ui),
                    Page::Text => demo_content::render_text(ui),
                    Page::Lists => demo_content::render_lists(ui),
                    Page::Tables => demo_content::render_tables(ui),
                    Page::Styles => demo_content::render_styles(ui),
                    Page::Form => {
                        self.world
                            .resource_scope(|_world, mut state: Mut<'_, AppState>| {
                                demo_content::render_form(ui, &mut state);
                            });
                    }
                    Page::Monitor => {
                        let state = self.world.resource::<AppState>();
                        demo_content::render_monitor(ui, state);
                    }
                }
            });
        });
    }
}
