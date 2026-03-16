---
styles:
  title:
    bold: true
    color: "#FFD700"
    size: 28.0
  accent:
    color: "#FF6B00"
    bold: true
  danger:
    bold: true
    color: "#FF4444"
  success:
    bold: true
    color: "#00CC66"
  muted:
    italic: true
    color: "#888888"
    weak: true
---

# Hello litui ::title

This is a **bold** statement with *italic* and ~~strikethrough~~ text.

::accent(This inline span is orange and bold.)

::muted(This text is gray and subtle.)

## Styled Paragraphs

This paragraph has a success style applied. ::success

This paragraph warns of danger. ::danger

## Lists

- First item
- Second item with **bold**
- Nested list:
  - Sub-item one
  - Sub-item two

1. Ordered first
2. Ordered second
3. Ordered third

## Blockquotes

> This is a blockquote.
> It can span multiple lines.

## Code

Inline `code` works too.

```
fn main() {
    println!("Hello from a code block!");
}
```

## Links

Visit [egui](https://github.com/emilk/egui) for more.

---

Markdown with frontmatter styles — all resolved at compile time.
