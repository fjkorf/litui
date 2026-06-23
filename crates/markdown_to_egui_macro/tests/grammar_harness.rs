//! Grammar test harness for Litui markdown parser.
//!
//! - Loads valid and invalid .md test cases from `tests/grammar_cases/`
//! - Invokes the parser directly
//! - Asserts on parse results or error diagnostics
//! - Designed for easy extension as grammar evolves

use std::fs;
use std::path::Path;

#[test]
fn grammar_harness() {
    let dir = Path::new("tests/grammar_cases");
    for entry in fs::read_dir(dir).expect("read_dir failed") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let content = fs::read_to_string(&path).expect("read file");
        let is_invalid = path.file_stem().unwrap().to_str().unwrap().contains("invalid");
        let parse_result = markdown_to_egui_macro::parse_markdown(&content);
        if is_invalid {
            assert!(parse_result.is_err(), "{} should fail to parse", path.display());
        } else {
            assert!(parse_result.is_ok(), "{} should parse successfully", path.display());
        }
    }
}
