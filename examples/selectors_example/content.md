---
styles:
  primary:
    bold: true
    color: "#4488FF"
  danger:
    bold: true
    color: "#FF4444"
  success:
    bold: true
    color: "#00CC66"
  warning:
    bold: true
    color: "#FFAA00"
  large:
    size: 20.0
    bold: true
  muted:
    italic: true
    color: "#888888"
    weak: true
  accent:
    color: "#FF6B00"
    bold: true
---

# Selectors and Styled Containers

## Class Selectors

Styled buttons using `.class` syntax:

[button.primary](Submit)

[button.danger](Cancel)

[button.primary.large](Big_Blue_Button)

## Inline Styled Spans

::accent(This text is orange and bold)

::muted(This text is gray and subtle)

## Styled Blockquotes

> Operation completed successfully. ::success

> Critical failure detected. ::danger

> Approaching rate limit — slow down. ::warning

## Styled Lists

- All systems operational ::success
- Deployment pending review ::warning
- Build failed on main ::danger
- Scheduled for deprecation ::muted

1. Configure environment ::success
2. Review security settings ::warning
3. Fix critical vulnerability ::danger
