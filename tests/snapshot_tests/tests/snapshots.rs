#![expect(clippy::unwrap_used)]

use egui_kittest::Harness;
use litui_helpers::*;
use litui_macro::include_litui_ui;
use snapshot_tests::snapshot_markdown;

#[test]
fn example_md() {
    snapshot_markdown("example_md", include_litui_ui!("fixtures/example.md"));
}

#[test]
fn headings_only() {
    snapshot_markdown("headings_only", include_litui_ui!("fixtures/headings.md"));
}

#[test]
fn tables() {
    snapshot_markdown("tables", include_litui_ui!("fixtures/tables.md"));
}

#[test]
fn styled() {
    snapshot_markdown("styled", include_litui_ui!("fixtures/styled.md"));
}

#[test]
fn widgets() {
    snapshot_markdown("widgets", include_litui_ui!("fixtures/widgets.md"));
}

#[test]
fn selectors() {
    snapshot_markdown("selectors", include_litui_ui!("fixtures/selectors.md"));
}

#[test]
fn inline_styles() {
    snapshot_markdown(
        "inline_styles",
        include_litui_ui!("fixtures/inline_styles.md"),
    );
}

#[test]
fn widgets_in_table() {
    snapshot_markdown(
        "widgets_in_table",
        include_litui_ui!("fixtures/widgets_in_table.md"),
    );
}

#[test]
fn stateful() {
    // Macro returns (render_fn, default_state) tuple for stateful markdown
    let (render, mut state) = include_litui_ui!("fixtures/stateful.md");
    let width = 800.0;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(width, 600.0))
        .with_max_steps(16)
        .wgpu()
        .build_ui(move |ui| {
            render(ui, &mut state);
        });
    harness.run_steps(2);
    harness.fit_contents();
    #[expect(deprecated)]
    let height = harness.ctx.screen_rect().height().max(600.0);
    harness.set_size(egui::vec2(width, height));
    harness.run_steps(2);
    harness.snapshot("stateful");
}

#[test]
fn stateful_button() {
    let (render, mut state) = include_litui_ui!("fixtures/stateful_button.md");
    let width = 800.0;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(width, 600.0))
        .with_max_steps(16)
        .wgpu()
        .build_ui(move |ui| {
            render(ui, &mut state);
        });
    harness.run_steps(2);
    harness.fit_contents();
    #[expect(deprecated)]
    let height = harness.ctx.screen_rect().height().max(600.0);
    harness.set_size(egui::vec2(width, height));
    harness.run_steps(2);
    harness.snapshot("stateful_button");
}

#[test]
fn image() {
    let render = include_litui_ui!("fixtures/image.md");
    let width = 800.0;
    let mut loaders_installed = false;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(width, 600.0))
        .with_max_steps(16)
        .wgpu()
        .build_ui(move |ui| {
            if !loaders_installed {
                egui_extras::install_image_loaders(ui.ctx());
                loaders_installed = true;
            }
            render(ui);
        });
    harness.run_steps(4);
    harness.fit_contents();
    #[expect(deprecated)]
    let height = harness.ctx.screen_rect().height().max(600.0);
    harness.set_size(egui::vec2(width, height));
    harness.run_steps(4);
    harness.snapshot("image");
}

#[test]
fn styled_blockquote() {
    snapshot_markdown(
        "styled_blockquote",
        include_litui_ui!("fixtures/styled_blockquote.md"),
    );
}

#[test]
fn advanced_button() {
    let (render, mut state) = include_litui_ui!("fixtures/advanced_button.md");
    let width = 800.0;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(width, 600.0))
        .with_max_steps(16)
        .wgpu()
        .build_ui(move |ui| {
            render(ui, &mut state);
        });
    harness.run_steps(2);
    harness.fit_contents();
    #[expect(deprecated)]
    let height = harness.ctx.screen_rect().height().max(600.0);
    harness.set_size(egui::vec2(width, height));
    harness.run_steps(2);
    harness.snapshot("advanced_button");
}

#[test]
fn new_widgets() {
    let (render, mut state) = include_litui_ui!("fixtures/new_widgets.md");
    let width = 800.0;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(width, 600.0))
        .with_max_steps(16)
        .wgpu()
        .build_ui(move |ui| {
            render(ui, &mut state);
        });
    harness.run_steps(2);
    harness.fit_contents();
    #[expect(deprecated)]
    let height = harness.ctx.screen_rect().height().max(600.0);
    harness.set_size(egui::vec2(width, height));
    harness.run_steps(2);
    harness.snapshot("new_widgets");
}

#[test]
fn new_tier_widgets() {
    let (render, mut state) = include_litui_ui!("fixtures/new_tier_widgets.md");
    let width = 800.0;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(width, 600.0))
        .with_max_steps(16)
        .wgpu()
        .build_ui(move |ui| {
            render(ui, &mut state);
        });
    harness.run_steps(2);
    harness.fit_contents();
    #[expect(deprecated)]
    let height = harness.ctx.screen_rect().height().max(600.0);
    harness.set_size(egui::vec2(width, height));
    harness.run_steps(2);
    harness.snapshot("new_tier_widgets");
}

#[test]
fn display_only() {
    let (render, mut state) = include_litui_ui!("fixtures/display_only.md");
    // Populate display-only fields from "code" (simulating ECS/API data)
    state.monster_name = "Goblin".to_string();
    state.hp = "12/15".to_string();
    state.ac = "14".to_string();
    state.speed = "3.5".to_string();
    let width = 800.0;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(width, 600.0))
        .with_max_steps(16)
        .wgpu()
        .build_ui(move |ui| {
            render(ui, &mut state);
        });
    harness.run_steps(2);
    harness.fit_contents();
    #[expect(deprecated)]
    let height = harness.ctx.screen_rect().height().max(600.0);
    harness.set_size(egui::vec2(width, height));
    harness.run_steps(2);
    harness.snapshot("display_only");
}

#[test]
fn select_widget() {
    let (render, mut state) = include_litui_ui!("fixtures/select_widget.md");
    // Populate the list at "runtime"
    state.species_list = vec![
        "Human".to_string(),
        "Elf".to_string(),
        "Dwarf".to_string(),
        "Orc".to_string(),
        "Halfling".to_string(),
    ];
    state.chosen_species = 2; // Dwarf selected
    let width = 800.0;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(width, 600.0))
        .with_max_steps(16)
        .wgpu()
        .build_ui(move |ui| {
            render(ui, &mut state);
        });
    harness.run_steps(2);
    harness.fit_contents();
    #[expect(deprecated)]
    let height = harness.ctx.screen_rect().height().max(600.0);
    harness.set_size(egui::vec2(width, height));
    harness.run_steps(2);
    harness.snapshot("select_widget");
}

#[test]
fn foreach_table() {
    let (render, mut state) = include_litui_ui!("fixtures/foreach_table.md");
    // Populate inventory rows using push + field mutation
    // (can't name ItemsRow from outside the macro expansion)
    {
        let items = &mut state.items;
        for (letter, name, qty) in [
            ("a", "Iron Sword", "1"),
            ("b", "Health Potion", "3"),
            ("c", "Gold Coin", "42"),
        ] {
            items.push(Default::default());
            let row = items.last_mut().unwrap();
            row.letter = letter.to_string();
            row.name = name.to_string();
            row.quantity = qty.to_string();
        }
    }
    let width = 800.0;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(width, 600.0))
        .with_max_steps(16)
        .wgpu()
        .build_ui(move |ui| {
            render(ui, &mut state);
        });
    harness.run_steps(2);
    harness.fit_contents();
    #[expect(deprecated)]
    let height = harness.ctx.screen_rect().height().max(600.0);
    harness.set_size(egui::vec2(width, height));
    harness.run_steps(2);
    harness.snapshot("foreach_table");
}

#[test]
fn stateful_progress() {
    let (render, mut state) = include_litui_ui!("fixtures/stateful_progress.md");
    state.hp_frac = 0.65;
    state.mp_frac = 0.3;
    state.xp_frac = 0.8;
    let width = 800.0;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(width, 600.0))
        .with_max_steps(16)
        .wgpu()
        .build_ui(move |ui| {
            render(ui, &mut state);
        });
    harness.run_steps(2);
    harness.fit_contents();
    #[expect(deprecated)]
    let height = harness.ctx.screen_rect().height().max(600.0);
    harness.set_size(egui::vec2(width, height));
    harness.run_steps(2);
    harness.snapshot("stateful_progress");
}

#[test]
fn log_widget() {
    let (render, mut state) = include_litui_ui!("fixtures/log_widget.md");
    state.messages = vec![
        "You enter the dungeon.".to_string(),
        "A goblin appears!".to_string(),
        "You hit the goblin. (8 damage)".to_string(),
        "The goblin misses you.".to_string(),
        "You kill the goblin!".to_string(),
    ];
    let width = 800.0;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(width, 600.0))
        .with_max_steps(16)
        .wgpu()
        .build_ui(move |ui| {
            render(ui, &mut state);
        });
    harness.run_steps(2);
    harness.fit_contents();
    #[expect(deprecated)]
    let height = harness.ctx.screen_rect().height().max(600.0);
    harness.set_size(egui::vec2(width, height));
    harness.run_steps(2);
    harness.snapshot("log_widget");
}

#[test]
fn if_block_true() {
    let (render, mut state) = include_litui_ui!("fixtures/if_block.md");
    state.has_orb = true;
    let width = 800.0;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(width, 600.0))
        .with_max_steps(16)
        .wgpu()
        .build_ui(move |ui| {
            render(ui, &mut state);
        });
    harness.run_steps(2);
    harness.fit_contents();
    #[expect(deprecated)]
    let height = harness.ctx.screen_rect().height().max(600.0);
    harness.set_size(egui::vec2(width, height));
    harness.run_steps(2);
    harness.snapshot("if_block_true");
}

#[test]
fn if_block_false() {
    let (render, mut state) = include_litui_ui!("fixtures/if_block.md");
    state.has_orb = false;
    let width = 800.0;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(width, 600.0))
        .with_max_steps(16)
        .wgpu()
        .build_ui(move |ui| {
            render(ui, &mut state);
        });
    harness.run_steps(2);
    harness.fit_contents();
    #[expect(deprecated)]
    let height = harness.ctx.screen_rect().height().max(600.0);
    harness.set_size(egui::vec2(width, height));
    harness.run_steps(2);
    harness.snapshot("if_block_false");
}

#[test]
fn style_block() {
    let (render, mut state) = include_litui_ui!("fixtures/style_block.md");
    state.hp_style = "hp_danger".to_string();
    state.hp_text = "12/60".to_string();
    state.mp_text = "8/30".to_string();
    state.xl_text = "12".to_string();
    let width = 800.0;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(width, 600.0))
        .with_max_steps(16)
        .wgpu()
        .build_ui(move |ui| {
            render(ui, &mut state);
        });
    harness.run_steps(2);
    harness.fit_contents();
    #[expect(deprecated)]
    let height = harness.ctx.screen_rect().height().max(600.0);
    harness.set_size(egui::vec2(width, height));
    harness.run_steps(2);
    harness.snapshot("style_block");
}

#[test]
fn frame_style() {
    snapshot_markdown("frame_style", include_litui_ui!("fixtures/frame_style.md"));
}

#[test]
fn alignment() {
    snapshot_markdown("alignment", include_litui_ui!("fixtures/alignment.md"));
}

#[test]
fn table_alignment() {
    snapshot_markdown(
        "table_alignment",
        include_litui_ui!("fixtures/table_alignment.md"),
    );
}

#[test]
fn weighted_columns() {
    snapshot_markdown(
        "weighted_columns",
        include_litui_ui!("fixtures/weighted_columns.md"),
    );
}

#[test]
fn light_mode() {
    let render = include_litui_ui!("fixtures/light_mode.md");
    let width = 800.0;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(width, 600.0))
        .with_max_steps(16)
        .wgpu()
        .build_ui(move |ui| {
            ui.ctx().set_visuals(egui::Visuals::light());
            render(ui);
        });
    harness.run_steps(2);
    harness.fit_contents();
    #[expect(deprecated)]
    let height = harness.ctx.screen_rect().height().max(600.0);
    harness.set_size(egui::vec2(width, height));
    harness.run_steps(2);
    harness.snapshot("light_mode");
}

#[test]
fn foreach_checkbox() {
    let (render, mut state) = include_litui_ui!("fixtures/foreach_checkbox.md");
    // Populate rows with typed fields: done (bool), name (String), on_delete_count (u32)
    {
        let items = &mut state.items;
        items.push(Default::default());
        items.last_mut().unwrap().name = "Buy groceries".to_string();
        items.last_mut().unwrap().done = false;

        items.push(Default::default());
        items.last_mut().unwrap().name = "Send invoice".to_string();
        items.last_mut().unwrap().done = true;

        items.push(Default::default());
        items.last_mut().unwrap().name = "Fix bug".to_string();
        items.last_mut().unwrap().done = false;
    }
    let width = 800.0;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(width, 600.0))
        .with_max_steps(16)
        .wgpu()
        .build_ui(move |ui| {
            render(ui, &mut state);
        });
    harness.run_steps(2);
    harness.fit_contents();
    #[expect(deprecated)]
    let height = harness.ctx.screen_rect().height().max(600.0);
    harness.set_size(egui::vec2(width, height));
    harness.run_steps(2);
    harness.snapshot("foreach_checkbox");
}

#[test]
fn foreach_styled() {
    let (render, mut state) = include_litui_ui!("fixtures/foreach_styled.md");
    {
        let items = &mut state.items;
        // Row with danger style
        items.push(Default::default());
        items.last_mut().unwrap().name = "Overdue task".to_string();
        items.last_mut().unwrap().row_style = "danger".to_string();

        // Row with success style
        items.push(Default::default());
        items.last_mut().unwrap().name = "Completed task".to_string();
        items.last_mut().unwrap().row_style = "success".to_string();

        // Row with muted style
        items.push(Default::default());
        items.last_mut().unwrap().name = "Normal task".to_string();
        items.last_mut().unwrap().row_style = "muted".to_string();
    }
    let width = 800.0;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(width, 600.0))
        .with_max_steps(16)
        .wgpu()
        .build_ui(move |ui| {
            render(ui, &mut state);
        });
    harness.run_steps(2);
    harness.fit_contents();
    #[expect(deprecated)]
    let height = harness.ctx.screen_rect().height().max(600.0);
    harness.set_size(egui::vec2(width, height));
    harness.run_steps(2);
    harness.snapshot("foreach_styled");
}

#[test]
fn datepicker_widget() {
    let (render, mut state) = include_litui_ui!("fixtures/datepicker.md");
    state.chosen_date = chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
    let width = 800.0;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(width, 600.0))
        .with_max_steps(16)
        .wgpu()
        .build_ui(move |ui| {
            render(ui, &mut state);
        });
    harness.run_steps(2);
    harness.fit_contents();
    #[expect(deprecated)]
    let height = harness.ctx.screen_rect().height().max(600.0);
    harness.set_size(egui::vec2(width, height));
    harness.run_steps(2);
    harness.snapshot("datepicker_widget");
}
