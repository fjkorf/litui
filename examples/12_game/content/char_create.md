---
page:
  name: CharCreate
  label: Create Character
  default: true
widgets:
---

# Create Your Character ::title

Choose a species and a class, then confirm to begin your adventure.

| Species | Class |
|---------|-------|
| [select](chosen_species){species_list} | [select](chosen_job){job_list} |

## Preview ::stat

| Stat | Value |
|------|-------|
| **Name** | [display](preview_name) |
| **HP** | [display](preview_hp) |
| **STR** | [display](preview_str) |
| **DEX** | [display](preview_dex) |

[button.success](Start_Game){confirm}
