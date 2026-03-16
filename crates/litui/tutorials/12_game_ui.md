# Game UI

> Run it: `cargo run -p tut_12_game`

This tutorial combines everything into a **complete game UI** — character creation, inventory, monster info, and a persistent stat panel.

## What's new

All previous concepts applied together: multi-page navigation, container panels, dynamic data, runtime styling, display-only pages, and styled frames.

## Architecture

```text
content/
  _app.md        — shared styles + widget configs (parent)
  char_create.md — character creation (default central page)
  inventory.md   — item list with ::: foreach
  monster_info.md — display-only monster card
  stats.md       — persistent right panel (panel: right)
```

## Character creation

```text
[select](chosen_species){species_list}
[select](chosen_job){job_list}

**Name:** [display](player_name)
```

Two scrollable `[select]` widgets for species and job, populated from `Vec<String>` fields. A display shows the computed character name.

## Inventory with foreach

```text
::: foreach items

| {letter} | {name} | {quantity} |
|-----------|--------|------------|

:::
```

Auto-generates `ItemsRow { letter, name, quantity }`. Populate from game state each frame.

## Persistent stat panel

```yaml
page:
  name: Stats
  label: Stats
  panel: right
  width: 180
```

Always visible on the right side. Shows HP/MP progress bars and dynamic HP color:

```text
[progress](hp_frac){hp_bar}

**HP:** [display](hp_text) ::$hp_color
```

The `::$hp_color` shorthand applies runtime color — set `state.hp_color = "hp_good".into()` when healthy, `"hp_danger".into()` when low.

## Monster info card

A display-only page — no input widgets, just `[display]` fields populated from code:

```text
# [display](monster_name) ::title

| Stat | Value |
|------|-------|
| **HP** | [display](monster_hp) |
| **AC** | [display](monster_ac) |

::: frame panel

[display](monster_desc)

:::
```

## Wiring it up

```rust,ignore
fn populate_data(state: &mut AppState) {
    state.player_name = format!("{} {}", species, job);
    state.hp_frac = 0.7;
    state.hp_text = format!("{}/{}", current_hp, max_hp);
    state.hp_color = if hp_frac > 0.5 { "hp_good" } else { "hp_danger" }.into();
}
```

One call to `self.md.show_all(ctx)` renders everything: stat panel, navigation, and the current central page.

## Expert tip

litui's module stratification shows `helpers/lib` and `macro/frontmatter` as the most foundational modules (stratification 0.17 and 0.25), while example apps are leaf nodes (2.00+). This DAG structure means changes to foundational modules ripple outward, while game-specific code is isolated. When building a game UI, your `.md` files and `main.rs` sit at the leaf level — you consume litui's API without touching its internals. The `AppState` struct is your bridge between litui's generated UI and your game logic.

## What we built

A complete roguelike-style game UI with character creation, inventory management, monster info cards, and a persistent stat sidebar — entirely from markdown files.
