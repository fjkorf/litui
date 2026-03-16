# Tables

> Run it: `cargo run -p eframe_demo` — navigate to the Tables page

Tables use standard GFM (GitHub Flavored Markdown) syntax and render as `egui::Grid` with striped rows and bold headers.

## Basic table

```markdown
| Name   | Role       | Status  |
|--------|------------|---------|
| Alice  | Engineer   | Active  |
| Bob    | Designer   | Away    |
| Claire | PM         | Active  |
```

The separator row (`|---|---|---|`) is required — it tells pulldown-cmark this is a table, not just pipes in text. Alignment colons (`:---`, `:---:`, `---:`) are parsed but egui Grid cells are left-aligned by default.

![Simple table](img/tables_basic.png)

## Inline formatting in cells

Standard inline markdown works inside table cells:

```markdown
| Feature        | Status            | Notes                          |
|----------------|-------------------|--------------------------------|
| **Rendering**  | *Complete*        | `v0.33` release                |
| ~~Old parser~~ | Removed           | Replaced with pulldown-cmark   |
| Links          | [Docs](https://docs.rs) | External links work     |
| Mixed          | **bold** + `code` | Multiple formats in one cell   |
```

Bold, italic, strikethrough, inline code, and hyperlinks all work. They compose the same way they do in paragraphs.

![Formatted table](img/tables_formatted.png)

## Widgets inside tables

Table cells can contain widgets. This is useful for building settings panels and data entry forms:

```markdown
---
widgets:
  vol:
    min: 0
    max: 100
---

| Setting  | Control               | Current         |
|----------|-----------------------|-----------------|
| Volume   | [slider](volume){vol} | [display](volume) |
| Mute     | [checkbox](muted)     |                 |
| Status   | [progress](0.75)      | 75%             |
```

Display widgets read from the shared `AppState`, so the "Current" column updates live as the slider moves. This only works inside `define_markdown_app!` where `AppState` exists.

![Widgets in table](img/tables_widgets.png)

## How tables render

The macro translates GFM tables into `egui::Grid`:

- The grid ID is auto-generated from the table's position in the document
- Rows alternate with `grid.striped(true)` for readability
- Header row cells are rendered bold automatically
- Each cell is its own layout scope — widgets, formatted text, and plain text all work
- Cells flow left-to-right, rows top-to-bottom

## Table limitations

A few things to know:

- **No cell merging** — every row must have the same number of columns
- **No nested tables** — pulldown-cmark doesn't support it, neither does litui
- **No per-cell styling** — you can use inline markdown but not `::key` style presets on individual cells
- **Column width** — egui auto-sizes columns based on content; you can't set explicit widths

## Styled text in tables

While `::key` doesn't work on individual cells, inline markdown formatting gives you plenty of control:

```markdown
| Priority | Task                          |
|----------|-------------------------------|
| **HIGH** | Fix crash on startup          |
| *medium* | Update dependencies           |
| ~~low~~  | Refactor old module           |
```

For color, use [inline styled spans](crate::_tutorial::_05_selectors_and_spans) inside cells:

```markdown
| Status | Message                        |
|--------|--------------------------------|
| OK     | ::success(All systems go)   |
| WARN   | ::warning(Memory high)      |
| FAIL   | ::danger(Disk failure)      |
```

## Previous / Next

Previous: [Frontmatter Styles](crate::_tutorial::_02_frontmatter_styles)

Next: [Widgets](crate::_tutorial::_04_widgets) — add sliders, checkboxes, and buttons to your markdown.
