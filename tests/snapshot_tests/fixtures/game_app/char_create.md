---
page:
  name: CharCreate
  label: Create
  default: true
widgets:
  species_list:
    max_height: 120.0
  job_list:
    max_height: 120.0
---

# Character Creation ::title

Pick a species and a job to begin.

| Species | Job |
|---------|-----|
| [select](chosen_species){species_list} | [select](chosen_job){job_list} |

## Preview ::stat

| Field | Value |
|-------|-------|
| **Name** | [display](preview_name) |
| **Class** | [display](preview_class) |

[button.success](Confirm)
