# pulldown-cmark 0.9 Event Model

**This is the single most important thing to understand.** The macro's entire logic is driven by pulldown-cmark's event stream. Getting events wrong causes rendering bugs.

## Version-Specific API

- Pinned to **0.9.x**. Later versions (0.10+) changed `Event::End(Tag)` to `Event::End(TagEnd)` — a breaking change. Do NOT upgrade without rewriting the event loop.
- Strikethrough requires `Options::ENABLE_STRIKETHROUGH` passed to `Parser::new_ext()`. Without it, `~~text~~` is emitted as literal tildes.

## Event Stream Structure

Events are strictly hierarchical Start/End pairs. Text only appears inside containers.

```
Start(List(None))           <- None = unordered; Some(1) = ordered starting at 1
  Start(Item)
    Start(Paragraph)        <- only in "loose" lists (blank lines between items)
      Text("item text")
    End(Paragraph)
  End(Item)
End(List(None))
```

## Tight vs Loose Lists — CRITICAL GOTCHA

pulldown-cmark distinguishes **tight** lists (no blank lines between items) from **loose** lists:

- **Tight list** (no blank lines): Text appears directly inside `Item` — NO `Paragraph` wrapper.
  ```
  Start(Item) -> Text("bullet text") -> End(Item)
  ```
- **Loose list** (blank lines between items): Text is wrapped in `Paragraph`.
  ```
  Start(Item) -> Start(Paragraph) -> Text("bullet text") -> End(Paragraph) -> End(Item)
  ```

**Why this matters**: If you only flush content at `End(Paragraph)`, tight list items never flush. You MUST also flush at `End(Item)` as a fallback, and at `Start(List)` when a sub-list begins inside a parent item (otherwise the parent's text accumulates into the child's `pending_text`).

## Nested Lists are Inside Parent Items

A sub-list appears BEFORE `End(Item)` of its parent, not after:

```
Start(Item)
  Text("parent item")       <- or Start(Paragraph)...End(Paragraph) if loose
  Start(List(None))          <- nested list is INSIDE the parent item
    Start(Item)
      Text("child item")
    End(Item)
  End(List(None))
End(Item)                    <- parent closes AFTER nested list
```

## Task Lists (GFM)

Requires `Options::ENABLE_TASKLISTS`. Task list items emit a `TaskListMarker(bool)` event immediately after `Start(Item)` (tight) or after `Start(Paragraph)` (loose):

```
Start(List(None))
  Start(Item)
    TaskListMarker(true)        <- checked: - [x]
    Text("done task")
  End(Item)
  Start(Item)
    TaskListMarker(false)       <- unchecked: - [ ]
    Text("pending task")
  End(Item)
End(List(None))
```

The marker appears before any text. The macro captures it in a `task_list_checked: Option<bool>` state variable, which is passed to `emit_list_item()` and consumed (via `.take()`) when the item is flushed. This replaces the bullet/number prefix with a checkbox drawn by `emit_task_checkbox()`.

## Blockquote Nesting

Each `>` level produces nested `Start(BlockQuote)` events, not sequential ones:

```
Start(BlockQuote)            <- > level 1
  Start(Paragraph)
    Text("level 1")
  End(Paragraph)
  Start(BlockQuote)          <- > > level 2 (nested inside level 1)
    Start(Paragraph)
      Text("level 2")
    End(Paragraph)
  End(BlockQuote)
End(BlockQuote)
```

## Inline Formatting

`Emphasis`, `Strong`, `Strikethrough` are inline tags that wrap their text:

```
Start(Paragraph)
  Text("before ")
  Start(Strong)
    Start(Emphasis)
      Text("bold italic")   <- both flags active simultaneously
    End(Emphasis)
  End(Strong)
  Text(" after")
End(Paragraph)
```

Style flags must compose — a single `if/else` chain that checks one flag at a time will fail for nested styles.

## Links

Links carry URL in the `Start` event. Text is accumulated between `Start(Link)` and `End(Link)`:

```
Start(Link(LinkType, "https://url", "title"))
  Text("link text")
End(Link(...))
```

The `End(Link)` event also carries the URL in pulldown-cmark 0.9.
