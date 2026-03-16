//! litui — literate UI for egui.
//!
//! This proc-macro crate reads `.md` files at compile time, parses them with
//! [`pulldown_cmark`] 0.9, and emits Rust code that calls
//! [`markdown_to_egui_helpers`] functions to render the content in egui.
//!
//! # Entry Points
//!
//! - [`include_markdown_ui!`] -- single-file inclusion. Returns a closure
//!   `|ui: &mut egui::Ui| { ... }` for static markdown, or a
//!   `(fn(&mut Ui, &mut MdFormState), MdFormState)` tuple when stateful
//!   widgets (slider, checkbox, etc.) are present.
//!
//! - [`define_markdown_app!`] -- multi-page app skeleton. Generates a `Page`
//!   enum, per-page render functions, an `AppState` struct (if any page has
//!   widgets), and an `MdApp` struct with navigation and dispatch.
//!
//! # Module Structure
//!
//! - `frontmatter` -- YAML frontmatter types (`Frontmatter`, `StyleDef`,
//!   `WidgetDef`, `PageDef`), parsing, parent/child merging, CSS-like selector
//!   parsing, hex color parsing, and `{key}` detection.
//! - `parse` -- pulldown-cmark event loop that converts markdown into
//!   `ParsedMarkdown` (accumulated `TokenStream` fragments + widget fields).
//! - `codegen` -- converts `ParsedMarkdown` into final `TokenStream` output
//!   for both macro entry points.
//!
//! # Usage
//!
//! ```rust,ignore
//! use markdown_to_egui_helpers::*;
//! use markdown_to_egui_macro::include_markdown_ui;
//!
//! // Static markdown (no widgets):
//! let render = include_markdown_ui!("content.md");
//! render(ui);
//!
//! // Markdown with stateful widgets:
//! let (render, mut state) = include_markdown_ui!("form.md");
//! render(ui, &mut state);
//! ```
//!
//! ```rust,ignore
//! use markdown_to_egui_helpers::*;
//! use markdown_to_egui_macro::define_markdown_app;
//!
//! define_markdown_app! {
//!     parent: "content/_app.md",
//!     "content/about.md",
//!     "content/form.md",
//! }
//! // Generates: Page enum, AppState, render_about(), render_form(), MdApp
//! ```

mod codegen;
mod frontmatter;
mod parse;

use syn::{Error, LitStr};

use crate::codegen::{define_markdown_app_impl, parsed_to_include_tokens};
use crate::frontmatter::{Frontmatter, merge_frontmatter, strip_frontmatter};
use crate::parse::{ParsedMarkdown, markdown_to_egui};

// ── Shared helpers ─────────────────────────────────────────────────

/// Read a markdown file and parse it into structured data.
/// Returns (frontmatter, parsed_markdown) or a compile error.
pub(crate) fn load_and_parse_md(
    path: &str,
    parent: Option<&Frontmatter>,
) -> Result<(Frontmatter, ParsedMarkdown), proc_macro2::TokenStream> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let abs_path = std::path::Path::new(&manifest_dir).join(path);
    let content = std::fs::read_to_string(&abs_path).map_err(|e| {
        Error::new(
            proc_macro2::Span::call_site(),
            format!("Failed to read {path}: {e}"),
        )
        .to_compile_error()
    })?;

    let (yaml_str, markdown) = strip_frontmatter(&content);
    let child_frontmatter: Frontmatter = if yaml_str.is_empty() {
        Frontmatter::default()
    } else {
        serde_yaml::from_str(yaml_str).map_err(|e| {
            Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to parse frontmatter YAML in {path}: {e}"),
            )
            .to_compile_error()
        })?
    };

    let frontmatter = if let Some(parent_fm) = parent {
        merge_frontmatter(parent_fm, child_frontmatter)
    } else {
        child_frontmatter
    };

    let parsed = markdown_to_egui(markdown, &frontmatter);
    Ok((frontmatter, parsed))
}

// ── Proc-macro entry points ───────────────────────────────────────

/// Macro to include markdown as egui UI code.
#[proc_macro]
pub fn include_markdown_ui(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let lit_str = match syn::parse2::<LitStr>(input.into()) {
        Ok(lit) => lit,
        Err(e) => return e.to_compile_error().into(),
    };

    match load_and_parse_md(&lit_str.value(), None) {
        Ok((_frontmatter, parsed)) => parsed_to_include_tokens(parsed).into(),
        Err(err) => err.into(),
    }
}

/// Item-position macro that generates a full app skeleton from markdown files.
///
/// Each `.md` file must include a `page:` section in its YAML frontmatter:
/// ```yaml
/// ---
/// page:
///   name: About
///   label: About
///   default: true
/// ---
/// ```
///
/// Generates:
/// - `Page` enum with one variant per file
/// - State structs for pages with stateful widgets (named `{PageName}State`)
/// - Render functions (`render_{snake_name}`)
/// - `MdApp` struct with navigation and dispatch
#[proc_macro]
pub fn define_markdown_app(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match define_markdown_app_impl(input.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into(),
    }
}
