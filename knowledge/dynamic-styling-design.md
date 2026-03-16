# Dynamic Styling and Container Directives — Design Research

## Status: Researched, not yet implemented

## Dynamic Styling

### Problem
Game UIs need text color/style to change based on runtime state (HP percentage → green/yellow/red). litui's current style system is 100% compile-time.

### Chosen Design: `::: style` block + `::$field` shorthand

**Block scope** — applies egui `ui.visuals_mut().override_text_color` to all content:
```markdown
::: style hp_style

**HP:** [display](hp_text)
**MP:** [display](mp_text)

:::
```

State: `hp_style: String` on AppState. Set to a frontmatter style name (`"hp_good"`, `"hp_danger"`, etc.) by game code each frame.

**Shorthand** — for single display elements:
```markdown
[display](hp_text)::$hp_style
```

### Generated Code

Both use a compile-time style lookup function generated from frontmatter:
```rust
fn __resolve_style_color(name: &str) -> Option<egui::Color32> {
    match name {
        "hp_good" => Some(egui::Color32::from_rgb(0, 204, 102)),
        "hp_warn" => Some(egui::Color32::from_rgb(255, 170, 0)),
        "hp_danger" => Some(egui::Color32::from_rgb(255, 68, 68)),
        _ => None,
    }
}
```

`::: style` block uses `ui.visuals_mut()` (egui-idiomatic scope). `::$field` uses `RichText::color()` on a single label.

### Limitation
Only `color` is dynamically applied via visuals override. Properties like `bold`, `italic`, `size` are text-level and can't be scoped via `ui.visuals_mut()`. For full StyleDef scoping, a runtime helper with RichText construction would be needed.

---

## Container Directives

### Problem
Every litui render call requires Rust boilerplate to create the egui container (SidePanel, Window, etc.).

### Chosen Design: `panel:` in PageDef frontmatter + `show_all(ctx)` method

```yaml
page:
  name: Stats
  label: Stats
  panel: right
  width: 180
```

Values: `left`, `right`, `top`, `bottom`, `window`, `central` (default).

### Behavior
- Side panels are **always visible** (persist across page switches)
- Windows appear when the current page matches
- CentralPanel pages use existing page dispatch
- `show_all(&egui::Context)` is a NEW method — `show_page(&mut Ui)` stays unchanged (non-breaking)

### egui Constraint
Panels and windows require `&egui::Context`, not `&mut Ui`. Rendering order: side panels → top/bottom panels → central panel → windows.

---

## Design Decisions (Resolved)

1. **`::: style` blocks nest** — inner scope overrides outer. egui's `visuals_mut()` naturally supports this since each scope sets its own override.
2. **Side panels persist** across page switches — always visible regardless of current page. This matches the game pattern (stat bar always shown).
3. **Windows auto-declare `{name}_open: bool`** on AppState — toggled by the window's close button. Game code can set it to show/hide the window programmatically.
