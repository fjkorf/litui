---
page:
  name: Styles
  label: Styles
---

# Style Gallery

All styles below are defined in YAML frontmatter and applied with `::key` suffixes. The macro resolves them at compile time into literal egui `RichText` method chains.

---

## Color Styles

This is the primary style. ::primary

This is the success style. ::success

This is the danger style. ::danger

This is the warning style with background. ::warning

This is muted and understated. ::muted

## Size Styles

This text is large. ::large

This is a small footnote-style annotation. ::small_note

## Special Styles

This looks like a code annotation. ::code

This has a highlighted background. ::highlighted

This text is struck through and dimmed. ::strike

---

## Styled Containers

The `{key}` syntax also styles blockquote bars and list bullet/number markers.

> Success: all tests passing. ::success

> Danger: disk space critically low. ::danger

> Warning: API rate limit approaching. ::warning

- Feature shipped to production ::success
- Known issue under investigation ::warning
- Broken in latest release ::danger
- Scheduled for deprecation ::muted

1. Deploy to staging ::success
2. Run integration tests ::warning
3. Roll back if failures detected ::danger

---

## Combining with Inline Markdown

You can use **bold** and *italic* inside a styled paragraph — the frontmatter style composes with inline formatting. ::primary

Tables also work with styles defined above:

| Style | Key | Properties |
|-------|-----|-----------|
| **Primary** | `primary` | Blue, bold, 20pt |
| **Success** | `success` | Green, bold |
| **Danger** | `danger` | Red, bold |
| **Warning** | `warning` | Orange, bold, dark bg |
| **Muted** | `muted` | Gray, italic, weak |
