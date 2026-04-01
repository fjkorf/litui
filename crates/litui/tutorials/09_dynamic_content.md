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

## Collapsible sections

Wrap content in an expandable/collapsible header using `::: collapsing`:

```text
::: collapsing "Inventory Details"

- Weight capacity: 50 lb
- Gold: 127

:::
```

The title can be a quoted string, an unquoted word, or a `{field}` reference (for foreach).

### State-tracked open/close

Append `{bool_field}` to sync the open/closed state with `AppState`:

```text
::: collapsing "Server Info" {show_server_info}

Server status details here.

:::
```

Auto-declares `show_server_info: bool` (default `false`). The app can programmatically open/close the section by setting this field, and user clicks update it back.

### Nesting

Collapsing inside collapsing works — each gets a unique egui ID:

```text
::: collapsing "Outer"

::: collapsing "Inner"

Nested content.

:::

:::
```

### Inside foreach

Use `{field}` for per-row collapsible headers:

```text
::: foreach bones

::: collapsing {name}

Bone details here.

:::

:::
```

## Tree rendering

Add `children` after the field name to render recursive tree structures:

```text
::: foreach bones children

::: collapsing {name}

{description}

:::

:::
```

This generates a row struct with `children: Vec<Self>`. The body renders recursively — each node's children are rendered with the same template beneath it. Populate from code:

```rust,ignore
let mut arm = BonesRow::default();
arm.name = "Arm".into();
arm.children.push(BonesRow {
    name: "Hand".into(),
    ..Default::default()
});
state.bones.push(arm);
```

Tree foreach works naturally with `::: collapsing` for expandable hierarchies like scene graphs, file browsers, and property trees.

## Widgets in Foreach Tables

You can use input widgets inside foreach blocks. They create typed fields on the row struct:

```text
::: foreach tasks

| [checkbox](done) | {title} | [button](remove){on_remove} |
|---|---|---|

:::
```

This generates:

```rust,ignore
pub struct TasksRow {
    pub done: bool,             // from [checkbox](done)
    pub title: String,          // from {title}
    pub on_remove_count: u32,   // from [button](remove){on_remove}
}
```

Each row gets its own `done: bool` and `on_remove_count: u32` fields. Populate them from your app code just like display fields. Widget configs (`{on_remove}`) reference the global frontmatter `widgets:` section, not per-row config.

## Expert tip

Runtime styling generates a `__resolve_style_color()` match function at compile time from all frontmatter styles that have a `color` property. At runtime, the field's string value is matched against style names: `"success" => Some(Color32::from_rgb(0, 204, 102))`, `"danger" => Some(Color32::from_rgb(255, 68, 68))`, etc. The color is applied via `ui.visuals_mut().override_text_color` — egui's clone-on-write style system ensures it only affects content within the block scope.

## What we built

Dynamic data tables, conditional sections, and runtime-colored text — driven by `AppState` fields.
