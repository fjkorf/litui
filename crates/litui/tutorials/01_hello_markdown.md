# Hello Markdown

> Run it: `cargo run -p markdown_macro_example`

Your first litui UI in 60 seconds. Write markdown, compile it into egui widgets, done.

## Setup

Add litui and eframe to your `Cargo.toml`:

```toml
[dependencies]
litui = "0.33"
eframe = "0.33"
```

## Write some markdown

Create `content.md` next to your `src/` directory. litui supports the standard markdown features you'd expect:

- **Headings** — H1 (28pt), H2 (22pt), H3 (18pt)
- **Inline formatting** — `**bold**`, `*italic*`, `~~strikethrough~~`, combinations
- **Inline code** — backtick-delimited monospace with background
- **Bullet and numbered lists** — nested to any depth
- **Blockquotes** — nested with depth-based vertical bars
- **Fenced code blocks** — monospace with background
- **Links** — clickable egui hyperlinks
- **Horizontal rules** — `---` separators
- **Line breaks** — soft and hard

![Headings and text](img/hello_headings.png)

![Lists and blockquotes](img/hello_lists.png)

## Wire it up

Create `src/main.rs`:

```rust,ignore
use eframe::egui;
use litui::*;

fn main() -> eframe::Result {
    eframe::run_native(
        "My First litui App",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(MyApp))),
    )
}

struct MyApp;

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let render = include_markdown_ui!("content.md");
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                render(ui);
            });
        });
    }
}
```

That's it. `include_markdown_ui!` reads your markdown at compile time and returns a closure. Call it with `&mut Ui` and your content renders as native egui widgets.

No runtime parsing. No markdown library in your binary. Just compiled Rust.

## What just happened

The macro:

1. Reads `content.md` at compile time
2. Parses it with pulldown-cmark
3. Emits a Rust closure that calls egui helper functions (`h1()`, `styled_label()`, `emit_bullet_prefix()`, etc.)
4. The closure captures nothing — it's pure function calls

If your markdown has no interactive widgets (sliders, checkboxes, etc.), the macro returns a simple `impl FnMut(&mut egui::Ui)`. We'll add widgets in [Tutorial 04](crate::_tutorial::_04_widgets).

## Next up

[Frontmatter Styles](crate::_tutorial::_02_frontmatter_styles) — make your text colorful with YAML-defined style presets.
