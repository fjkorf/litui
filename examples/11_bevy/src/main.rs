use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use litui::*;

mod pages {
    use egui;
    use litui::*;

    define_markdown_app! {
        parent: "content/_app.md",
        "content/about.md",
        "content/form.md",
    }
}

use pages::*;

impl Resource for Page {}
impl Resource for AppState {}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "11 Bevy".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .init_resource::<Page>()
        .init_resource::<AppState>()
        .add_systems(Startup, setup)
        .add_systems(EguiPrimaryContextPass, (render_nav, render_page).chain())
        .run();
}

fn setup(mut commands: Commands<'_, '_>) {
    commands.spawn(Camera2d);
}

fn render_nav(mut ctxs: EguiContexts<'_, '_>, mut current: ResMut<'_, Page>) -> Result {
    egui_extras::install_image_loaders(ctxs.ctx_mut()?);
    egui::TopBottomPanel::top("nav").show(ctxs.ctx_mut()?, |ui| {
        ui.horizontal(|ui| {
            for &p in Page::ALL {
                if ui.selectable_label(*current == p, p.label()).clicked() {
                    *current = p;
                }
            }
        });
    });
    Ok(())
}

fn render_page(
    mut ctxs: EguiContexts<'_, '_>,
    current: Res<'_, Page>,
    mut state: ResMut<'_, AppState>,
) -> Result {
    egui::CentralPanel::default().show(ctxs.ctx_mut()?, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| match *current {
            Page::About => render_about(ui, &state),
            Page::Form => render_form(ui, &mut state),
        });
    });
    Ok(())
}
