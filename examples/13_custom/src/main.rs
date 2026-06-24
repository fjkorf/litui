//! Spike: prove the `[custom](slot)` escape hatch works in the Bevy path.
//!
//! `define_litui_app!` generates an `AppState` that, for each
//! `[custom](slot)` directive, contains a field of type
//! `Option<Box<dyn FnMut(&mut egui::Ui) + Send + Sync>>`. The user fills those
//! slots in at startup; the generated render functions invoke them via
//! take/replace so the closures can draw raw egui inline with Markdown.
//!
//! This example exercises two cases RustRetro cares about:
//!   1. `demo_slot` — a custom widget embedded inside an otherwise-Markdown page.
//!   2. `panel_slot` — a page that is *essentially just* a custom slot (a whole
//!      bespoke panel living as a litui page).

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

mod pages {
    use egui;
    use litui::*;

    define_litui_app! {
        parent: "content/_app.md",
        "content/page.md",
        "content/panel.md",
    }
}

use pages::*;

// The generated `Page` and `AppState` live in this crate (emitted by the
// macro). Bevy 0.19 requires `Resource: Component`, so wrap them in newtypes
// that derive `Resource` (the derive also satisfies the `Component` supertrait).
#[derive(Resource, Default)]
struct PageRes(Page);
#[derive(Resource, Default)]
struct AppStateRes(AppState);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "13 Custom".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .init_resource::<PageRes>()
        .init_resource::<AppStateRes>()
        .add_systems(Startup, (setup, install_custom_slots))
        .add_systems(EguiPrimaryContextPass, (render_nav, render_page).chain())
        .run();
}

fn setup(mut commands: Commands<'_, '_>) {
    commands.spawn(Camera2d);
}

/// Fill the custom slots. This is the user-facing API:
/// `state.<slot> = Some(Box::new(|ui| { /* raw egui */ }))`.
fn install_custom_slots(mut state: ResMut<'_, AppStateRes>) {
    // Case 1: a custom widget inside a Markdown page.
    state.0.demo_slot = Some(Box::new(|ui| {
        eprintln!("[13_custom] demo_slot closure invoked");
        ui.label("custom egui here");
        let _ = ui.button("native");
    }));

    // Case 2: a whole bespoke panel as a litui page. A real RustRetro panel
    // would capture an `Arc<Mutex<EmulatorState>>` here (Send + Sync), which
    // satisfies the required bound.
    let mut frame_counter: u64 = 0;
    state.0.panel_slot = Some(Box::new(move |ui| {
        frame_counter += 1; // proves FnMut: the closure mutates captured state
        ui.heading("Bespoke Panel (raw egui)");
        ui.label(format!("rendered frame #{frame_counter}"));
        ui.separator();
        ui.horizontal(|ui| {
            let _ = ui.button("disassemble");
            let _ = ui.button("step");
        });
    }));
}

// bevy_egui 0.40 exposes only &Context (no root Ui), so ctx-based panel show stays; egui 0.34 deprecated it.
#[allow(deprecated)]
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

// bevy_egui 0.40 exposes only &Context (no root Ui), so ctx-based panel show stays; egui 0.34 deprecated it.
#[allow(deprecated)]
fn render_page(
    mut ctxs: EguiContexts<'_, '_>,
    current: Res<'_, PageRes>,
    mut state: ResMut<'_, AppStateRes>,
) -> Result {
    egui::CentralPanel::default().show(ctxs.ctx_mut()?, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| match current.0 {
            // Both pages have custom slots, so both take `&mut AppState`.
            Page::Mixed => render_mixed(ui, &mut state.0),
            Page::Panel => render_panel(ui, &mut state.0),
        });
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::pages::*;
    use std::cell::RefCell;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Headless proof that the custom-slot closure actually runs when the
    /// generated render function is invoked (no window required).
    #[test]
    fn custom_slot_closure_is_invoked() {
        let calls = Arc::new(AtomicUsize::new(0));

        let mut state = AppState::default();
        let counter = calls.clone();
        state.panel_slot = Some(Box::new(move |ui| {
            counter.fetch_add(1, Ordering::SeqCst);
            ui.label("invoked from test");
        }));

        // `__run_test_ui` takes an `Fn` closure, so use a RefCell to get
        // interior mutability for the `&mut AppState` the render fn needs.
        let state = RefCell::new(state);
        // Render the "whole page is a custom slot" page twice headlessly.
        egui::__run_test_ui(|ui| render_panel(ui, &mut state.borrow_mut()));
        egui::__run_test_ui(|ui| render_panel(ui, &mut state.borrow_mut()));

        assert_eq!(
            calls.load(Ordering::SeqCst),
            2,
            "panel_slot closure should run once per render call"
        );

        // The slot must be put back after each call (take/replace pattern).
        assert!(
            state.borrow().panel_slot.is_some(),
            "slot must be restored after rendering"
        );
    }

    /// Proves the slot composes inside a Markdown page (case 1) and that an
    /// unset slot renders nothing without panicking.
    #[test]
    fn mixed_page_slot_optional() {
        let mut state = AppState::default();
        // demo_slot left as None — render must not panic.
        {
            let state = RefCell::new(&mut state);
            egui::__run_test_ui(|ui| render_mixed(ui, &mut state.borrow_mut()));
        }

        let ran = Arc::new(AtomicUsize::new(0));
        let c = ran.clone();
        state.demo_slot = Some(Box::new(move |ui| {
            c.fetch_add(1, Ordering::SeqCst);
            ui.label("x");
        }));
        let state = RefCell::new(&mut state);
        egui::__run_test_ui(|ui| render_mixed(ui, &mut state.borrow_mut()));
        assert_eq!(ran.load(Ordering::SeqCst), 1);
    }
}
