#!/usr/bin/env python3
"""Extract full API reference from Rust source files for Claude.

Parses .rs files to extract module docs, struct/enum definitions with fields,
function signatures with parameter types and return types, const declarations,
and module import relationships. Outputs a single knowledge/api/API.md.

Usage:
    python3 scripts/generate-doc-markdown.py
"""

import re
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
OUTPUT = WORKSPACE / "knowledge" / "api" / "API.md"

# Files to document (ordered by importance for code writing)
SOURCE_FILES = [
    "crates/markdown_to_egui_macro/src/lib.rs",
    "crates/markdown_to_egui_macro/src/frontmatter.rs",
    "crates/markdown_to_egui_macro/src/parse.rs",
    "crates/markdown_to_egui_macro/src/codegen.rs",
    "crates/markdown_to_egui_helpers/src/lib.rs",
    "crates/litui/src/lib.rs",
    "examples/demo_content/src/lib.rs",
]


def extract_module_docs(lines):
    """Extract leading //! module doc comments."""
    docs = []
    for line in lines:
        stripped = line.strip()
        if stripped.startswith("//!"):
            docs.append(stripped[3:].lstrip(" ") if len(stripped) > 3 else "")
        elif stripped == "" and docs:
            docs.append("")
        else:
            break
    while docs and docs[-1] == "":
        docs.pop()
    return docs


def extract_doc_comment(lines, start):
    """Extract consecutive /// lines. Returns (docs, next_line_idx)."""
    docs = []
    i = start
    while i < len(lines) and lines[i].strip().startswith("///"):
        content = lines[i].strip()[3:]
        docs.append(content.lstrip(" ") if content else "")
        i += 1
    while docs and docs[-1] == "":
        docs.pop()
    return docs, i


def find_closing_brace(lines, start_line):
    """Find line of closing } matching first { on or after start_line."""
    depth = 0
    for i in range(start_line, len(lines)):
        for ch in lines[i]:
            if ch == "{":
                depth += 1
            elif ch == "}":
                depth -= 1
                if depth == 0:
                    return i
    return len(lines) - 1


def extract_block(lines, start_line):
    """Extract brace-delimited block from start through matching }."""
    end = find_closing_brace(lines, start_line)
    return "\n".join(lines[start_line : end + 1])


def extract_fn_signature(lines, start_line):
    """Extract function signature up to (but not including) the opening {."""
    sig_parts = []
    for i in range(start_line, min(start_line + 20, len(lines))):
        line = lines[i].rstrip()
        if "{" in line:
            sig_parts.append(line[: line.index("{")].rstrip())
            break
        sig_parts.append(line)
    return "\n".join(sig_parts)


def extract_const_value(lines, start_line):
    """Extract const declaration including multi-line array values."""
    result = []
    for i in range(start_line, min(start_line + 30, len(lines))):
        result.append(lines[i].rstrip())
        if lines[i].rstrip().endswith(";") or lines[i].rstrip().endswith("];"):
            break
    return "\n".join(result)


def extract_use_crate(lines):
    """Extract use crate::... import statements."""
    imports = []
    i = 0
    while i < len(lines):
        line = lines[i].strip()
        if line.startswith("use crate::"):
            parts = [line]
            while not line.endswith(";"):
                i += 1
                if i >= len(lines):
                    break
                line = lines[i].strip()
                parts.append(line)
            imports.append(" ".join(parts))
        i += 1
    return imports


def process_file(filepath):
    """Process a .rs file and extract all items with full definitions."""
    lines = filepath.read_text().splitlines()
    rel_path = filepath.relative_to(WORKSPACE)

    result = {
        "path": str(rel_path),
        "module_docs": extract_module_docs(lines),
        "structs": [],
        "enums": [],
        "functions": [],
        "consts": [],
        "imports": extract_use_crate(lines),
    }

    i = 0
    while i < len(lines):
        stripped = lines[i].strip()

        # /// doc comment block → look for the item it documents
        if stripped.startswith("///"):
            docs, next_i = extract_doc_comment(lines, i)
            i = next_i

            # Skip #[...] attributes and blank lines
            while i < len(lines) and (
                lines[i].strip().startswith("#[") or lines[i].strip() == ""
            ):
                i += 1
            if i >= len(lines):
                break

            item_line = lines[i].strip()

            # Struct with body
            if re.match(r"(?:pub(?:\(crate\))?\s+)?struct\s+\w+", item_line):
                name = re.search(r"struct\s+(\w+)", item_line).group(1)
                if "{" in item_line or (
                    i + 1 < len(lines) and "{" in lines[i + 1]
                ):
                    code = extract_block(lines, i)
                else:
                    code = item_line
                result["structs"].append(
                    {"name": name, "docs": docs, "code": code, "line": i + 1}
                )

            # Enum
            elif re.match(r"(?:pub(?:\(crate\))?\s+)?enum\s+\w+", item_line):
                name = re.search(r"enum\s+(\w+)", item_line).group(1)
                code = extract_block(lines, i)
                result["enums"].append(
                    {"name": name, "docs": docs, "code": code, "line": i + 1}
                )

            # Function
            elif re.match(r"(?:pub(?:\(crate\))?\s+)?fn\s+\w+", item_line):
                name = re.search(r"fn\s+(\w+)", item_line).group(1)
                sig = extract_fn_signature(lines, i)
                result["functions"].append(
                    {"name": name, "docs": docs, "signature": sig, "line": i + 1}
                )

            # Const
            elif re.match(r"(?:pub(?:\(crate\))?\s+)?const\s+\w+", item_line):
                name = re.search(r"const\s+(\w+)", item_line).group(1)
                code = extract_const_value(lines, i)
                result["consts"].append(
                    {"name": name, "docs": docs, "code": code, "line": i + 1}
                )

            continue

        # Undocumented consts (like WIDGET_NAMES inside functions) — grab // comments above
        if re.match(r"\s*const\s+[A-Z_]+\s*:", stripped) and not stripped.startswith(
            "//"
        ):
            comment_docs = []
            j = i - 1
            while j >= 0 and lines[j].strip().startswith("//"):
                comment = lines[j].strip().lstrip("/ ").strip()
                comment_docs.insert(0, comment)
                j -= 1

            m = re.search(r"const\s+(\w+)", stripped)
            if m:
                code = extract_const_value(lines, i)
                result["consts"].append(
                    {
                        "name": m.group(1),
                        "docs": comment_docs,
                        "code": code.strip(),
                        "line": i + 1,
                    }
                )

        i += 1

    return result


def format_section(data):
    """Format one file's data as markdown."""
    parts = []
    parts.append(f"## `{data['path']}`\n")

    if data["module_docs"]:
        parts.append("\n".join(data["module_docs"]))
        parts.append("")

    for kind, key, code_field in [
        ("Structs", "structs", "code"),
        ("Enums", "enums", "code"),
        ("Functions", "functions", "signature"),
        ("Constants", "consts", "code"),
    ]:
        items = data[key]
        if not items:
            continue
        parts.append(f"### {kind}\n")
        for item in items:
            parts.append(f"#### `{item['name']}` (line {item['line']})\n")
            parts.append(f"```rust\n{item[code_field]}\n```\n")
            if item["docs"]:
                parts.append("\n".join(item["docs"]))
            parts.append("")

    if data["imports"]:
        parts.append("### Module Dependencies\n")
        parts.append("```rust")
        for imp in data["imports"]:
            parts.append(imp)
        parts.append("```\n")

    return "\n".join(parts)


def main():
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)

    sections = [
        "# API Reference\n",
        "Generated from source by `python3 scripts/generate-doc-markdown.py`.",
        "Contains full type definitions, function signatures, and module dependencies.\n",
        "---\n",
    ]

    total = 0
    for rel_path in SOURCE_FILES:
        fp = WORKSPACE / rel_path
        if not fp.exists():
            continue
        data = process_file(fp)
        n = len(data["structs"]) + len(data["enums"]) + len(data["functions"]) + len(data["consts"])
        total += n
        if data["module_docs"] or n > 0:
            sections.append(format_section(data))

    OUTPUT.write_text("\n".join(sections) + "\n")

    # Remove old per-crate files
    for old in OUTPUT.parent.glob("*.md"):
        if old.name != "API.md":
            old.unlink()
            print(f"  Removed: {old.name}")

    print(f"  Generated {OUTPUT.relative_to(WORKSPACE)} ({total} items)")


if __name__ == "__main__":
    main()
