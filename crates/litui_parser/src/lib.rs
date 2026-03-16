//! litui parser — pure-data markdown parsing for litui.
//!
//! This crate parses markdown content (with YAML frontmatter, widget directives,
//! and block directives) into a pure-data AST that can be independently tested.
//! It has **no** dependency on `proc-macro2`, `quote`, or `syn`.
//!
//! The macro crate (`litui_macro`) consumes the AST to generate
//! `TokenStream` code for egui rendering.

pub mod ast;
pub mod error;
pub mod frontmatter;
pub mod parse;
