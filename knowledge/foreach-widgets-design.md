# Foreach Widgets — Implementation Record

**Status: Implemented**

## Summary

Input widgets (checkbox, button, textedit, etc.) now work inside `::: foreach` blocks. The implementation added typed row fields so that widgets generate the correct field types on the row struct instead of the main state struct, and the foreach loop iterates mutably so widgets can modify per-row state.

## What was built

- **`RowField` enum in AST** (`litui_parser/src/ast.rs`) — `Display` variant for `{field}` references (String) and `Widget` variant for input widgets (typed: `bool`, `u32`, `String`, `f64`, etc.)
- **Parser routes widget fields to foreach's `row_fields`** (`litui_parser/src/parse.rs`) — when in foreach scope, widget declarations go to the row struct instead of top-level state
- **`CodegenContext` struct** (`litui_macro/src/codegen_ast.rs`) — threads `in_foreach` flag through codegen so widget code emits `__row.field` instead of `state.field`
- **Mutable foreach loop** — iterates `&mut state.field` so widgets can modify per-row values
- **Typed row struct fields** — `bool` for checkbox/toggle, `u32` for button counts, `String` for textedit/textarea/display, `f64` for slider/dragvalue, etc.

## Pipeline changes

| Layer | File | Change |
|-------|------|--------|
| AST | `litui_parser/src/ast.rs` | Added `RowField` enum with `Display` and `Widget` variants |
| Parser | `litui_parser/src/parse.rs` | Tracks foreach scope, routes widgets to `row_fields` |
| Bridge | `litui_macro/src/parse.rs` | Updated `WidgetField::Foreach` bridge type for `RowField` |
| Codegen | `litui_macro/src/codegen.rs` | Generates typed row fields from `RowField` |
| Runtime | `litui_macro/src/codegen_ast.rs` | `CodegenContext` with `in_foreach`, mutable loop (`&mut`), scoped widget references (`__row.field`) |

## Supported widgets in foreach

| Widget | Row field type | Example |
|--------|---------------|---------|
| `[checkbox](done)` | `done: bool` | Toggle per-row |
| `[toggle](active)` | `active: bool` | Toggle per-row |
| `[button](Delete){on_delete}` | `on_delete_count: u32` | Click counter per-row |
| `[textedit](name){cfg}` | `name: String` | Editable text per-row |
| `[textarea](notes){cfg}` | `notes: String` | Multi-line text per-row |
| `[slider](value){cfg}` | `value: f64` | Slider per-row |
| `[dragvalue](qty){cfg}` | `qty: f64` | Drag value per-row |
| `[display](label){cfg}` | `label: String` | Read-only display per-row |
| `[progress](pct){cfg}` | `pct: f64` | Progress bar per-row |

## egui ID uniqueness

Foreach tables hash row pointers for table IDs. Widget IDs include `(row_ptr, field_name)` to avoid ID collisions across rows.

## Backward compatibility

- Existing `{field}` display references are unchanged (still `String`)
- Widget syntax inside foreach is purely additive
- Widget config `{cfg}` references global frontmatter `widgets:` section (not per-row config)
