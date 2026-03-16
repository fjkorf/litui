---
page:
  name: Lists
  label: Lists
---

# Lists

## Bullet Lists

- First item
- Second item
- Third item

## Nested Bullets

- Top level
  - Nested level
    - Deeply nested

## Ordered Lists

1. First
2. Second
3. Third

## Nested Ordered

1. Outer item one
2. Outer item two
   1. Inner item A
   2. Inner item B
3. Outer item three

## Mixed Nesting

- Unordered parent
  1. Ordered child one
  2. Ordered child two
- Another unordered
  - Bullet child
    1. Deep ordered

## Lists in Blockquotes

> Quoted list:
> - Item A
> - Item B
>   - Nested in quote
>
> > Deeper quote:
> > 1. Numbered in deep quote
> > 2. Second item

## Formatting in Lists

- **Bold item**
- *Italic item*
- ~~Strikethrough item~~
- Item with `inline code`
- Item with a [link](https://example.com)

## Styled Lists

When a list item ends with `{key}`, the style's color is applied to both the bullet/number and the text.

- All systems operational ::success
- Deployment pending review ::warning
- Build failed on main branch ::danger
- Deprecated — will be removed in v2 ::muted

1. Configure environment ::success
2. Review security settings ::warning
3. Fix critical vulnerability ::danger
