# litui_macro

A procedural macro to convert Markdown files into egui UI code at compile time.

## Features
- Parses Markdown files and generates egui UI code
- Supports headings, lists, blockquotes, tables, code blocks, paragraphs, and text
- Error handling for missing or invalid files
- Expandable for advanced features (styling, hot reloading, custom directives)

## Usage
Add to your Cargo.toml:

```toml
[dependencies]
litui_macro = { path = "../litui_macro" }
```

Invoke macro in your code:

```rust
use litui_macro::include_litui_ui;
let generated = include_litui_ui!("test.md");
// Use: generated(&mut ui);
```

## Example
See `examples/01_hello/src/main.rs` for a minimal egui app using the macro.
See `tests/basic_macro.rs` for a test example.

## Roadmap
- Performance optimizations
- IDE integration
- Advanced features (hot reloading, frontmatter, custom directives)

## License
MIT OR Apache-2.0
