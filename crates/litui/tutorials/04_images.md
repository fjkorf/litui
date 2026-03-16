# Images

> Run it: `cargo run -p tut_04_images`

This tutorial adds **images** using standard markdown syntax.

## What's new

`![alt text](url)` renders via `egui::Image`. Supports HTTP URLs and `file://` local paths.

## The markdown

```text
![Ferris the crab](https://rustacean.net/assets/rustacean-flat-noshadow.png)
```

## Setup required

Your app must call `egui_extras::install_image_loaders(ctx)` once per frame (before rendering) to enable image loading:

```rust,ignore
egui_extras::install_image_loaders(ctx);
```

Add the dependency:

```toml
egui_extras = { version = "0.33", features = ["all_loaders"] }
```

## Images in tables

Images work inside table cells:

```text
| Mascot | Description |
|--------|-------------|
| ![Ferris](https://rustacean.net/assets/rustacean-flat-noshadow.png) | The Rust mascot |
```

## Expert tip

The macro emits `ui.image(egui::include_image!("url"))` for each image. The `egui_extras` image loaders handle format detection (PNG, JPEG, GIF, BMP, SVG) and caching. For local files, use `file://` URIs — the path is resolved relative to the crate manifest directory at compile time.

## What we built

Images from URLs and local files, embedded in markdown content and table cells.
