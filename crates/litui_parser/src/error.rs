//! Parse error types for litui parser.
//!
//! Errors carry a message but no `proc_macro2::Span` — the macro crate
//! wraps these into `compile_error!` with the appropriate source span.

use std::fmt;

/// An error encountered during markdown parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
}

impl ParseError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ParseError {}
