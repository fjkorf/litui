//! Snapshot tests for the multi-page tutorial app using `define_litui_app!`.
//! Tests each generated page to verify rendering.

#![expect(clippy::unwrap_used)]

use snapshot_tests::snapshot_markdown;

mod demo {
    use litui_helpers::*;
    use litui_macro::define_litui_app;

    define_litui_app! {
        parent: "../../examples/08_multi_page/content/_app.md",
        "../../examples/08_multi_page/content/about.md",
        "../../examples/08_multi_page/content/form.md",
        "../../examples/08_multi_page/content/monitor.md",
    }
}

#[test]
fn demo_about() {
    let state = demo::AppState::default();
    snapshot_markdown("demo_about", move |ui| demo::render_about(ui, &state));
}

#[test]
fn demo_form() {
    let mut state = demo::AppState::default();
    snapshot_markdown("demo_form", move |ui| {
        demo::render_form(ui, &mut state);
    });
}

#[test]
fn demo_monitor_default() {
    let state = demo::AppState::default();
    snapshot_markdown("demo_monitor_default", move |ui| {
        demo::render_monitor(ui, &state);
    });
}

// Test [foreach] inside define_litui_app! (regression for __row scope bug)
mod foreach_app {
    use litui_helpers::*;
    use litui_macro::define_litui_app;

    define_litui_app! {
        "fixtures/foreach_page.md",
    }
}

#[test]
fn foreach_in_app() {
    let mut state = foreach_app::AppState::default();
    {
        let items = &mut state.items;
        for (letter, name, qty) in [("a", "Sword", "1"), ("b", "Potion", "3")] {
            items.push(Default::default());
            let row = items.last_mut().unwrap();
            row.letter = letter.to_string();
            row.name = name.to_string();
            row.qty = qty.to_string();
        }
    }
    snapshot_markdown("foreach_in_app", move |ui| {
        foreach_app::render_inventory(ui, &mut state);
    });
}

// Test game_app: [select], [foreach], and [display] in a multi-page define_litui_app!
mod game_app {
    use litui_helpers::*;
    use litui_macro::define_litui_app;

    define_litui_app! {
        parent: "fixtures/game_app/_styles.md",
        "fixtures/game_app/char_create.md",
        "fixtures/game_app/inventory.md",
        "fixtures/game_app/monster_info.md",
    }
}

#[test]
fn game_char_create() {
    let mut state = game_app::AppState::default();
    state.species_list = vec!["Human".to_string(), "Elf".to_string(), "Dwarf".to_string()];
    state.job_list = vec![
        "Warrior".to_string(),
        "Mage".to_string(),
        "Rogue".to_string(),
    ];
    state.preview_name = "Gandalf".to_string();
    state.preview_class = "Mage".to_string();
    snapshot_markdown("game_char_create", move |ui| {
        game_app::render_char_create(ui, &mut state);
    });
}

#[test]
fn game_inventory() {
    let mut state = game_app::AppState::default();
    state.gold = "250".to_string();
    {
        let items = &mut state.inv_items;
        for (letter, name, qty) in [
            ("a", "Iron Sword", "1"),
            ("b", "Health Potion", "5"),
            ("c", "Shield", "1"),
        ] {
            items.push(Default::default());
            let row = items.last_mut().unwrap();
            row.letter = letter.to_string();
            row.name = name.to_string();
            row.qty = qty.to_string();
        }
    }
    snapshot_markdown("game_inventory", move |ui| {
        game_app::render_inventory(ui, &mut state);
    });
}

#[test]
fn game_inventory_empty() {
    let mut state = game_app::AppState::default();
    state.gold = "0".to_string();
    snapshot_markdown("game_inventory_empty", move |ui| {
        game_app::render_inventory(ui, &mut state);
    });
}

#[test]
fn game_monster_info() {
    let mut state = game_app::AppState::default();
    state.monster_name = "Red Dragon".to_string();
    state.hp = "256".to_string();
    state.ac = "19".to_string();
    state.attack = "2d6+8 fire breath".to_string();
    snapshot_markdown("game_monster_info", move |ui| {
        game_app::render_monster_info(ui, &state);
    });
}
