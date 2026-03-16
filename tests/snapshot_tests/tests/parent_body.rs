//! Test that parent frontmatter with markdown body generates render_shared().

#![expect(clippy::unwrap_used)]

use snapshot_tests::snapshot_markdown;

mod app {
    use litui_helpers::*;
    use litui_macro::define_litui_app;

    define_litui_app! {
        parent: "fixtures/parent_with_body.md",
        "fixtures/child_page.md",
    }
}

#[test]
fn parent_shared_content() {
    snapshot_markdown("parent_shared_content", |ui| app::render_shared(ui));
}

#[test]
fn child_inherits_style() {
    snapshot_markdown("child_inherits_style", |ui| {
        app::render_child_page(ui);
    });
}
