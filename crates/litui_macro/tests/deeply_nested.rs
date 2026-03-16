use litui_helpers::*;
extern crate egui;
// Tests for deeply nested markdown rendering using the macro.

#[test]
fn deeply_nested_markdown_renders_correctly() {
    // This is a placeholder for a real test. In a real test, you would invoke the macro on a deeply nested markdown string
    // and check that the generated code compiles and produces the expected UI tree.
    // For now, we just check that the macro expands without error.
    let _ui = litui_macro::include_litui_ui!("tests/deeply_nested.md");
}
