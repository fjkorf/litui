# Upgrade Guide: :: Style Syntax

## What Changed

Style application syntax changed from `{key}` to `::key`. Widget config syntax `{config}` is unchanged.

## Before / After

### Line styles (paragraphs, headings, blockquotes, lists)

```markdown
BEFORE: # Title {title}
AFTER:  # Title ::title

BEFORE: Some warning text. {danger}
AFTER:  Some warning text. ::danger

BEFORE: > Important note {warning}
AFTER:  > Important note ::warning

BEFORE: - All systems go {success}
AFTER:  - All systems go ::success
```

### Widget config — NO CHANGE

```markdown
[slider](volume){vol}           ← unchanged
[button.primary](Submit){cfg}   ← unchanged
[display](hp_text){fmt}         ← unchanged
```

`{config}` after a widget link still means widget config lookup. Only the style application syntax changed.

### Runtime styling

```markdown
BEFORE: [display](hp_text){$hp_style}
AFTER:  [display](hp_text) ::$hp_style
```

## Why

The old `{key}` syntax had dual meaning — widget config after widgets, style application at paragraph end. The new `::key` syntax eliminates this ambiguity:

- `{config}` = widget config (exclusively)
- `::key` = compile-time style application
- `::$field` = runtime style reference

### Inline styled spans

```markdown
BEFORE: [.accent](<orange bold text>)
AFTER:  ::accent(orange bold text)
```

No angle brackets or special escaping needed — parentheses contain the styled text, spaces work fine.

### Block directives

```markdown
BEFORE: [foreach](items) ... [/foreach]()
AFTER:  ::: foreach items ... :::

BEFORE: [if](has_orb) ... [/if]()
AFTER:  ::: if has_orb ... :::

NEW:    ::: style hp_style ... :::
```

Bare `:::` closes the innermost block. Named close `::: /foreach` also works for clarity in nested blocks.

## Migration

Find and replace in your `.md` files:

1. Trailing ` {stylename}` → ` ::stylename` (on paragraphs, headings, list items, blockquotes)
2. `[.class](<text>)` → `::class(text)` (inline styled spans)
3. `{$field}` after display → `::$field`
4. `[foreach](X)` → `::: foreach X`, `[/foreach]()` → `:::`
5. `[if](X)` → `::: if X`, `[/if]()` → `:::`

Widget configs like `[slider](vol){cfg}` need NO changes.
