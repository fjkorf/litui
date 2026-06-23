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

        .add_systems(Startup, setup)
        .add_systems(Startup, setup_mdapp)
        .add_systems(EguiPrimaryContextPass, render_markdown)
        .run();
}

fn setup(mut commands: Commands<'_, '_>) {
    commands.spawn(Camera2d);
}

fn setup_mdapp(mut commands: Commands<'_, '_>) {
    commands.insert_resource(pages::MdApp::default());
}

fn render_markdown(mut ctxs: EguiContexts<'_, '_>, mut md: ResMut<'_, pages::MdApp>) -> Result {
    egui_extras::install_image_loaders(ctxs.ctx_mut()?);
    md.show_all(ctxs.ctx_mut()?);
    Ok(())
}
