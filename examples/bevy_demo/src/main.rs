//! Full Bevy demo: litui multi-page app rendered via bevy_egui.
//!
//! This is the Bevy-native counterpart to `eframe_demo`. Same 7 pages,
//! same content, same `auto_unmute` ECS system — but running in Bevy's
//! app loop with bevy_egui providing the egui context.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use demo_content::{self, AppState, Page};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "litui Bevy Demo".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .init_resource::<Page>()
        .init_resource::<AppState>()
        .add_systems(Startup, setup)
        .add_systems(Update, demo_content::auto_unmute)
        .add_systems(EguiPrimaryContextPass, (render_nav, render_page).chain())
        .run();
}

fn setup(mut commands: Commands<'_, '_>) {
    commands.spawn(Camera2d);
}

/// Render the page navigation bar as a top panel.
fn render_nav(mut contexts: EguiContexts<'_, '_>, mut current: ResMut<'_, Page>) -> Result {
    egui_extras::install_image_loaders(contexts.ctx_mut()?);
    egui::TopBottomPanel::top("nav").show(contexts.ctx_mut()?, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.visuals_mut().button_frame = false;
            for &page in Page::ALL {
                if ui
                    .selectable_label(*current == page, page.label())
                    .clicked()
                {
                    *current = page;
                }
            }
        });
    });
    Ok(())
}

/// Render the currently selected page in the central panel.
fn render_page(
    mut contexts: EguiContexts<'_, '_>,
    current: Res<'_, Page>,
    mut state: ResMut<'_, AppState>,
) -> Result {
    egui::CentralPanel::default().show(contexts.ctx_mut()?, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| match *current {
            Page::About => demo_content::render_about(ui),
            Page::Text => demo_content::render_text(ui),
            Page::Lists => demo_content::render_lists(ui),
            Page::Tables => demo_content::render_tables(ui),
            Page::Styles => demo_content::render_styles(ui),
            Page::Form => demo_content::render_form(ui, &mut state),
            Page::Monitor => demo_content::render_monitor(ui, &state),
        });
    });
    Ok(())
}
