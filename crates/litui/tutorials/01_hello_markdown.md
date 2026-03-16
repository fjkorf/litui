# Hello Markdown

> Run it: `cargo run -p tut_01_hello`

litui turns markdown into native egui widgets at compile time. No runtime parsing, no overhead — just write `.md` and get a UI.

## The markdown

```text
# Hello litui

This is a **bold** statement with *italic* and ~~strikethrough~~ text.

## Lists

- First item
- Second item with **bold**
- Nested list:
  - Sub-item one
  - Sub-item two

1. Ordered first
2. Ordered second

## Blockquotes

> This is a blockquote.
> It can span multiple lines.

## Code

Inline `code` works too.

## Links

Visit [egui](https://github.com/emilk/egui) for more.
```

## The Rust

```rust,ignore
use eframe::egui;
use litui::*;

fn main() -> eframe::Result {
    eframe::run_simple_native("01 Hello", Default::default(), |ctx, _| {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let render = include_markdown_ui!("content.md");
                render(ui);
            });
        });
    })
}
```

`include_markdown_ui!` reads the `.md` file at compile time and returns a closure that renders egui widgets. Call it with `render(ui)` — that's it.

## Supported elements

| Markdown | Renders as |
|----------|-----------|
| `# Heading` | `h1()` / `h2()` / `h3()` sized labels |
| `**bold**` | Bold `RichText` |
| `*italic*` | Italic `RichText` |
| `` `code` `` | Monospace with background |
| `- item` | Indented bullet list |
| `1. item` | Numbered list |
| `> quote` | Vertical bar + indented text |
| `[text](url)` | Clickable hyperlink |
| `---` | Horizontal separator |
| Fenced code blocks | Monospace group with background |

## Expert tip

The macro runs during `cargo build`, not at runtime. It parses the markdown with pulldown-cmark, walks the event stream, and emits Rust function calls (`h1(ui, "Hello")`, `styled_label(ui, "bold text", true, false, false)`, etc.). The result is a closure with zero allocation and zero parsing at runtime — just direct egui API calls baked into your binary.

## What we built

A static markdown page rendered as native egui widgets with one line of Rust.
