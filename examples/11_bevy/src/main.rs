use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use litui::*;

mod pages {
    use egui;
    use litui::*;

    define_litui_app! {
        parent: "content/_app.md",
        "content/about.md",
        "content/form.md",
    }
}

use pages::*;

#[derive(Resource, Default)]
struct PageRes(Page);
#[derive(Resource, Default)]
struct AppStateRes(AppState);

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
        .init_resource::<PageRes>()
        .init_resource::<AppStateRes>()
        .add_systems(Startup, setup)
        .add_systems(EguiPrimaryContextPass, (render_nav, render_page).chain())
        .run();
}

fn setup(mut commands: Commands<'_, '_>) {
    commands.spawn(Camera2d);
}

fn render_nav(mut ctxs: EguiContexts<'_, '_>, mut current: ResMut<'_, PageRes>) -> Result {
    egui_extras::install_image_loaders(ctxs.ctx_mut()?);
    egui::TopBottomPanel::top("nav").show(ctxs.ctx_mut()?, |ui| {
        ui.horizontal(|ui| {
            for &p in Page::ALL {
                if ui.selectable_label(current.0 == p, p.label()).clicked() {
                    current.0 = p;
                }
            }
        });
    });
    Ok(())
}

fn render_page(
    mut ctxs: EguiContexts<'_, '_>,
    current: Res<'_, PageRes>,
    mut state: ResMut<'_, AppStateRes>,
) -> Result {
    egui::CentralPanel::default().show(ctxs.ctx_mut()?, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| match current.0 {
            Page::About => render_about(ui, &state.0),
            Page::Form => render_form(ui, &mut state.0),
        });
    });
    Ok(())
}
