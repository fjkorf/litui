# Layout and Spacing

> Run it: `cargo run -p tut_07_layout`

This tutorial adds **horizontal layout**, **columns**, and **spacing configuration**.

## What's new

Block directives `::: horizontal` and `::: columns N` control element placement. The `spacing:` frontmatter section customizes vertical gaps.

## Horizontal layout

Place elements side by side:

```text
::: horizontal

[button](Save) [button](Cancel) [button](Reset)

:::
```

Everything inside flows left-to-right in a single row via `ui.horizontal()`.

## Column layout

Split into equal-width columns with `::: next` separators:

```text
::: columns 2

### Controls

[slider](volume){vol}

[checkbox](muted)

::: next

### Status

**Name:** [display](name)

**Volume:** [display](volume)

:::
```

Each column gets an independent `Ui` via `ui.columns()`.

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

Block directives (`::: horizontal`, `::: columns`, `::: frame`, `::: foreach`, `::: if`, `::: style`) all use the same stack-based pattern. When a directive opens, the current `code_body` is saved and a fresh one starts accumulating. When `:::` closes, the accumulated body is popped and wrapped in the directive's container code (`ui.horizontal(|ui| { body })`, `ui.columns(N, |cols| { ... })`, etc.). This `code_body` swap pattern makes nesting directives straightforward — each level pushes its own frame onto the stack.

## What we built

Side-by-side buttons, two-column layouts, and custom spacing — all from markdown directives.
