# Images

> Run it: `cargo run -p markdown_macro_example`

Embed images in your UI with standard markdown syntax. The macro resolves paths at compile
time and emits `egui::Image` widgets.

## Basic syntax

```markdown
![Screenshot of the dashboard](assets/dashboard.png)

![](assets/logo.png)
```

Standard markdown image syntax. Alt text is optional but recommended -- it displays as
fallback text if the image fails to load.

## Path resolution

**Relative paths** are resolved against `CARGO_MANIFEST_DIR` at compile time and converted
to `file://` URIs. If your crate root is `/home/dev/myapp` and you write
`![](assets/logo.png)`, the macro emits `file:///home/dev/myapp/assets/logo.png`.

**Absolute URLs** are passed through unchanged:

```markdown
![Logo](https://example.com/logo.png)
![Local](file:///tmp/screenshot.png)
```

## You must install image loaders

This is the one thing that trips everyone up. Without this line, images render as a broken
icon with alt text:

```rust,ignore
egui_extras::install_image_loaders(ctx);
```

Call it once per frame in your `update()` method (it's cheap -- it no-ops after the first call).

### Full setup

Add the dependency:

```toml
[dependencies]
egui_extras = { version = "0.33", features = ["all_loaders"] }
```

Wire it into your app:

```rust,ignore
use eframe::egui;
use litui::*;

fn main() -> eframe::Result {
    eframe::run_native(
        "Image Example",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(MyApp))),
    )
}

struct MyApp;

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx); // <-- this line

        let render = include_markdown_ui!("content.md");
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                render(ui);
            });
        });
    }
}
```

Without `install_image_loaders`, egui has no idea how to decode PNG, JPEG, or SVG data.
The image widget still renders -- it just shows the alt text in a frame where the image
should be.

![Image widget](img/image_widget.png)

## Images in tables

Images work inside table cells:

```markdown
| Icon | Service | Status |
|------|---------|--------|
| ![](assets/api.png) | API Gateway | Online |
| ![](assets/db.png) | Database | Online |
| ![](assets/cache.png) | Cache | Degraded |
```

Each cell's image is sized to fit the cell. This is useful for icon grids or status boards.

## Alt text as fallback

When an image fails to load (wrong path, missing file, no loaders installed), egui displays
the alt text instead. Use descriptive alt text so the UI remains usable:

```markdown
![API status: online](assets/green_check.png)
```

If the image is missing, the user still sees "API status: online" rather than a blank space.

## Supported formats

With `egui_extras` `all_loaders` feature enabled: PNG, JPEG, GIF, BMP, TIFF, SVG, and more.
The exact list depends on the `image` crate's feature flags pulled in by `egui_extras`.

For SVG support specifically, the `all_loaders` feature includes the `resvg` loader.

## Next up

[Third-Party Widgets](crate::_tutorial::_09_third_party_widgets) -- integrate external egui widget crates into your markdown UI.
