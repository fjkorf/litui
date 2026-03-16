//! Bridge types for code generation.
//!
//! These types bridge the `litui_parser` AST output to the codegen module.
//! `WidgetField` and `WidgetType` wrap the `litui_parser` equivalents with
//! `TokenStream` generation methods. `ParsedMarkdown` holds the generated
//! token output that `codegen.rs` consumes.

use quote::quote;

// ── Widget type system ───────────────────────────────────────────

/// The Rust type of a widget's state field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WidgetType {
    F64,
    Bool,
    U32,
    Usize,
    String,
    ByteArray4,
    VecString,
    Date,
}

impl WidgetType {
    pub fn to_tokens(self) -> proc_macro2::TokenStream {
        match self {
            Self::F64 => quote! { f64 },
            Self::Bool => quote! { bool },
            Self::U32 => quote! { u32 },
            Self::Usize => quote! { usize },
            Self::String => quote! { String },
            Self::ByteArray4 => quote! { [u8; 4] },
            Self::VecString => quote! { Vec<String> },
            Self::Date => quote! { chrono::NaiveDate },
        }
    }

    pub fn default_tokens(self) -> proc_macro2::TokenStream {
        match self {
            Self::F64 => quote! { 0.0 },
            Self::Bool => quote! { false },
            Self::U32 => quote! { 0 },
            Self::Usize => quote! { 0 },
            Self::String => quote! { String::new() },
            Self::ByteArray4 => quote! { [255, 255, 255, 255] },
            Self::VecString => quote! { Vec::new() },
            Self::Date => quote! { chrono::NaiveDate::default() },
        }
    }
}

/// A field inside a foreach row struct (bridge type for codegen).
#[derive(Clone)]
pub(crate) enum RowField {
    /// `{field}` text reference — always String, display-only.
    Display(String),
    /// Widget inside foreach — typed, interactive.
    Widget { name: String, ty: WidgetType },
}

impl RowField {
    pub fn name(&self) -> &str {
        match self {
            Self::Display(n) | Self::Widget { name: n, .. } => n,
        }
    }
}

/// A widget field discovered during parsing. Collected into a generated
/// state struct (`LituiFormState` or `AppState`).
#[derive(Clone)]
pub(crate) enum WidgetField {
    /// Standard stateful widget (slider, checkbox, textedit, etc.)
    Stateful { name: String, ty: WidgetType },
    /// Foreach collection — generates a row struct + `Vec<RowStruct>`
    Foreach {
        name: String,
        row_fields: Vec<RowField>,
    },
}

impl WidgetField {
    pub fn name(&self) -> &str {
        match self {
            Self::Stateful { name, .. } | Self::Foreach { name, .. } => name,
        }
    }

    pub fn ty(&self) -> Option<WidgetType> {
        match self {
            Self::Stateful { ty, .. } => Some(*ty),
            Self::Foreach { .. } => None,
        }
    }
}

// ── ParsedMarkdown output ────────────────────────────────────────

/// Structured output from parsing + codegen of a single markdown file.
///
/// Produced by `codegen_ast::document_to_parsed()`, consumed by
/// `codegen::parsed_to_include_tokens()` and `codegen::define_litui_app_impl()`.
pub(crate) struct ParsedMarkdown {
    pub(crate) code_body: Vec<proc_macro2::TokenStream>,
    pub(crate) widget_fields: Vec<WidgetField>,
    /// True if the generated code references `state` (e.g., display widgets).
    pub(crate) references_state: bool,
    /// Field names referenced by display widgets (for validation against `AppState`).
    pub(crate) display_refs: Vec<String>,
    /// Generated style lookup function tokens (when dynamic styling is used).
    pub(crate) style_table: Option<proc_macro2::TokenStream>,
    /// Widget config keys referenced via `{key}` in widget directives.
    pub(crate) used_widget_configs: std::collections::HashSet<String>,
}
