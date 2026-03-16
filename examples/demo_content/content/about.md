---
page:
  name: About
  label: About
  default: true
---

# litui ::title

A compile-time Markdown-to-egui UI generator. ::subtitle

![Ferris the crab](https://rustacean.net/assets/rustacean-flat-noshadow.png)

---

## What Is This?

This demo app is **entirely driven by markdown files**. Every page you see — headings, paragraphs, lists, tables, styled text, and interactive widgets — is generated at compile time from `.md` files by the `include_markdown_ui!` proc-macro.

::: frame frame

Zero runtime parsing. Zero overhead. Fully type-checked Rust.

:::

## How It Works

1. Write your UI content in `.md` files with optional YAML frontmatter
2. The proc-macro reads and parses the markdown at compile time
3. It emits a Rust closure that calls egui helper functions
4. The result is native egui widgets with no runtime Markdown parsing

## Features ::highlight

- **Headings** (H1-H3) with custom sizes
- **Inline styles** — bold, *italic*, ~~strikethrough~~, composable
- **Nested lists** — bullet and ordered, any depth
- **GFM tables** — with bold headers and striped rows
- **Blockquotes** — nested with depth-based vertical bars
- **Code blocks** and `inline code`
- **Links** — clickable [hyperlinks](https://github.com/emilk/egui)
- **Frontmatter styles** — `{key}` references resolved at compile time
- **Images** — standard `![alt](url)` syntax
- **Styled containers** — `{key}` colors blockquote bars and list bullets
- **Widget directives** — sliders, checkboxes, text inputs, buttons, radio, combobox, color picker

## Links

- [egui](https://github.com/emilk/egui) — the immediate-mode GUI library
- [pulldown-cmark](https://github.com/pulldown-cmark/pulldown-cmark) — the Markdown parser
