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

## Collapsible Sections

::: collapsing "Inventory Details"

Toggle this section to show or hide the summary.

- Weight capacity: 50 lb
- Gold: 127

:::

::: collapsing "Server Info" {show_server_info}

Server status: [display](status_text) ::$status_style

This section's open/close state is tracked in `AppState`.

:::
