---
page:
  name: Text
  label: Text
styles:
  big:
    bold: true
    size: 24.0
  code_style:
    monospace: true
    color: "#00FF88"
    background: "#2D2D2D"
  warning:
    bold: true
    color: "#FF4444"
    background: "#442222"
---

# Typography Showcase

## Headings

# Heading 1
## Heading 2
### Heading 3

## Inline Formatting

Regular text with **bold**, *italic*, and ~~strikethrough~~.

You can **combine *bold and italic*** together.

Inline `code` renders in monospace with a background.

## Styled Text via Frontmatter

This line is big and bold. ::big

This line is accented orange. ::accent

This line is subtle and gray. ::subtle

This line looks like code but is a paragraph. ::code_style

This line is a red warning with background. ::warning

## Blockquotes

> A simple blockquote.

> Nested blockquotes:
> > Level two
> > > Level three — with increasing indentation bars

## Styled Blockquotes

When a blockquote paragraph ends with `{key}`, the style colors the vertical bars and text.

> This is a warning — proceed with caution. ::warning

> Operation completed. ::success

> Critical failure detected. ::danger

## Code Blocks

```
fn main() {
    println!("Hello from a code block!");
    let x = 42;
}
```

## Horizontal Rules

Content above the rule.

---

Content below the rule.

## Line Breaks

Line with a soft break
continues on the next line.

Line with a hard break.\
Starts a new line.
