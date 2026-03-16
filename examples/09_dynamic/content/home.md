---
page:
  name: Home
  label: Home
  default: true
---

# Dynamic Content ::title

This page demonstrates runtime data: foreach iteration, conditionals, and dynamic styling.

## Inventory

::: foreach items

| {name} | {quantity} | {weight} |
|--------|-----------|----------|

:::

## Conditional Section

::: if show_details

::: frame panel

**Details visible.** Toggle `show_details` from code to hide this section.

:::

:::

## Dynamic Status

Server status: [display](status_text) ::$status_style
