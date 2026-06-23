//! Game demo: a 3-page roguelike companion using litui widgets.

use eframe::egui;

mod pages {
    use egui;
    use litui::*;

    define_markdown_app! {
        parent: "content/_app.md",
        "content/char_create.md",
        "content/inventory.md",
        "content/monster_info.md",
        "content/stats.md",
    }
}
use pages::*;

fn main() -> eframe::Result {
    eframe::run_native(
        "Game Demo",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(GameApp::default()))),
    )
}

struct GameApp {
    md: MdApp,
}

impl Default for GameApp {
    fn default() -> Self {
        let mut md = MdApp::default();
        populate_data(&mut md.state);
        Self { md }
    }
}

const SPECIES: &[&str] = &["Human", "Elf", "Dwarf", "Orc", "Halfling"];
const JOBS: &[&str] = &["Fighter", "Wizard", "Rogue", "Cleric", "Ranger"];
const BASE_HP: &[u32] = &[10, 8, 12, 11, 7];
const BASE_STR: &[u32] = &[10, 8, 12, 14, 6];
const BASE_DEX: &[u32] = &[10, 14, 8, 6, 16];
const JOB_HP: &[i32] = &[4, -2, 0, 2, 1];
const JOB_STR: &[i32] = &[3, -1, 1, 0, 2];
const JOB_DEX: &[i32] = &[0, 1, 4, -1, 3];

fn populate_data(state: &mut AppState) {
    state.species_list = SPECIES.iter().map(|s| (*s).into()).collect();
    state.job_list = JOBS.iter().map(|s| (*s).into()).collect();

    let si = state.chosen_species.min(SPECIES.len().saturating_sub(1));
    let ji = state.chosen_job.min(JOBS.len().saturating_sub(1));

    state.preview_name = format!("{} {}", SPECIES[si], JOBS[ji]);
    let hp = (BASE_HP[si] as i32 + JOB_HP[ji]).max(1) as u32;
    let str_val = (BASE_STR[si] as i32 + JOB_STR[ji]).max(1) as u32;
    let dex_val = (BASE_DEX[si] as i32 + JOB_DEX[ji]).max(1) as u32;
    state.preview_hp = format!("{hp}");
    state.preview_str = format!("{str_val}");
    state.preview_dex = format!("{dex_val}");

    state.gold = "42".into();

    if state.inv_items.is_empty() {
        state.inv_items = vec![
            Inv_itemsRow {
                letter: "a".into(),
                name: "Iron Sword".into(),
                qty: "1".into(),
            },
            Inv_itemsRow {
                letter: "b".into(),
                name: "Health Potion".into(),
                qty: "3".into(),
            },
            Inv_itemsRow {
                letter: "c".into(),
                name: "Gold Coin".into(),
                qty: "42".into(),
            },
        ];
    }

    state.monster_name = "Goblin".into();
    state.monster_hd = "1d8".into();
    state.monster_hp = "6".into();
    state.monster_ac = "15".into();
    state.monster_speed = "30 ft".into();
    state.monster_attack = "Scimitar +4 (1d6+2)".into();
    state.monster_desc = "A small, cunning humanoid with green skin and sharp teeth. \
        Goblins prefer ambushes and dirty tricks over fair fights."
        .into();

    // Stats panel
    state.player_name = format!("{} {}", SPECIES[si], JOBS[ji]);
    state.hp_frac = 0.7;
    state.mp_frac = 0.4;
    state.xl = "5".into();
    state.hp_text = format!("{}/{}", (state.hp_frac * 100.0) as u32, 100);
    state.hp_color = if state.hp_frac > 0.5 {
        "hp_good"
    } else if state.hp_frac > 0.25 {
        "hp_warn"
    } else {
        "hp_danger"
    }
    .into();
}

impl eframe::App for GameApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        populate_data(&mut self.md.state);
        self.md.show_all(ctx);
    }
}
