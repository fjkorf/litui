# Common Pitfalls

1. **Forgetting `Options::ENABLE_STRIKETHROUGH`** — pulldown-cmark defaults to no strikethrough support. The macro uses `Parser::new_ext(content, options)` with this enabled.

2. **Assuming Paragraph wrappers in lists** — Tight lists have NO Paragraph events. Always handle the `End(Item)` fallback.

3. **Style flags as if/else instead of composable** — Bold+italic must both apply simultaneously. Use `styled_label(ui, text, bold, italic, strikethrough)` which chains `.strong()`, `.italics()`, `.strikethrough()` on `RichText`.

4. **Nested layout closures** — Putting `horizontal_wrapped` inside another `horizontal_wrapped` causes items to flow inline within the parent. Each list item / paragraph must be its own top-level `horizontal_wrapped` call.

5. **`allocate_ui_with_layout` / `with_layout` size issues** — These pre-allocate the full available space. In an unbounded scroll area, this creates a massive empty canvas. Prefer `horizontal_wrapped` per-element instead of one global wrapping layout.

6. **pulldown-cmark 0.9 vs 0.10+ API** — `Event::End(Tag::X)` in 0.9 becomes `Event::End(TagEnd::X)` in 0.10. The `End` variant carries the full `Tag` in 0.9 (including URL for links). Do not upgrade without rewriting.

7. **`CARGO_MANIFEST_DIR` resolution** — The macro resolves file paths relative to the crate calling the macro, not the macro crate itself. Test fixtures must be relative to the test crate's manifest directory.

8. **egui is from crates.io** — Migration from fjkorf/egui fork to upstream egui 0.33 is complete. All `[patch.crates-io]` overrides have been removed. `eframe::App` uses `fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame)`. Use `CentralPanel::default().show(ctx, |ui| { ... })`.

9. **Frontmatter must be stripped before pulldown-cmark** — The `---` delimiters would be parsed as `ThematicBreak` (horizontal rules) by pulldown-cmark. Always call `strip_frontmatter()` first.

10. **`::key` is plain text to pulldown-cmark** — The parser has no concept of style suffixes. Detection happens in our code at flush boundaries, not in the event stream. If you're debugging style keys, check `detect_style_suffix()` logic, not pulldown-cmark events.

11. **Undefined style keys/classes panic at compile time** — This is intentional. A typo in `::prommo` or `.prommo` should fail the build, not silently render unstyled text. The panic message includes the key/class name.

12. **Widget link syntax requires single-word or angle-bracket content** — `[button](Click me)` breaks because spaces in URLs break pulldown-cmark link parsing. Use `[button](<Click me>)` or `[button](Click_me)`.

13. **Stateful widgets change the macro return type** — If ANY stateful widget is present, `include_markdown_ui!` returns `(fn, MdFormState)` instead of a closure. Destructure as `let (render, mut state) = ...`.

14. **Spinner widget causes kittest max_steps exceeded** — The spinner continuously requests repaint. Tests using spinner fixtures must use `Harness::builder().with_max_steps(16)`.

15. **Shared widget fields must have matching types** — In `define_markdown_app!`, all widget fields merge into a single `AppState`. Two pages can declare the same field name if the types match (e.g., both `[slider](volume)` — shared `f64`). Conflicting types (e.g., `[slider](foo)` on one page, `[checkbox](foo)` on another) produce a compile error.

16. **Display widgets self-declare fields** — `[display](field)` reads from `AppState`. If no input widget declares the field, display self-declares it as `String`. This enables display-only pages (stat cards, dashboards). If an input widget on another page already declares the field, the input widget's type wins.

17. **`{config}` is widget config only** — `{config}` after a widget link resolves against `widgets:` in frontmatter (min/max/label/format), NOT `styles:`. For style application, use `::key` on paragraphs/headings or `.class` selectors on link text: `[button.accent](Submit)` not `[button](Submit)::accent`.

18. **Inline styled spans use `::class(text)` syntax** — `::accent(orange bold text)` applies the `accent` style to the parenthesized text. Spaces work fine inside the parentheses. The old `[.class](<text>)` link-based syntax is deprecated.
