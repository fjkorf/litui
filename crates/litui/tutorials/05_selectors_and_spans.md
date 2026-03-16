# Selectors and Styled Spans

> Run it: `cargo run -p selectors_example`

Selectors give you CSS-like control over widget styling and element identity, all resolved at compile time. No runtime lookup, no style sheets — just frontmatter keys and link syntax.

## Class selectors

Append `.class` to any widget's link text to apply a frontmatter style:

```markdown
---
styles:
  primary:
    color: "#FFFFFF"
    background: "#2266DD"
    bold: true
  large:
    size: 20.0
  danger:
    color: "#FF4444"
    bold: true
---

[button.primary](Submit)
[button.primary.large](Submit)
[button.danger](Delete)
```

The class name references a key in `styles:`. Multiple classes compose left-to-right via `merge_style_defs()` — if two classes set the same property, the last one wins.

`[button.primary.large](Submit)` gets the `primary` style first, then `large` merged on top. Since `primary` doesn't set `size` and `large` does, you get bold white-on-blue text at 20pt.

## ID selectors

Append `#id` to set an `egui::Id` on the widget via `ui.push_id()`:

```markdown
[slider#volume](vol){vol_cfg}
[slider#brightness](bright){bright_cfg}
```

IDs are useful when you have multiple widgets of the same type — egui needs unique IDs to track internal state (which slider is being dragged, etc.). Without an explicit ID, the macro auto-generates one from position, but explicit IDs are more stable.

## Combining selectors

You can use both ID and classes on the same widget:

```markdown
[button#submit.primary.large](Save)
```

Parse order: `button` is the base name, `#submit` is the ID, `.primary.large` are classes. The base name determines the widget type, the ID sets egui identity, and classes apply styles.

## Inline styled text spans

Here's where selectors get interesting beyond widgets. An empty base name with classes creates a styled text fragment:

```markdown
---
styles:
  accent:
    color: "#FF6B00"
    bold: true
  subtle:
    color: "#888888"
    italic: true
---

This has ::accent(important orange text) inline with regular text.

And some ::subtle(gray italic aside) in the middle of a sentence.
```

The `::accent(text)` syntax means: no widget, no link — just apply the `accent` style to this text span. Parentheses contain the styled text. Spaces work fine.


![Selectors](img/selectors.png)

## Multi-word content

This applies to all link-based syntax (widgets, buttons, styled spans). Two options for spaces:

**Angle brackets** (recommended):

```markdown
[button](<Click me now>)
::accent(orange bold text)
```

**Underscores** (converted to spaces at compile time):

```markdown
[button](Click_me_now)
```

Both produce the same display text. Angle brackets are clearer when the content has mixed formatting or is long.

## `.class` vs `::key` vs `{key}` — the distinction

This trips people up. Three syntaxes, three purposes:

| Syntax | Where | Purpose |
|--------|-------|---------|
| `::key` | End of heading/paragraph/blockquote/list item | Applies frontmatter **style** (color, bold, size) |
| `.class` | On widgets: `[button.primary]` | Applies frontmatter **style** to a widget |
| `{key}` | After widgets: `[slider](vol){cfg}` | References frontmatter **widget config** (min, max, label) |

On plain text (headings, paragraphs), `::key` applies styles. On widgets, `{key}` applies config and `.class` applies styles. If you want to style a widget, use `.class`:

```markdown
# Good
[button.primary](Submit){on_click}    # .primary = style, {on_click} = config

# Wrong — {primary} on a widget looks up widget config, not styles. Use .class for widget styling
[button](Submit){primary}
```

## Styled buttons in practice

Buttons are the most common use case for class selectors:

```markdown
---
styles:
  primary:
    color: "#FFFFFF"
    background: "#2266DD"
    bold: true
  secondary:
    color: "#CCCCCC"
  destructive:
    color: "#FFFFFF"
    background: "#CC3333"
    bold: true
---

[button.primary](<Save Changes>){on_save}

[button.secondary](Cancel)

[button.destructive](Delete){on_delete}
```

Each button gets a distinct visual treatment. The `{on_save}` and `{on_delete}` configs generate click counters in state.

## Undefined classes fail the build

Just like undefined `::key` style references, an undefined `.class` panics at compile time. Typo `.primay` instead of `.primary`? Build error with a clear message. No silent fallback to unstyled text.

## Previous / Next

Previous: [Widgets](crate::_tutorial::_04_widgets)

Next: [Styled Containers](crate::_tutorial::_06_styled_containers) — apply styles to blockquotes and list items.
