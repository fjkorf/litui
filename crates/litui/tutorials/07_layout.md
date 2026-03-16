# Layout and Spacing

> Run it: `cargo run -p tut_07_layout`

This tutorial covers **alignment**, **horizontal layout**, **columns**, **weighted columns**, and **spacing configuration**.

## What's new

Block directives control element placement and alignment. The `spacing:` frontmatter section customizes vertical gaps.

## Alignment directives

Center, right-align, or fill content blocks:

```text
::: center
## Game Over
Your score: 42
:::

::: right
*Last updated: today*
:::

::: fill
[button](wide_button)
:::
```

- `::: center` — horizontally centers all content in the block
- `::: right` — right-aligns all content
- `::: fill` — stretches widgets to fill available width

These nest inside any other directive, including frames and columns.

## Horizontal layout

Place elements side by side with optional alignment:

```text
::: horizontal
[button](Save) [button](Cancel) [button](Reset)
:::
```

Everything inside flows left-to-right in a single row. Add an alignment argument for different positioning:

```text
::: horizontal center
[button](OK)
:::

::: horizontal right
[button](Submit)
:::

::: horizontal space-between
[button](Back)
::: next
[button](Next)
:::
```

- `::: horizontal` — left-aligned (default)
- `::: horizontal center` — items centered in the row
- `::: horizontal right` — items right-aligned
- `::: horizontal space-between` — first group left, second group right; use `::: next` to separate

## Column layout

Split into equal-width columns with `::: next` separators:

```text
::: columns 2

### Controls
[slider](volume){vol}

::: next

### Status
**Volume:** [display](volume)
:::
```

Each column gets an independent `Ui` via `ui.columns()`.

## Weighted columns

Use colon-separated weights for proportional column widths:

```text
::: columns 3:1
Main content takes 75% width.
::: next
Sidebar takes 25%.
:::

::: columns 1:2:1
Left nav
::: next
Main content (50%)
::: next
Right panel
:::
```

Weights are ratios: `3:1` means 75%/25%, `1:2:1` means 25%/50%/25%. When all weights are equal, litui uses `ui.columns()`. When weights differ, it uses `egui_extras::StripBuilder` with proportional sizes.

## Table column alignment

Standard GFM table alignment is honored:

```text
| Left   | Center | Right  |
|:-------|:------:|-------:|
| text   |  text  |   text |
```

- `:---` or `---` = left-aligned (default)
- `:---:` = center-aligned
- `---:` = right-aligned

## Spacing configuration

Override default vertical gaps in frontmatter:

```yaml
spacing:
  paragraph: 6.0
  heading_h2: 14.0
```

| Field | Default | Effect |
|-------|---------|--------|
| `paragraph` | 8.0 | Gap after paragraphs |
| `table` | 8.0 | Gap after tables |
| `heading_h1` | 16.0 | Space before H1 |
| `heading_h2` | 12.0 | Space before H2 |
| `heading_h3` | 8.0 | Space before H3 |
| `heading_h4` | 4.0 | Space before H4+ |
| `item` | — | `ui.spacing_mut().item_spacing.y` |

## Expert tip

Block directives (`::: center`, `::: right`, `::: fill`, `::: horizontal`, `::: columns`, `::: frame`, `::: foreach`, `::: if`, `::: style`) all use the same stack-based pattern. When a directive opens, the current `code_body` is saved and a fresh one starts accumulating. When `:::` closes, the accumulated body is popped and wrapped in the directive's container code. This `code_body` swap pattern makes nesting directives straightforward — each level pushes its own frame onto the stack.

## What we built

Centered headings, right-aligned text, proportional column layouts, aligned horizontal button rows, and custom spacing — all from markdown directives.
