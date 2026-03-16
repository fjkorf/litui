# Game UI Patterns

> Run it: `cargo run -p game_demo`

litui was designed for game UI. This tutorial shows the three most common screens in a
roguelike — character creation, inventory, and monster info — all built from markdown.

## Character creation with `[select]`

Two scrollable lists for species and job selection, plus a stat preview:

```text
| | |
|---|---|
| [select](chosen_species){species_list} | [select](chosen_job){job_list} |

## Stats

| Stat | Value |
|------|-------|
| **Name** | [display](preview_name) |
| **HP** | [display](preview_hp) |
```

The `[select]` widgets generate `Vec<String>` + `usize` fields on AppState.
Populate them from your data files at startup:

```rust,ignore
state.species_list = species_data.iter().map(|s| s.name.clone()).collect();
state.job_list = job_data.iter().map(|j| j.name.clone()).collect();
```

Then compute the stat preview each frame based on the selection:

```rust,ignore
let species = &species_data[state.chosen_species];
let job = &job_data[state.chosen_job];
state.preview_name = format!("{} {}", species.name, job.name);
state.preview_hp = format!("{}", species.base_hp + job.hp_bonus);
```

## Inventory with `::: foreach`

Dynamic item lists that change every frame:

```text
**Gold:** [display](gold)

::: foreach inv_items

| {letter} | {name} | {qty} |
|-----------|--------|-------|

:::
```

Remember the blank lines around the table — CommonMark needs them.

The macro generates `InvItemsRow { letter, name, qty }` on AppState. Populate from
your ECS inventory component:

```rust,ignore
state.inv_items.clear();
for (i, item) in inventory.items.iter().enumerate() {
    let mut row = InvItemsRow::default();
    row.letter = format!("{}", (b'a' + i as u8) as char);
    row.name = item.name.clone();
    row.qty = format!("x{}", item.quantity);
    state.inv_items.push(row);
}
state.gold = format!("{}", inventory.gold);
```

When the inventory is empty, the foreach renders nothing — no crash, no blank table.

## Monster info with `[display]`

Pure display-only page — all fields populated from code, no input widgets:

```text
# [display](monster_name)

| Stat | Value |
|------|-------|
| **HD** | [display](hd) |
| **HP** | [display](hp) |
| **AC** | [display](ac) |
| **Speed** | [display](speed) |
```

Display widgets self-declare as `String` when no input widget exists. Populate them
from your ECS monster component:

```rust,ignore
state.monster_name = monster.name.clone();
state.hp = format!("{}/{}", monster.hp.current, monster.hp.max);
state.ac = monster.ac.to_string();
```

## Stat panel with containers

The stat panel stays visible while you switch between character creation and inventory. Use `panel: right`:

```yaml
page:
  name: Stats
  label: Stats
  panel: right
  width: 180
```

```text
[progress](hp_frac){hp_bar}
[progress](mp_frac){mp_bar}
**XL:** [display](xl) ::stat
```

With `fill: "#8B0000"` on the `hp_bar` config, the HP bar renders in dark red.

### Dynamic HP color

Wrap HP text in a `::: style` block for runtime color:

```text
::: style hp_color

**HP:** [display](hp_text)

:::
```

Set `state.hp_color` to `"hp_good"`, `"hp_warn"`, or `"hp_danger"` based on health percentage. The text color changes every frame.

## Wiring it up

Use `define_markdown_app!` for multi-page navigation:

```rust,ignore
mod pages {
    use egui;
    use litui::*;

    define_markdown_app! {
        parent: "content/_app.md",
        "content/char_create.md",
        "content/inventory.md",
        "content/monster_info.md",
    }
}
```

In your eframe `update()`:

```rust,ignore
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    populate_game_data(&mut self.md.state);
    self.md.show_all(ctx);
}
```

`show_all()` handles the stat panel (right side), navigation (top), central content, and popup windows — all from the `panel:` config in each page's frontmatter.

## What's NOT litui

Some screens are better as hand-coded egui:
- The dungeon grid (tile rendering, mouse picking)
- The minimap (custom drawing)
- The targeting overlay (transparent layer over the game view)
- Complex equipment/paper doll UI

litui handles the **text-heavy, data-driven** screens. Hand-code the **spatial, interactive** ones.

## Previous / Next

Previous: [Bevy Integration](crate::_tutorial::_10_bevy_integration)

This is the final tutorial. For API reference, see the [main crate docs](crate).
