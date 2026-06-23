// _codegen.rs: In-progress, corrected version of codegen.rs
// Copying and correcting codegen.rs chunk by chunk for clarity and maintainability.

//! Code generation: `ParsedMarkdown` to final `TokenStream`.
//! This module contains the two code generation paths:
//! - [`parsed_to_include_tokens()`] -- for `include_markdown_ui!`. Returns either
//!   a closure (no stateful widgets) or a `(fn, MdFormState)` tuple (has widgets).
//! - [`define_markdown_app_impl()`] -- for `define_markdown_app!`. Generates a
//!   `Page` enum, optional `AppState` struct, per-page `render_*()` functions,
//!   and an `MdApp` struct with `show_nav()` and `show_page()` methods.

use quote::quote;
use syn::{Error, LitStr};
use crate::frontmatter::{Frontmatter, PageDef};
use markdown_to_egui_parser::{ParsedMarkdown, WidgetField, WidgetType};
use crate::parse::capitalize_first;

// Helper: Generate field definition tokens for a WidgetField, handling foreach row structs.
// Returns (field_def, field_default, optional_row_struct).
pub fn widget_field_tokens(
    f: &WidgetField,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    Option<proc_macro2::TokenStream>,
) {
    match f {
        WidgetField::Stateful { name, ty } => {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            let ty_tokens = ty.to_tokens();
            let default_tokens = ty.default_tokens();
            (
                quote! { pub #ident: #ty_tokens },
                quote! { #ident: #default_tokens },
                None,
            )
        }
        WidgetField::Foreach { name, row_fields } => {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            let struct_name = capitalize_first(name);
            let struct_ident = syn::Ident::new(&struct_name, proc_macro2::Span::call_site());
            let row_field_defs: Vec<proc_macro2::TokenStream> = row_fields
                .iter()
                .map(|rf| {
                    let rf_ident = syn::Ident::new(rf, proc_macro2::Span::call_site());
                    quote! { pub #rf_ident: String }
                })
                .collect();
            let row_field_defaults: Vec<proc_macro2::TokenStream> = row_fields
                .iter()
                .map(|rf| {
                    let rf_ident = syn::Ident::new(rf, proc_macro2::Span::call_site());
                    quote! { #rf_ident: String::new() }
                })
                .collect();
            let row_struct = quote! {
                #[derive(Clone, Debug)]
                #[allow(non_camel_case_types)]
                pub struct #struct_ident {
                    #(#row_field_defs,)*
                }
                impl Default for #struct_ident {
                    fn default() -> Self {
                        Self {
                            #(#row_field_defaults,)*
                        }
                    }
                }
            };
            (
                quote! { pub #ident: Vec<#struct_ident> },
                quote! { #ident: Vec::new() },
                Some(row_struct),
            )
        }
    }
}

// Macro codegen for include_markdown_ui!
pub fn parsed_to_include_tokens(parsed: markdown_to_egui_parser::ParsedMarkdown) -> proc_macro2::TokenStream {
    let markdown_to_egui_parser::ParsedMarkdown {
        code_body,
        widget_fields,
        references_state: _,
        display_refs: _,
        style_table,
        used_widget_configs: _,
    } = parsed;

    if widget_fields.is_empty() {
        quote! {
            |ui: &mut egui::Ui| {
                #(#code_body)*
            }
        }
    } else {
        let mut field_defs = Vec::new();
        let mut field_defaults = Vec::new();
        let mut row_structs = Vec::new();
        for f in &widget_fields {
            let (def, default, row_struct) = widget_field_tokens(f);
            field_defs.push(def);
            field_defaults.push(default);
            if let Some(rs) = row_struct {
                row_structs.push(rs);
            }
        }
        quote! {
            {
                #(#row_structs)*
                #style_table
                #[derive(Clone, Debug)]
                #[allow(non_camel_case_types)]
                pub struct MdFormState {
                    #(#field_defs,)*
                }
                impl Default for MdFormState {
                    fn default() -> Self {
                        Self {
                            #(#field_defaults,)*
                        }
                    }
                }
                fn __md_render(ui: &mut egui::Ui, state: &mut MdFormState) {
                    #(#code_body)*
                }
                (__md_render, MdFormState::default())
            }
        }
    }
}
