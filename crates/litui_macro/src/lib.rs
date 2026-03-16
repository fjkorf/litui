//! litui — literate UI for egui.
//!
//! This proc-macro crate reads `.md` files at compile time, parses them with
//! [`pulldown_cmark`] 0.9, and emits Rust code that calls
//! [`litui_helpers`] functions to render the content in egui.
//!
//! # Entry Points
//!
//! - [`include_litui_ui!`] -- single-file inclusion. Returns a closure
//!   `|ui: &mut egui::Ui| { ... }` for static markdown, or a
//!   `(fn(&mut Ui, &mut LituiFormState), LituiFormState)` tuple when stateful
//!   widgets (slider, checkbox, etc.) are present.
//!
//! - [`define_litui_app!`] -- multi-page app skeleton. Generates a `Page`
//!   enum, per-page render functions, an `AppState` struct (if any page has
//!   widgets), and an `LituiApp` struct with navigation and dispatch.
//!
//! # Module Structure
//!
//! Parsing is handled by the [`litui_parser`] crate which produces a pure-data
//! AST (no `TokenStream` dependencies). Code generation lives in this crate:
//!
//! - `parse` -- bridge types (`ParsedMarkdown`, `WidgetField`, `WidgetType`)
//!   that connect parser output to codegen.
//! - `codegen_ast` -- converts `litui_parser::ast::Document` into
//!   `ParsedMarkdown` by walking the AST and emitting `TokenStream` code.
//! - `codegen` -- converts `ParsedMarkdown` into final `TokenStream` output
//!   for both macro entry points.
//!
//! # Usage
//!
//! ```rust,ignore
//! use litui_helpers::*;
//! use litui_macro::include_litui_ui;
//!
//! // Static markdown (no widgets):
//! let render = include_litui_ui!("content.md");
//! render(ui);
//!
//! // Markdown with stateful widgets:
//! let (render, mut state) = include_litui_ui!("form.md");
//! render(ui, &mut state);
//! ```
//!
//! ```rust,ignore
//! use litui_helpers::*;
//! use litui_macro::define_litui_app;
//!
//! define_litui_app! {
//!     parent: "content/_app.md",
//!     "content/about.md",
//!     "content/form.md",
//! }
//! // Generates: Page enum, AppState, render_about(), render_form(), LituiApp
//! ```

mod codegen;
mod codegen_ast;
mod parse;

use syn::{Error, LitStr};

use crate::codegen::{define_litui_app_impl, parsed_to_include_tokens};
use crate::parse::ParsedMarkdown;

// Re-export litui_parser types used by the macro crate
use litui_parser::frontmatter::{Frontmatter, merge_frontmatter, strip_frontmatter};

// ── Shared helpers ─────────────────────────────────────────────────

/// Read a markdown file and parse it into structured data.
/// Returns (frontmatter, `parsed_markdown`) or a compile error.
///
/// Uses the new `litui_parser` for markdown → AST, then `codegen_ast` for AST → `TokenStream`.
pub(crate) fn load_and_parse_md(
    path: &str,
    parent: Option<&Frontmatter>,
    source_span: proc_macro2::Span,
) -> Result<(Frontmatter, ParsedMarkdown), proc_macro2::TokenStream> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let abs_path = std::path::Path::new(&manifest_dir).join(path);
    let content = std::fs::read_to_string(&abs_path).map_err(|e| {
        Error::new(source_span, format!("Failed to read {path}: {e}")).to_compile_error()
    })?;

    let (yaml_str, markdown) = strip_frontmatter(&content);
    let child_frontmatter: Frontmatter = if yaml_str.is_empty() {
        Frontmatter::default()
    } else {
        serde_yaml::from_str(yaml_str).map_err(|e| {
            Error::new(
                source_span,
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

    // New path: litui_parser → AST → codegen_ast → ParsedMarkdown
    let doc = litui_parser::parse::parse_document(markdown, &frontmatter)
        .map_err(|e| Error::new(source_span, e.message).to_compile_error())?;
    let parsed =
        codegen_ast::document_to_parsed(&doc, &frontmatter, source_span)?.into_parsed_markdown();
    Ok((frontmatter, parsed))
}

// ── Proc-macro entry points ───────────────────────────────────────

/// Macro to include markdown as egui UI code.
#[proc_macro]
pub fn include_litui_ui(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let lit_str = match syn::parse2::<LitStr>(input.into()) {
        Ok(lit) => lit,
        Err(e) => return e.to_compile_error().into(),
    };

    match load_and_parse_md(&lit_str.value(), None, lit_str.span()) {
        Ok((frontmatter, parsed)) => {
            // Validate unused widget configs
            let unused: Vec<&String> = frontmatter
                .widgets
                .keys()
                .filter(|k| !parsed.used_widget_configs.contains(k.as_str()))
                .collect();
            if !unused.is_empty() {
                let mut names: Vec<&str> = unused.iter().map(|s| s.as_str()).collect();
                names.sort();
                return Error::new(
                    lit_str.span(),
                    format!(
                        "Unused widget config(s) in frontmatter `widgets:` section: {}. \
                         These are defined but never referenced by any widget via {{key}}.",
                        names.join(", ")
                    ),
                )
                .to_compile_error()
                .into();
            }
            parsed_to_include_tokens(parsed).into()
        }
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
/// - `LituiApp` struct with navigation and dispatch
#[proc_macro]
pub fn define_litui_app(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match define_litui_app_impl(input.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into(),
    }
}
