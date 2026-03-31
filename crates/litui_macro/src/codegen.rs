//! Code generation: `ParsedMarkdown` to final `TokenStream`.
//!
//! This module contains the two code generation paths:
//!
//! - [`parsed_to_include_tokens()`] -- for `include_litui_ui!`. Returns either
//!   a closure (no stateful widgets) or a `(fn, LituiFormState)` tuple (has widgets).
//!
//! - [`define_litui_app_impl()`] -- for `define_litui_app!`. Generates a
//!   `Page` enum, optional `AppState` struct, per-page `render_*()` functions,
//!   and a `LituiApp` struct with `show_nav()` and `show_page()` methods.

use quote::quote;
use syn::{Error, LitStr};

use crate::parse::{ParsedMarkdown, RowField, WidgetField, WidgetType};
use litui_parser::frontmatter::{
    Frontmatter, PageDef, ThemeDef, ThemeOverrides, capitalize_first, parse_hex_color,
};

/// Generate field definition tokens for a `WidgetField`, handling foreach row structs.
/// Returns (`field_def`, `field_default`, `optional_row_struct`).
fn widget_field_tokens(
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
            let struct_name = format!("{}Row", capitalize_first(name));
            let struct_ident = syn::Ident::new(&struct_name, proc_macro2::Span::call_site());

            let row_field_defs: Vec<proc_macro2::TokenStream> = row_fields
                .iter()
                .map(|rf| {
                    let rf_ident = syn::Ident::new(rf.name(), proc_macro2::Span::call_site());
                    match rf {
                        RowField::Display(_) => quote! { pub #rf_ident: String },
                        RowField::Widget { ty, .. } => {
                            let ty_tokens = ty.to_tokens();
                            quote! { pub #rf_ident: #ty_tokens }
                        }
                    }
                })
                .collect();

            let row_field_defaults: Vec<proc_macro2::TokenStream> = row_fields
                .iter()
                .map(|rf| {
                    let rf_ident = syn::Ident::new(rf.name(), proc_macro2::Span::call_site());
                    match rf {
                        RowField::Display(_) => quote! { #rf_ident: String::new() },
                        RowField::Widget { ty, .. } => {
                            let default_tokens = ty.default_tokens();
                            quote! { #rf_ident: #default_tokens }
                        }
                    }
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

/// Convert a [`ParsedMarkdown`] into the `TokenStream` returned by `include_litui_ui!`.
///
/// When `widget_fields` is empty, emits a simple closure `|ui: &mut egui::Ui| { ... }`.
/// When stateful widgets are present, emits a `LituiFormState` struct with `Default`,
/// a render function, and returns the tuple `(__md_render, LituiFormState::default())`.
pub(crate) fn parsed_to_include_tokens(parsed: ParsedMarkdown) -> proc_macro2::TokenStream {
    let ParsedMarkdown {
        code_body,
        widget_fields,
        references_state: _,
        display_refs: _,
        style_table,
        used_widget_configs: _,
    } = parsed;

    if widget_fields.is_empty() {
        // No stateful widgets — return a simple closure (backwards compatible)
        quote! {
            |ui: &mut egui::Ui| {
                #(#code_body)*
            }
        }
    } else {
        // Stateful widgets present — emit struct + render function + default state
        // Usage: let (render, mut state) = include_litui_ui!("form.md");
        //        render(ui, &mut state);

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
                pub struct LituiFormState {
                    #(#field_defs,)*
                }
                impl Default for LituiFormState {
                    fn default() -> Self {
                        Self {
                            #(#field_defaults,)*
                        }
                    }
                }
                fn __md_render(ui: &mut egui::Ui, state: &mut LituiFormState) {
                    #(#code_body)*
                }
                (__md_render, LituiFormState::default())
            }
        }
    }
}

// ── define_litui_app! ──────────────────────────────────────────

/// Convert a `PascalCase` name to `snake_case`.
fn to_snake_case(s: &str) -> String {
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push(ch.to_lowercase().next().unwrap_or(ch));
        } else {
            out.push(ch);
        }
    }
    out
}

/// Parsed input for `define_litui_app!`: optional `parent: "path"` followed
/// by comma-separated page file paths.
struct AppInput {
    parent_path: Option<LitStr>,
    page_paths: Vec<LitStr>,
}

impl syn::parse::Parse for AppInput {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let mut parent_path = None;

        // Check for `parent: "path"` keyword
        if input.peek(syn::Ident) {
            let fork = input.fork();
            let ident: syn::Ident = fork.parse()?;
            if ident == "parent" {
                // Commit to this parse path
                let _ident: syn::Ident = input.parse()?;
                input.parse::<syn::Token![:]>()?;
                parent_path = Some(input.parse::<LitStr>()?);
                if input.peek(syn::Token![,]) {
                    input.parse::<syn::Token![,]>()?;
                }
            }
        }

        let page_paths =
            syn::punctuated::Punctuated::<LitStr, syn::Token![,]>::parse_terminated(input)?
                .into_iter()
                .collect();

        Ok(Self {
            parent_path,
            page_paths,
        })
    }
}

/// Implementation of `define_litui_app!`.
///
/// Loads all page files (with optional parent frontmatter merging), validates
/// page metadata, collects widget fields across all pages, validates display
/// widget references, and generates:
/// - `Page` enum with `Default`, `ALL` const, and `label()` method
/// - `AppState` struct (if any page has stateful widgets)
/// - `render_shared(ui)` (if parent has markdown body)
/// - Per-page `render_{snake_name}()` functions
/// - `LituiApp` struct with `show_nav()` and `show_page()`
pub(crate) fn define_litui_app_impl(
    input: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    let app_input: AppInput = syn::parse2(input).map_err(|e: syn::Error| e.to_compile_error())?;

    if app_input.page_paths.is_empty() {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            "define_litui_app! requires at least one page file",
        )
        .to_compile_error());
    }

    // Load parent frontmatter if specified
    let parent_fm: Option<Frontmatter>;
    let mut shared_render_fn: Option<proc_macro2::TokenStream> = None;

    if let Some(ref parent_lit) = app_input.parent_path {
        let (fm, parsed) = crate::load_and_parse_md(&parent_lit.value(), None, parent_lit.span())?;
        if fm.page.is_some() {
            return Err(Error::new(
                parent_lit.span(),
                "Parent file must not have a `page:` section — it is not a page",
            )
            .to_compile_error());
        }
        if !parsed.widget_fields.is_empty() {
            return Err(Error::new(
                parent_lit.span(),
                "Parent file must not contain stateful widgets (slider, checkbox, etc.)",
            )
            .to_compile_error());
        }
        // Generate render_shared() if parent has markdown body
        if !parsed.code_body.is_empty() {
            let body = &parsed.code_body;
            shared_render_fn = Some(quote! {
                pub fn render_shared(ui: &mut egui::Ui) {
                    #(#body)*
                }
            });
        }
        parent_fm = Some(fm);
    } else {
        parent_fm = None;
    }

    // Extract nav configuration from parent frontmatter
    let nav_position = parent_fm
        .as_ref()
        .and_then(|fm| fm.nav.as_ref())
        .map(|n| n.position.as_str())
        .unwrap_or("top");
    let nav_show_all = parent_fm
        .as_ref()
        .and_then(|fm| fm.nav.as_ref())
        .map(|n| n.show_all)
        .unwrap_or(false);

    // Load and parse all page files
    struct PageInfo {
        page_def: PageDef,
        parsed: ParsedMarkdown,
        frontmatter_widget_keys: Vec<String>,
    }

    let mut pages: Vec<PageInfo> = Vec::new();

    for lit in &app_input.page_paths {
        let path_str = lit.value();
        let (fm, parsed) = crate::load_and_parse_md(&path_str, parent_fm.as_ref(), lit.span())?;
        let page_def = fm.page.ok_or_else(|| {
            Error::new(
                lit.span(),
                format!(
                    "File {:?} is missing `page:` section in frontmatter",
                    lit.value()
                ),
            )
            .to_compile_error()
        })?;
        let frontmatter_widget_keys = fm.widgets.keys().cloned().collect();
        pages.push(PageInfo {
            page_def,
            parsed,
            frontmatter_widget_keys,
        });
    }

    // Validate exactly one default
    let default_count = pages.iter().filter(|p| p.page_def.default).count();
    if default_count == 0 {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            "No page has `default: true` — exactly one page must be the default",
        )
        .to_compile_error());
    }
    if default_count > 1 {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            "Multiple pages have `default: true` — exactly one page must be the default",
        )
        .to_compile_error());
    }

    // ── Generate Page enum ────────────────────────────────────────

    let variant_idents: Vec<syn::Ident> = pages
        .iter()
        .map(|p| syn::Ident::new(&p.page_def.name, proc_macro2::Span::call_site()))
        .collect();

    let labels: Vec<&str> = pages.iter().map(|p| p.page_def.label.as_str()).collect();

    let default_variant = pages
        .iter()
        .find(|p| p.page_def.default)
        .map(|p| syn::Ident::new(&p.page_def.name, proc_macro2::Span::call_site()))
        .expect("validated above");

    let all_variants = &variant_idents;

    // Collect navigable variants: navigable == Some(true), or
    // navigable is None and panel is None (central pages are navigable by default)
    let nav_variants: Vec<&syn::Ident> = pages
        .iter()
        .zip(variant_idents.iter())
        .filter(|(p, _)| {
            p.page_def.navigable == Some(true)
                || (p.page_def.navigable.is_none() && p.page_def.panel.is_none())
        })
        .map(|(_, v)| v)
        .collect();

    let label_arms: Vec<proc_macro2::TokenStream> = variant_idents
        .iter()
        .zip(labels.iter())
        .map(|(v, l)| quote! { Self::#v => #l })
        .collect();

    let page_enum = quote! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum Page {
            #(#all_variants,)*
        }

        impl Default for Page {
            fn default() -> Self {
                Self::#default_variant
            }
        }

        impl Page {
            pub const ALL: &[Self] = &[#(Self::#all_variants,)*];

            pub const NAV_PAGES: &[Self] = &[#(Self::#nav_variants,)*];

            pub fn label(self) -> &'static str {
                match self {
                    #(#label_arms,)*
                }
            }
        }
    };

    // ── Collect all widget fields into flat AppState ──────────────

    // First pass: collect all input widget fields (non-display).
    // Second pass: add display-only fields that weren't declared by input widgets.
    // This two-pass approach ensures input widget types always win over
    // display self-declarations (which default to String).

    let mut all_widget_fields: Vec<WidgetField> = Vec::new();
    let mut seen_fields = std::collections::HashSet::new();
    let mut display_declared: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Pass 1: collect input widget fields (skip display self-declarations)
    for page_info in &pages {
        for field in &page_info.parsed.widget_fields {
            let field_name = field.name();
            // A display self-declaration is a String-typed field whose name
            // appears in display_refs. Input widgets (slider→f64, checkbox→bool,
            // etc.) are NOT self-declarations even if a [display] also references
            // the same field on the same page.
            let is_display_self_decl = field.ty() == Some(WidgetType::String)
                && page_info
                    .parsed
                    .display_refs
                    .contains(&field_name.to_owned());
            if is_display_self_decl {
                display_declared.insert(field_name.to_owned());
                continue;
            }
            if !seen_fields.insert(field_name.to_owned()) {
                // Same name already declared — allow if types match (shared
                // field across pages), error if types conflict.
                let existing = all_widget_fields.iter().find(|f| f.name() == field_name);
                if let Some(existing) = existing {
                    if existing.ty() == field.ty() {
                        continue;
                    }
                    return Err(Error::new(
                        proc_macro2::Span::call_site(),
                        format!(
                            "Widget field '{field_name}' declared with conflicting types — \
                             fields shared across pages must have the same type"
                        ),
                    )
                    .to_compile_error());
                }
            }
            all_widget_fields.push(field.clone());
        }
    }

    // Pass 2: add display-only fields (not declared by any input widget)
    for name in &display_declared {
        if !seen_fields.contains(name) {
            seen_fields.insert(name.clone());
            all_widget_fields.push(WidgetField::Stateful {
                name: name.clone(),
                ty: WidgetType::String,
            });
        }
    }

    // Pass 3: auto-declare `open: bool` fields for window pages with `open:` in frontmatter
    for page_info in &pages {
        if let Some(ref open_field) = page_info.page_def.open
            && !seen_fields.contains(open_field)
        {
            seen_fields.insert(open_field.clone());
            all_widget_fields.push(WidgetField::Stateful {
                name: open_field.clone(),
                ty: WidgetType::Bool,
            });
        }
    }

    let has_app_state = !all_widget_fields.is_empty();

    // Display widgets now self-declare their fields, so this validation
    // only catches typos where a display references a field that wasn't
    // declared by any widget (input or display) on any page.
    for page_info in &pages {
        for display_ref in &page_info.parsed.display_refs {
            if !seen_fields.contains(display_ref) {
                return Err(Error::new(
                    proc_macro2::Span::call_site(),
                    format!(
                        "[display]({display_ref}) references unknown field '{display_ref}' — \
                         no widget declares this field. Check for typos or add a \
                         widget like [slider]({display_ref}) on another page."
                    ),
                )
                .to_compile_error());
            }
        }
    }

    // ── Validate unused widget configs ─────────────────────────────
    // Union all used_widget_configs across pages, compare against all defined configs
    {
        let mut all_used: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut all_defined: std::collections::HashSet<String> = std::collections::HashSet::new();
        for page_info in &pages {
            all_used.extend(page_info.parsed.used_widget_configs.iter().cloned());
            all_defined.extend(page_info.frontmatter_widget_keys.iter().cloned());
        }
        let unused: Vec<&String> = all_defined
            .iter()
            .filter(|k| !all_used.contains(k.as_str()))
            .collect();
        if !unused.is_empty() {
            let mut names: Vec<&str> = unused.iter().map(|s| s.as_str()).collect();
            names.sort();
            return Err(Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "Unused widget config(s) in frontmatter `widgets:` section: {}. \
                     These are defined but never referenced by any widget via {{key}}. \
                     Remove them or check for typos.",
                    names.join(", ")
                ),
            )
            .to_compile_error());
        }
    }

    // ── Generate AppState struct ───────────────────────────────────

    let state_struct = if has_app_state {
        let mut field_defs = Vec::new();
        let mut field_defaults = Vec::new();
        let mut row_structs = Vec::new();
        for f in &all_widget_fields {
            let (def, default, row_struct) = widget_field_tokens(f);
            field_defs.push(def);
            field_defaults.push(default);
            if let Some(rs) = row_struct {
                row_structs.push(rs);
            }
        }

        quote! {
            #(#row_structs)*

            #[derive(Clone, Debug)]
            pub struct AppState {
                #(#field_defs,)*
            }
            impl Default for AppState {
                fn default() -> Self {
                    Self {
                        #(#field_defaults,)*
                    }
                }
            }
        }
    } else {
        quote! {}
    };

    // ── Generate render functions ──────────────────────────────────

    let mut render_fns: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut dispatch_arms: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut app_style_table: Option<proc_macro2::TokenStream> = None;

    // Container info per page for show_all() generation
    struct PageContainer {
        variant: syn::Ident,
        render_fn: syn::Ident,
        panel: Option<String>,
        width: Option<f32>,
        height: Option<f32>,
        has_mutable_widgets: bool,
        open: Option<String>,
    }
    let mut page_containers: Vec<PageContainer> = Vec::new();

    for (i, page_info) in pages.into_iter().enumerate() {
        // Collect style table from any page that needs it
        if page_info.parsed.style_table.is_some() && app_style_table.is_none() {
            app_style_table = page_info.parsed.style_table.clone();
        }
        let variant = &variant_idents[i];
        let snake_name = to_snake_case(&page_info.page_def.name);
        let render_fn_name = syn::Ident::new(
            &format!("render_{snake_name}"),
            proc_macro2::Span::call_site(),
        );
        let code_body = &page_info.parsed.code_body;

        // A page has mutable widgets if it has input widgets (slider, checkbox,
        // etc.) — NOT just display-self-declared fields, which are read-only.
        let has_mutable_widgets = page_info
            .parsed
            .widget_fields
            .iter()
            .any(|f| !page_info.parsed.display_refs.contains(&f.name().to_owned()));
        let needs_state = has_mutable_widgets
            || page_info.parsed.references_state
            || page_info
                .parsed
                .widget_fields
                .iter()
                .any(|f| page_info.parsed.display_refs.contains(&f.name().to_owned()));

        if has_mutable_widgets && has_app_state {
            // Page writes to state (sliders, checkboxes, etc.)
            render_fns.push(quote! {
                pub fn #render_fn_name(ui: &mut egui::Ui, state: &mut AppState) {
                    #(#code_body)*
                }
            });

            dispatch_arms.push(quote! {
                Page::#variant => #render_fn_name(ui, &mut self.state),
            });
        } else if needs_state && has_app_state {
            // Page only reads state (display widgets)
            render_fns.push(quote! {
                pub fn #render_fn_name(ui: &mut egui::Ui, state: &AppState) {
                    #(#code_body)*
                }
            });

            dispatch_arms.push(quote! {
                Page::#variant => #render_fn_name(ui, &self.state),
            });
        } else {
            render_fns.push(quote! {
                pub fn #render_fn_name(ui: &mut egui::Ui) {
                    #(#code_body)*
                }
            });

            dispatch_arms.push(quote! {
                Page::#variant => #render_fn_name(ui),
            });
        }

        // Collect container info for show_all() generation
        page_containers.push(PageContainer {
            variant: variant.clone(),
            render_fn: render_fn_name.clone(),
            panel: page_info.page_def.panel.clone(),
            width: page_info.page_def.width,
            height: page_info.page_def.height,
            has_mutable_widgets,
            open: page_info.page_def.open.clone(),
        });
    }

    // ── Generate LituiApp struct ──────────────────────────────────

    let state_field = if has_app_state {
        quote! { pub state: AppState, }
    } else {
        quote! {}
    };
    let state_default = if has_app_state {
        quote! { state: AppState::default(), }
    } else {
        quote! {}
    };

    // Generate show_all() container dispatch
    let has_containers = page_containers.iter().any(|p| p.panel.is_some());

    let mut side_panel_code: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut top_bottom_panel_code: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut window_code: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut central_dispatch: Vec<proc_macro2::TokenStream> = Vec::new();

    for pc in &page_containers {
        let variant = &pc.variant;
        let render_fn = &pc.render_fn;
        let render_call = if pc.has_mutable_widgets && has_app_state {
            quote! { #render_fn(ui, &mut self.state) }
        } else if has_app_state {
            quote! { #render_fn(ui, &self.state) }
        } else {
            quote! { #render_fn(ui) }
        };

        match pc.panel.as_deref() {
            Some("left") => {
                let snake = to_snake_case(&pc.variant.to_string());
                let id_str = format!("panel_{snake}");
                let width = pc.width.unwrap_or(200.0);
                if let Some(ref open_field) = pc.open {
                    let open_ident = syn::Ident::new(open_field, proc_macro2::Span::call_site());
                    side_panel_code.push(quote! {
                        {
                            if self.state.#open_ident {
                                egui::SidePanel::left(#id_str)
                                    .default_width(#width)
                                    .show(ctx, |ui| {
                                        egui::ScrollArea::vertical().show(ui, |ui| {
                                            #render_call;
                                        });
                                    });
                            }
                        }
                    });
                } else {
                    side_panel_code.push(quote! {
                        egui::SidePanel::left(#id_str)
                            .default_width(#width)
                            .show(ctx, |ui| {
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    #render_call;
                                });
                            });
                    });
                }
            }
            Some("right") => {
                let snake = to_snake_case(&pc.variant.to_string());
                let id_str = format!("panel_{snake}");
                let width = pc.width.unwrap_or(200.0);
                if let Some(ref open_field) = pc.open {
                    let open_ident = syn::Ident::new(open_field, proc_macro2::Span::call_site());
                    side_panel_code.push(quote! {
                        {
                            if self.state.#open_ident {
                                egui::SidePanel::right(#id_str)
                                    .default_width(#width)
                                    .show(ctx, |ui| {
                                        egui::ScrollArea::vertical().show(ui, |ui| {
                                            #render_call;
                                        });
                                    });
                            }
                        }
                    });
                } else {
                    side_panel_code.push(quote! {
                        egui::SidePanel::right(#id_str)
                            .default_width(#width)
                            .show(ctx, |ui| {
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    #render_call;
                                });
                            });
                    });
                }
            }
            Some("top") => {
                let snake = to_snake_case(&pc.variant.to_string());
                let id_str = format!("panel_{snake}");
                let height = pc.height.unwrap_or(100.0);
                if let Some(ref open_field) = pc.open {
                    let open_ident = syn::Ident::new(open_field, proc_macro2::Span::call_site());
                    top_bottom_panel_code.push(quote! {
                        {
                            if self.state.#open_ident {
                                egui::TopBottomPanel::top(#id_str)
                                    .default_height(#height)
                                    .show(ctx, |ui| {
                                        #render_call;
                                    });
                            }
                        }
                    });
                } else {
                    top_bottom_panel_code.push(quote! {
                        egui::TopBottomPanel::top(#id_str)
                            .default_height(#height)
                            .show(ctx, |ui| {
                                #render_call;
                            });
                    });
                }
            }
            Some("bottom") => {
                let snake = to_snake_case(&pc.variant.to_string());
                let id_str = format!("panel_{snake}");
                let height = pc.height.unwrap_or(100.0);
                if let Some(ref open_field) = pc.open {
                    let open_ident = syn::Ident::new(open_field, proc_macro2::Span::call_site());
                    top_bottom_panel_code.push(quote! {
                        {
                            if self.state.#open_ident {
                                egui::TopBottomPanel::bottom(#id_str)
                                    .default_height(#height)
                                    .show(ctx, |ui| {
                                        #render_call;
                                    });
                            }
                        }
                    });
                } else {
                    top_bottom_panel_code.push(quote! {
                        egui::TopBottomPanel::bottom(#id_str)
                            .default_height(#height)
                            .show(ctx, |ui| {
                                #render_call;
                            });
                    });
                }
            }
            Some("window") => {
                let label = variant.to_string();
                let width = pc.width.unwrap_or(350.0);
                if let Some(ref open_field) = pc.open {
                    // State-driven window: visibility controlled by a bool field on AppState.
                    // Extract the bool into a local to avoid simultaneous borrows of self.state.
                    let open_ident = syn::Ident::new(open_field, proc_macro2::Span::call_site());
                    window_code.push(quote! {
                        {
                            let mut __open = self.state.#open_ident;
                            egui::Window::new(#label)
                                .default_width(#width)
                                .open(&mut __open)
                                .show(ctx, |ui| {
                                    egui::ScrollArea::vertical().show(ui, |ui| {
                                        #render_call;
                                    });
                                });
                            self.state.#open_ident = __open;
                        }
                    });
                } else {
                    // Page-driven window: visible when navigated to
                    window_code.push(quote! {
                        if self.current_page == Page::#variant {
                            egui::Window::new(#label)
                                .default_width(#width)
                                .show(ctx, |ui| {
                                    egui::ScrollArea::vertical().show(ui, |ui| {
                                        #render_call;
                                    });
                                });
                        }
                    });
                }
            }
            _ => {
                // Central panel page — dispatched in show_page()
                central_dispatch.push(quote! {
                    Page::#variant => {
                        #render_call;
                    }
                });
            }
        }
    }

    // Check if a theme setup function will be generated
    let has_theme = parent_fm
        .as_ref()
        .and_then(|fm| fm.theme.as_ref())
        .is_some();
    let theme_call = if has_theme {
        quote! { __setup_theme(ctx); }
    } else {
        quote! {}
    };

    // Generate nav panel code based on nav position
    let nav_panel_code = match nav_position {
        "bottom" => quote! {
            egui::TopBottomPanel::bottom("litui_nav").show(ctx, |ui| {
                self.show_nav(ui);
            });
        },
        "none" => quote! {},
        _ => quote! {
            egui::TopBottomPanel::top("litui_nav").show(ctx, |ui| {
                self.show_nav(ui);
            });
        },
    };

    let central_panel_code = if central_dispatch.is_empty() {
        // No central pages — skip CentralPanel entirely so the host
        // renderer (e.g. Bevy 3D viewport) shows through.
        quote! {}
    } else {
        quote! {
            // Central panel (current page if it's a central page)
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    match self.current_page {
                        #(#central_dispatch)*
                        _ => {} // Non-central pages handled above
                    }
                });
            });
        }
    };

    let show_all_method = if has_containers {
        quote! {
            /// Render all pages in their designated containers.
            /// Side panels are always visible. Windows appear for the current page.
            /// If any pages lack a `panel:` directive, a central panel dispatches them.
            /// When all pages have explicit panels, no central panel is emitted.
            pub fn show_all(&mut self, ctx: &egui::Context) {
                #theme_call

                // Side panels (always visible, unless gated by open:)
                #(#side_panel_code)*

                // Top/bottom panels (always visible, unless gated by open:)
                #(#top_bottom_panel_code)*

                // Navigation
                #nav_panel_code

                #central_panel_code

                // Windows (shown when current page matches)
                #(#window_code)*
            }
        }
    } else {
        // No containers — generate a simple nav + central panel layout
        quote! {
            pub fn show_all(&mut self, ctx: &egui::Context) {
                #theme_call
                #nav_panel_code
                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        self.show_page(ui);
                    });
                });
            }
        }
    };

    let nav_pages_ref = if nav_show_all {
        quote! { Page::ALL }
    } else {
        quote! { Page::NAV_PAGES }
    };

    let litui_app = quote! {
        pub struct LituiApp {
            pub current_page: Page,
            #state_field
        }

        impl Default for LituiApp {
            fn default() -> Self {
                Self {
                    current_page: Page::default(),
                    #state_default
                }
            }
        }

        impl LituiApp {
            pub fn show_nav(&mut self, ui: &mut egui::Ui) {
                ui.horizontal_wrapped(|ui| {
                    ui.visuals_mut().button_frame = false;
                    for &page in #nav_pages_ref {
                        if ui.selectable_label(self.current_page == page, page.label()).clicked() {
                            self.current_page = page;
                        }
                    }
                });
            }

            pub fn show_page(&mut self, ui: &mut egui::Ui) {
                match self.current_page {
                    #(#dispatch_arms)*
                }
            }

            #show_all_method
        }
    };

    // ── Generate theme setup function ────────────────────────────

    let theme_setup_fn = if let Some(ref parent) = parent_fm {
        generate_theme_setup(&parent.theme)
    } else {
        None
    };

    // ── Combine all generated items ───────────────────────────────

    Ok(quote! {
        #page_enum
        #state_struct
        #app_style_table
        #theme_setup_fn
        #shared_render_fn
        #(#render_fns)*
        #litui_app
    })
}

// ── Theme setup codegen ──────────────────────────────────────────

/// Generate a `__setup_theme(ctx)` function from the frontmatter `theme:` section.
fn generate_theme_setup(theme: &Option<ThemeDef>) -> Option<proc_macro2::TokenStream> {
    let theme = theme.as_ref()?;

    fn theme_field_tokens(
        field_value: &Option<String>,
        field_path: proc_macro2::TokenStream,
    ) -> Option<proc_macro2::TokenStream> {
        let hex = field_value.as_ref()?;
        let [r, g, b] = parse_hex_color(hex).ok()?;
        Some(quote! { style.visuals.#field_path = egui::Color32::from_rgb(#r, #g, #b); })
    }

    fn override_tokens(overrides: &ThemeOverrides) -> Vec<proc_macro2::TokenStream> {
        let mut stmts = Vec::new();
        if let Some(t) = theme_field_tokens(&overrides.hyperlink_color, quote! { hyperlink_color })
        {
            stmts.push(t);
        }
        if let Some(t) = theme_field_tokens(&overrides.warn_fg_color, quote! { warn_fg_color }) {
            stmts.push(t);
        }
        if let Some(t) = theme_field_tokens(&overrides.error_fg_color, quote! { error_fg_color }) {
            stmts.push(t);
        }
        if let Some(t) = theme_field_tokens(&overrides.code_bg_color, quote! { code_bg_color }) {
            stmts.push(t);
        }
        if let Some(t) = theme_field_tokens(&overrides.panel_fill, quote! { panel_fill }) {
            stmts.push(t);
        }
        if let Some(t) = theme_field_tokens(&overrides.window_fill, quote! { window_fill }) {
            stmts.push(t);
        }
        if let Some(t) = theme_field_tokens(&overrides.faint_bg_color, quote! { faint_bg_color }) {
            stmts.push(t);
        }
        if let Some(t) =
            theme_field_tokens(&overrides.extreme_bg_color, quote! { extreme_bg_color })
        {
            stmts.push(t);
        }
        if let Some(t) =
            theme_field_tokens(&overrides.selection_color, quote! { selection.bg_fill })
        {
            stmts.push(t);
        }
        stmts
    }

    // Base overrides (applied to both themes)
    let base_overrides = ThemeOverrides {
        hyperlink_color: theme.hyperlink_color.clone(),
        warn_fg_color: theme.warn_fg_color.clone(),
        error_fg_color: theme.error_fg_color.clone(),
        code_bg_color: theme.code_bg_color.clone(),
        panel_fill: theme.panel_fill.clone(),
        window_fill: theme.window_fill.clone(),
        selection_color: theme.selection_color.clone(),
        faint_bg_color: theme.faint_bg_color.clone(),
        extreme_bg_color: theme.extreme_bg_color.clone(),
    };
    let base_stmts = override_tokens(&base_overrides);

    let dark_stmts = theme.dark.as_ref().map(override_tokens).unwrap_or_default();
    let light_stmts = theme
        .light
        .as_ref()
        .map(override_tokens)
        .unwrap_or_default();

    let has_conditional = !dark_stmts.is_empty() || !light_stmts.is_empty();

    let conditional_block = if has_conditional {
        quote! {
            if style.visuals.dark_mode {
                #(#dark_stmts)*
            } else {
                #(#light_stmts)*
            }
        }
    } else {
        quote! {}
    };

    Some(quote! {
        fn __setup_theme(ctx: &egui::Context) {
            ctx.style_mut(|style| {
                #(#base_stmts)*
                #conditional_block
            });
        }
    })
}
