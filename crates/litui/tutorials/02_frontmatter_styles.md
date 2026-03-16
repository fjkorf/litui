# Frontmatter Styles

> Run it: `cargo run -p widgets_example`

Style your text with YAML frontmatter. Define named presets once, apply them anywhere with `::key`.

## The frontmatter block

A `---` fenced YAML block at the top of your markdown file. The macro strips it before parsing — pulldown-cmark never sees it.

```markdown
---
styles:
  accent:
    color: "#FF6B00"
    bold: true
  subtle:
    color: "#888888"
    italic: true
    size: 14.0
---

# Welcome ::accent

This paragraph gets the accent style. ::accent

A quieter note here. ::subtle
```

The `::key` at the end of a heading or paragraph tells the macro to look up that style and apply it. Everything resolves at compile time — no runtime style lookup, no CSS, just baked-in `RichText` method chains.

![Styled text](img/styles_basic.png)

## Style properties

Every property is optional. Only set what you need.

```yaml
styles:
  my_style:
    bold: true           # RichText::strong()
    italic: true         # RichText::italics()
    strikethrough: true  # RichText::strikethrough()
    underline: true      # RichText::underline()
    color: "#FF6B00"     # text color, hex RGB
    background: "#2A2A2A" # background highlight color
    size: 18.0           # font size in points (float)
    monospace: true      # monospace font
    weak: true           # dimmed/secondary text
```

Colors are `#RRGGBB` hex strings. The macro parses them at compile time into `Color32::from_rgb(r, g, b)`.

## Applying styles

Put `::key` at the end of the line. It works on:

**Headings:**

```markdown
# Error Report ::danger
## Status ::success
```

Heading styles merge with heading defaults. A `::accent` on an `# H1` keeps the 28pt size unless your style explicitly sets `size`.

**Paragraphs:**

```markdown
This entire paragraph gets the danger style applied. Every word. ::danger
```

The `::key` must be the last thing on the paragraph. The macro strips it from the displayed text.

## Composing with inline markdown

Frontmatter styles and inline markdown formatting stack. Bold, italic, code, and strikethrough all work inside a styled paragraph:

```markdown
---
styles:
  note:
    color: "#4488CC"
    size: 15.0
---

This has **bold** and *italic* words inside a blue paragraph. ::note
```

The `::note` style sets the base color and size. Inline `**bold**` adds `.strong()` on top. They compose — they don't fight.

## Multiple styles

Define as many as you need:

```yaml
styles:
  danger:
    color: "#FF4444"
    bold: true
  success:
    color: "#44BB44"
  warning:
    color: "#FFAA00"
    bold: true
  code_note:
    monospace: true
    color: "#AAAAAA"
    size: 13.0
```

```markdown
# System Status ::success

All services operational. ::success

Memory usage above 90%. ::warning

Disk failure on node-3. ::danger

`Check /var/log/syslog for details.` ::code_note
```

## Styled blockquotes and list items

The `::key` syntax also works on blockquotes and list items. The style's color tints the quote bar or bullet/number:

```markdown
> All systems operational. ::success

> Disk failure detected. ::danger

- Build passed ::success
- Tests skipped ::warning

1. Completed ::success
2. Blocked ::danger
```

## Undefined keys fail the build

Typo in a style key? The macro panics at compile time with a clear error. `::prommo` when you meant `::promo` won't silently render unstyled — it stops the build. This is intentional.

## Parent inheritance

When using `define_markdown_app!`, you can define shared styles in a parent file:

```rust,ignore
define_markdown_app! {
    parent: "content/_app.md",
    "content/page_one.md",
    "content/page_two.md",
}
```

The parent's `styles:` are inherited by all child pages. Child styles override parent styles on key collision. This keeps your theme in one place.

More on this in [Multi-Page Apps](crate::_tutorial::_07_multi_page_apps).

## What's happening under the hood

The macro:

1. Strips the `---` YAML block with `strip_frontmatter()`
2. Deserializes into `Frontmatter { styles, widgets, page }`
3. When it hits `::key` at a flush point, calls `detect_style_suffix()`
4. Looks up the key in the styles map
5. Emits a `styled_label_rich()` call with all properties as compile-time literals

A style `{ color: "#FF6B00", bold: true }` becomes `RichText::new(text).color(Color32::from_rgb(255, 107, 0)).strong()` in the generated code. Zero runtime cost.

## Dynamic styling at runtime

All styles shown above are compile-time — the color is baked into the generated code. For styles that change based on game state (HP color, threat level), use dynamic styling.

### ::: style block

Wrap content in a `::: style` fence. The style name is read from AppState at runtime:

```text
::: style hp_style

**HP:** [display](hp_text)
**MP:** [display](mp_text)

:::
```

State: `hp_style: String` — set to a frontmatter style name each frame:
```rust,ignore
state.hp_style = if hp_pct > 0.5 { "hp_good" } else { "hp_danger" }.into();
```

The macro generates a compile-time lookup table from your frontmatter styles. At runtime, it resolves the name and applies `ui.visuals_mut().override_text_color`. Content outside the block is unaffected.

Style blocks nest — inner blocks override outer.

### How it works

1. You define styles in YAML as usual (`hp_good`, `hp_danger`, etc.)
2. The macro generates a `__resolve_style_color()` match table at compile time
3. At runtime, `state.hp_style` is matched against the table
4. The resolved color is applied via `ui.visuals_mut().override_text_color`
5. Only `color` is dynamically applied — bold, size, etc. remain compile-time

This is egui's native pattern for scoped style changes.

## Next up

[Tables](crate::_tutorial::_03_tables) — render data grids with GFM table syntax.
