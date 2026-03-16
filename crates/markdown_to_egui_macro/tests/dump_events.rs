//! Debug test to dump pulldown-cmark event streams for various markdown patterns.

#[test]
fn dump_nested_bullet_list_4space() {
    eprintln!("\n=== Nested bullet list (4-space indent) ===");
    let md = "- Top-level bullet\n    - Nested bullet\n        - Deepest bullet\n";
    let parser = pulldown_cmark::Parser::new(md);
    for event in parser {
        eprintln!("{event:?}");
    }
}

#[test]
fn dump_nested_bullet_list_2space() {
    eprintln!("\n=== Nested bullet list (2-space indent) ===");
    let md = "- Top-level bullet\n  - Nested bullet\n    - Deepest bullet\n";
    let parser = pulldown_cmark::Parser::new(md);
    for event in parser {
        eprintln!("{event:?}");
    }
}

#[test]
fn dump_nested_ordered_list() {
    eprintln!("\n=== Nested ordered list ===");
    let md = "1. Ordered\n2. List\n   1. Nested\n   2. List\n";
    let parser = pulldown_cmark::Parser::new(md);
    for event in parser {
        eprintln!("{event:?}");
    }
}

#[test]
fn dump_blockquote_with_list() {
    eprintln!("\n=== Blockquote with nested list ===");
    let md = "> Blockquote level 1\n> > Blockquote level 2\n> > > Blockquote level 3\n> > > - List item 1\n> > >   - Nested list item 1\n";
    let parser = pulldown_cmark::Parser::new(md);
    for event in parser {
        eprintln!("{event:?}");
    }
}

#[test]
fn dump_table_with_alignment() {
    eprintln!("\n=== Table with alignment ===");
    let md = "| Left | Center | Right |\n|:-----|:------:|------:|\n| L1   | C1     | R1    |\n| L2   | C2     | R2    |\n";
    let mut opts = pulldown_cmark::Options::empty();
    opts.insert(pulldown_cmark::Options::ENABLE_TABLES);
    let parser = pulldown_cmark::Parser::new_ext(md, opts);
    for event in parser {
        eprintln!("{event:?}");
    }
}

#[test]
fn dump_widget_syntax() {
    eprintln!("\n=== Widget directive syntax ===");
    let md = "[button](Click_me){promo}\n\n[button](\"Click me\"){promo}\n\n[button](<Click me>){promo}\n\n[slider](volume){min=0, max=100}\n\n[spinner]()\n";
    let mut opts = pulldown_cmark::Options::empty();
    opts.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
    opts.insert(pulldown_cmark::Options::ENABLE_TABLES);
    let parser = pulldown_cmark::Parser::new_ext(md, opts);
    for event in parser {
        eprintln!("{event:?}");
    }
}

#[test]
fn dump_table_with_formatting() {
    eprintln!("\n=== Table with inline formatting ===");
    let md = "| Feature | Status |\n|---------|--------|\n| **Bold** | `code` |\n| *italic* | [link](url) |\n";
    let mut opts = pulldown_cmark::Options::empty();
    opts.insert(pulldown_cmark::Options::ENABLE_TABLES);
    opts.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
    let parser = pulldown_cmark::Parser::new_ext(md, opts);
    for event in parser {
        eprintln!("{event:?}");
    }
}

#[test]
fn dump_display_widget() {
    eprintln!("\n=== Display widget directive ===");
    let md = "[display](volume){vol_fmt}\n";
    let mut opts = pulldown_cmark::Options::empty();
    opts.insert(pulldown_cmark::Options::ENABLE_TABLES);
    let parser = pulldown_cmark::Parser::new_ext(md, opts);
    for event in parser {
        eprintln!("{event:?}");
    }
}

#[test]
fn dump_widget_in_table_cell() {
    eprintln!("\n=== Widget directive inside table cell ===");
    let md = "| Field | Value |\n|-------|-------|\n| Volume | [display](volume){vol_fmt} |\n| Muted | [display](muted) |\n";
    let mut opts = pulldown_cmark::Options::empty();
    opts.insert(pulldown_cmark::Options::ENABLE_TABLES);
    let parser = pulldown_cmark::Parser::new_ext(md, opts);
    for event in parser {
        eprintln!("{event:?}");
    }
}

#[test]
fn dump_inline_styled_span() {
    eprintln!("\n=== Inline styled text span (angle brackets) ===");
    let md = "Text with [.accent](<orange bold text>) inline.\n";
    let mut opts = pulldown_cmark::Options::empty();
    opts.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
    opts.insert(pulldown_cmark::Options::ENABLE_TABLES);
    let parser = pulldown_cmark::Parser::new_ext(md, opts);
    for event in parser {
        eprintln!("{event:?}");
    }
}

#[test]
fn dump_link_with_selectors() {
    eprintln!("\n=== Link with ID and class selectors ===");
    let md = "[button#submit.premium.large](Click_me)\n\n[slider#vol.accent](volume){vol}\n";
    let parser = pulldown_cmark::Parser::new(md);
    for event in parser {
        eprintln!("{event:?}");
    }
}
