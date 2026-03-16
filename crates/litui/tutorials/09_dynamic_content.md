# Dynamic Content

> Run it: `cargo run -p tut_09_dynamic`

This tutorial adds **runtime data** — iterating collections, conditional rendering, and dynamic styling.

## What's new

Block directives `::: foreach`, `::: if`, and `::: style` generate runtime control flow. The `::$field` shorthand applies runtime styles to individual paragraphs.

## Foreach iteration

Render a collection as a table:

```text
::: foreach items

| {name} | {quantity} | {weight} |
|--------|-----------|----------|

:::
```

This auto-generates:

```rust,ignore
pub struct ItemsRow {
    pub name: String,
    pub quantity: String,
    pub weight: String,
}
// On AppState: pub items: Vec<ItemsRow>
```

Populate from code:

```rust,ignore
state.items.push(ItemsRow {
    name: "Iron Sword".into(),
    quantity: "1".into(),
    weight: "3.5 lb".into(),
});
```

**Important:** Blank lines are required around the table inside foreach — CommonMark needs them for block-level elements.

## Conditional rendering

Show content based on a bool field:

```text
::: if show_details

Details visible only when `show_details` is true.

:::
```

Auto-declares `show_details: bool` on `AppState`.

## Runtime styling

Wrap content in a dynamic color override:

```text
::: style status_color

Server is operational.

:::
```

Set `state.status_color = "success".into()` to apply the `success` style's color at runtime.

For single paragraphs, use the `::$field` shorthand:

```text
Server status: [display](status_text) ::$status_style
```

Auto-declares `status_style: String`. Only the `color` property is applied at runtime.

## Expert tip

Runtime styling generates a `__resolve_style_color()` match function at compile time from all frontmatter styles that have a `color` property. At runtime, the field's string value is matched against style names: `"success" => Some(Color32::from_rgb(0, 204, 102))`, `"danger" => Some(Color32::from_rgb(255, 68, 68))`, etc. The color is applied via `ui.visuals_mut().override_text_color` — egui's clone-on-write style system ensures it only affects content within the block scope.

## What we built

Dynamic data tables, conditional sections, and runtime-colored text — driven by `AppState` fields.
