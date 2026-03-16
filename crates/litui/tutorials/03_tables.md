# Tables

> Run it: `cargo run -p tut_03_tables`

This tutorial adds **GFM tables** — pipe-delimited rows that render as striped `egui::Grid` widgets.

## What's new

Standard GitHub-Flavored Markdown table syntax. Headers render bold, rows alternate background color (striped).

## The markdown

```text
| Name | Role | Status |
|------|------|--------|
| Alice | Engineer | Active |
| Bob | Designer | On leave |
| Carol | Manager | Active |
```

## Inline formatting in cells

Tables support bold, italic, code, and links inside cells:

```text
| Feature | Syntax | Notes |
|---------|--------|-------|
| **Bold** | `**text**` | Double asterisks |
| *Italic* | `*text*` | Single asterisks |
| `Code` | backticks | Inline code |
| [Link](https://egui.rs) | `[text](url)` | Clickable |
```

## Expert tip

pulldown-cmark emits table events in a flat sequence: `Start(Table)`, `Start(TableHead)`, `Start(TableCell)`, text events, `End(TableCell)`, etc. The macro accumulates cell fragments into a 2D grid, then emits a single `egui::Grid::new(id).num_columns(N).striped(true).show(ui, |ui| { ... })` call. Each cell's fragments are flushed as inline `horizontal_wrapped` content. Grid IDs are auto-generated (`md_table_0`, `md_table_1`, ...) to avoid egui ID collisions.

## What we built

Tables with headers, striped rows, and inline formatting — rendered as egui Grid widgets.
