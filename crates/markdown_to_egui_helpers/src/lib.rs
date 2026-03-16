//! Helper functions for standardized heading and text styles in egui.
//! Used by the markdown_to_egui_macro crate and consumers.
//!
//! The macro generates a closure whose body runs inside a single
//! `left_to_right` wrapping layout (the easy_mark pattern). These helpers
//! emit widgets into that flow: `ui.label()` for inline text,
//! `ui.end_row()` for line breaks, and `allocate_exact_size()` for
//! indentation prefixes (bullets, numbers, quote bars).

use eframe::egui::{Align2, Hyperlink, RichText, Sense, Separator, TextStyle, Ui, pos2, vec2};

/// Represents the current style context for markdown rendering.
#[derive(Clone, Debug, Default)]
pub struct StyleContext {
    pub heading_level: Option<u8>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub code: bool,
    pub small: bool,
    pub raised: bool,
    pub weak: bool,
    pub blockquote: bool,
    pub link: Option<String>,
    pub list_number: Option<usize>,
    pub bullet: bool,
}

// ── Block-level helpers (break out of the inline flow) ──────────────

/// Render an H1 heading (28pt, bold, strong color) as a block-level element.
pub fn h1(ui: &mut Ui, text: &str) {
    ui.end_row();
    ui.label(
        RichText::new(text)
            .text_style(TextStyle::Heading)
            .strong()
            .size(28.)
            .color(ui.visuals().strong_text_color()),
    );
    ui.end_row();
}

/// Render an H2 heading (22pt, bold) as a block-level element.
pub fn h2(ui: &mut Ui, text: &str) {
    ui.end_row();
    ui.label(
        RichText::new(text)
            .text_style(TextStyle::Heading)
            .strong()
            .size(22.)
            .color(ui.visuals().text_color()),
    );
    ui.end_row();
}

/// Render an H3 heading (18pt, bold) as a block-level element.
pub fn h3(ui: &mut Ui, text: &str) {
    ui.end_row();
    ui.label(
        RichText::new(text)
            .text_style(TextStyle::Heading)
            .strong()
            .size(18.)
            .color(ui.visuals().text_color()),
    );
    ui.end_row();
}

/// Render body text with the default body text style.
pub fn body(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).text_style(TextStyle::Body));
}

/// Block-level fenced code.
pub fn code(ui: &mut Ui, text: &str) {
    ui.end_row();
    ui.group(|ui| {
        ui.visuals_mut().override_text_color = Some(ui.visuals().code_bg_color);
        ui.add_space(2.0);
        ui.label(
            RichText::new(text)
                .text_style(TextStyle::Monospace)
                .color(ui.visuals().strong_text_color()),
        );
        ui.add_space(2.0);
    });
    ui.end_row();
}

/// Render a clickable hyperlink with underline and hyperlink color.
pub fn hyperlink(ui: &mut Ui, text: &str, url: &str) {
    ui.add(Hyperlink::from_label_and_url(
        RichText::new(text)
            .underline()
            .color(ui.visuals().hyperlink_color),
        url,
    ));
}

/// Render a horizontal separator as a block-level element.
pub fn separator(ui: &mut Ui) {
    ui.end_row();
    ui.add(Separator::default().horizontal());
    ui.end_row();
}

// ── Legacy text-only helpers (kept for backward compat) ─────────────

/// Legacy: render a bullet point with single-level indent. Prefer `emit_bullet_prefix`.
pub fn bullet_point(ui: &mut Ui, text: &str) {
    let row_height = ui.text_style_height(&TextStyle::Body);
    let one_indent = row_height / 2.0;
    ui.horizontal(|ui| {
        ui.allocate_exact_size(vec2(one_indent, row_height), Sense::hover());
        ui.painter().circle_filled(
            ui.cursor().left_center(),
            row_height / 8.0,
            ui.visuals().strong_text_color(),
        );
        ui.label(RichText::new(text).text_style(TextStyle::Body));
    });
}

/// Legacy: render a numbered list item with single-level indent. Prefer `emit_numbered_prefix`.
pub fn numbered_point(ui: &mut Ui, number: &str, text: &str) {
    let font_id = TextStyle::Body.resolve(ui.style());
    let row_height = ui.fonts_mut(|f| f.row_height(&font_id));
    let width = 3.0 * row_height / 2.0;
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(vec2(width, row_height), Sense::hover());
        let text_num = format!("{number}.");
        let text_color = ui.visuals().strong_text_color();
        ui.painter().text(
            rect.right_center(),
            Align2::RIGHT_CENTER,
            text_num,
            font_id.clone(),
            text_color,
        );
        ui.label(RichText::new(text).text_style(TextStyle::Body));
    });
}

/// Legacy: render a blockquote with single-level indent. Prefer `emit_quote_bars`.
pub fn quote_indent(ui: &mut Ui, text: &str) {
    let row_height = ui.text_style_height(&TextStyle::Body);
    let one_indent = row_height / 2.0;
    let (rect, _) = ui.allocate_exact_size(vec2(2.0 * one_indent, row_height), Sense::hover());
    let rect = rect.expand2(ui.style().spacing.item_spacing * 0.5);
    ui.painter().line_segment(
        [rect.center_top(), rect.center_bottom()],
        (1.0, ui.visuals().weak_text_color()),
    );
    ui.label(RichText::new(text).text_style(TextStyle::Body).weak());
}

/// Legacy: render italic text. Prefer `styled_label`.
pub fn italic(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).italics().text_style(TextStyle::Body));
}

/// Legacy: render underlined text. Prefer `styled_label_rich`.
pub fn underline(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).underline().text_style(TextStyle::Body));
}

/// Legacy: render strikethrough text. Prefer `styled_label`.
pub fn strikethrough(ui: &mut Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .strikethrough()
            .text_style(TextStyle::Body),
    );
}

/// Legacy: render small text. Prefer `styled_label_rich` with a size override.
pub fn small(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).small().text_style(TextStyle::Body));
}

/// Legacy: render raised (superscript-like) text.
pub fn raised(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).raised().text_style(TextStyle::Body));
}

/// Legacy: render weak (dimmed) text. Prefer `styled_label_rich` with `weak: true`.
pub fn weak(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).weak().text_style(TextStyle::Body));
}

// ── Inline composable helpers (used by the macro in flow layout) ────

/// Inline text with composable style flags.
pub fn styled_label(ui: &mut Ui, text: &str, bold: bool, is_italic: bool, is_strikethrough: bool) {
    let mut rt = RichText::new(text).text_style(TextStyle::Body);
    if bold {
        rt = rt.strong();
    }
    if is_italic {
        rt = rt.italics();
    }
    if is_strikethrough {
        rt = rt.strikethrough();
    }
    ui.label(rt);
}

/// Clickable hyperlink with composable inline styles.
pub fn styled_hyperlink(
    ui: &mut Ui,
    text: &str,
    url: &str,
    bold: bool,
    is_italic: bool,
    is_strikethrough: bool,
) {
    let mut rt = RichText::new(text)
        .underline()
        .color(ui.visuals().hyperlink_color);
    if bold {
        rt = rt.strong();
    }
    if is_italic {
        rt = rt.italics();
    }
    if is_strikethrough {
        rt = rt.strikethrough();
    }
    ui.add(Hyperlink::from_label_and_url(rt, url));
}

/// Inline monospace code (not block-level).
pub fn inline_code(ui: &mut Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .text_style(TextStyle::Monospace)
            .background_color(ui.visuals().code_bg_color),
    );
}

/// Fully parameterized label for frontmatter-styled text.
/// Color/background are `Option<[u8; 3]>` RGB tuples resolved at compile time.
#[expect(clippy::fn_params_excessive_bools)]
pub fn styled_label_rich(
    ui: &mut Ui,
    text: &str,
    bold: bool,
    is_italic: bool,
    is_strikethrough: bool,
    is_underline: bool,
    color: Option<[u8; 3]>,
    background: Option<[u8; 3]>,
    size: Option<f32>,
    monospace: bool,
    is_weak: bool,
) {
    let style = if monospace {
        TextStyle::Monospace
    } else {
        TextStyle::Body
    };
    let mut rt = RichText::new(text).text_style(style);
    if bold {
        rt = rt.strong();
    }
    if is_italic {
        rt = rt.italics();
    }
    if is_strikethrough {
        rt = rt.strikethrough();
    }
    if is_underline {
        rt = rt.underline();
    }
    if is_weak {
        rt = rt.weak();
    }
    if let Some([r, g, b]) = color {
        rt = rt.color(eframe::egui::Color32::from_rgb(r, g, b));
    }
    if let Some([r, g, b]) = background {
        rt = rt.background_color(eframe::egui::Color32::from_rgb(r, g, b));
    }
    if let Some(s) = size {
        rt = rt.size(s);
    }
    ui.label(rt);
}

// ── Row-prefix helpers for the flow layout ──────────────────────────
// These emit a row prefix (bullet, number, quote bars) into the
// wrapping layout. Content labels that follow flow on the same row.

/// End the current row and add paragraph spacing.
pub fn end_paragraph(ui: &mut Ui) {
    ui.end_row();
    ui.allocate_exact_size(vec2(0.0, 4.0), Sense::hover());
    ui.end_row();
}

/// Emit blockquote vertical bars for `depth` levels at the start of a row.
pub fn emit_quote_bars(ui: &mut Ui, depth: usize) {
    emit_quote_bars_colored(ui, depth, None);
}

/// Emit blockquote vertical bars with an optional custom color.
pub fn emit_quote_bars_colored(ui: &mut Ui, depth: usize, bar_color_override: Option<[u8; 3]>) {
    let row_height = ui.text_style_height(&TextStyle::Body);
    let bar_spacing = row_height / 2.0;
    let indent_width = bar_spacing * 2.0 * depth as f32;
    let (rect, _) = ui.allocate_exact_size(vec2(indent_width, row_height), Sense::hover());
    let bar_color = match bar_color_override {
        Some([r, g, b]) => eframe::egui::Color32::from_rgb(r, g, b),
        None => ui.visuals().weak_text_color(),
    };
    for i in 0..depth {
        let x = rect.left() + bar_spacing * 2.0 * i as f32 + bar_spacing;
        ui.painter().line_segment(
            [pos2(x, rect.top()), pos2(x, rect.bottom())],
            (1.0, bar_color),
        );
    }
}

/// Emit a bullet prefix with depth-based indentation.
pub fn emit_bullet_prefix(ui: &mut Ui, depth: usize) {
    emit_bullet_prefix_colored(ui, depth, None);
}

/// Emit a bullet prefix with an optional custom color.
pub fn emit_bullet_prefix_colored(ui: &mut Ui, depth: usize, color_override: Option<[u8; 3]>) {
    let row_height = ui.text_style_height(&TextStyle::Body);
    let one_indent = row_height / 2.0;
    let indent = one_indent * depth as f32;
    ui.allocate_exact_size(vec2(indent, row_height), Sense::hover());
    let center = ui.cursor().left_center();
    let color = match color_override {
        Some([r, g, b]) => eframe::egui::Color32::from_rgb(r, g, b),
        None => ui.visuals().strong_text_color(),
    };
    ui.painter().circle_filled(center, row_height / 8.0, color);
    ui.allocate_exact_size(vec2(one_indent, row_height), Sense::hover());
}

/// Emit a numbered prefix with depth-based indentation.
pub fn emit_numbered_prefix(ui: &mut Ui, depth: usize, number: &str) {
    emit_numbered_prefix_colored(ui, depth, number, None);
}

/// Emit a numbered prefix with an optional custom color.
pub fn emit_numbered_prefix_colored(
    ui: &mut Ui,
    depth: usize,
    number: &str,
    color_override: Option<[u8; 3]>,
) {
    let font_id = TextStyle::Body.resolve(ui.style());
    let row_height = ui.fonts_mut(|f| f.row_height(&font_id));
    let one_indent = row_height / 2.0;
    let num_width = 3.0 * row_height / 2.0;
    let indent = one_indent * depth.saturating_sub(1) as f32;
    if indent > 0.0 {
        ui.allocate_exact_size(vec2(indent, row_height), Sense::hover());
    }
    let (rect, _) = ui.allocate_exact_size(vec2(num_width, row_height), Sense::hover());
    let text_num = format!("{number}.");
    let text_color = match color_override {
        Some([r, g, b]) => eframe::egui::Color32::from_rgb(r, g, b),
        None => ui.visuals().strong_text_color(),
    };
    ui.painter().text(
        rect.right_center(),
        Align2::RIGHT_CENTER,
        text_num,
        font_id,
        text_color,
    );
    ui.allocate_exact_size(vec2(one_indent / 2.0, row_height), Sense::hover());
}

/// Emit a task list checkbox prefix with depth-based indentation.
///
/// Renders a non-interactive checkbox (checked or unchecked) as the list item prefix,
/// matching the GFM `- [x]` / `- [ ]` task list syntax.
pub fn emit_task_checkbox(
    ui: &mut Ui,
    depth: usize,
    checked: bool,
    color_override: Option<[u8; 3]>,
) {
    let row_height = ui.text_style_height(&TextStyle::Body);
    let one_indent = row_height / 2.0;
    let indent = one_indent * depth as f32;
    ui.allocate_exact_size(vec2(indent, row_height), Sense::hover());

    let box_size = row_height * 0.75;
    let (rect, _) = ui.allocate_exact_size(vec2(box_size, row_height), Sense::hover());
    let box_rect = eframe::egui::Rect::from_center_size(
        rect.center(),
        vec2(box_size, box_size),
    );

    let color = match color_override {
        Some([r, g, b]) => eframe::egui::Color32::from_rgb(r, g, b),
        None => ui.visuals().strong_text_color(),
    };

    let rounding = box_size * 0.15;
    ui.painter().rect_stroke(
        box_rect,
        rounding,
        eframe::egui::Stroke::new(1.5, color),
        eframe::egui::StrokeKind::Inside,
    );

    if checked {
        // Draw a checkmark
        let margin = box_size * 0.2;
        let left = box_rect.left() + margin;
        let right = box_rect.right() - margin;
        let top = box_rect.top() + margin;
        let bottom = box_rect.bottom() - margin;
        let mid_x = left + (right - left) * 0.35;
        let mid_y = bottom;
        let points = vec![
            eframe::egui::pos2(left, top + (bottom - top) * 0.5),
            eframe::egui::pos2(mid_x, mid_y),
            eframe::egui::pos2(right, top),
        ];
        ui.painter().add(eframe::egui::Shape::line(
            points,
            eframe::egui::Stroke::new(1.5, color),
        ));
    }

    ui.allocate_exact_size(vec2(one_indent, row_height), Sense::hover());
}

// ── Toggle switch widget ─────────────────────────────────────────────
// iOS-style toggle, adapted from egui's demo widget_gallery.

/// iOS-style toggle switch widget. Click to flip the boolean.
pub fn toggle_switch(ui: &mut Ui, on: &mut bool) -> eframe::egui::Response {
    let desired_size = ui.spacing().interact_size.y * vec2(2.0, 1.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
        let visuals = ui.style().interact_selectable(&response, *on);
        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        ui.painter().rect(
            rect,
            radius,
            visuals.bg_fill,
            visuals.bg_stroke,
            eframe::egui::StrokeKind::Inside,
        );
        let circle_x = eframe::egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = pos2(circle_x, rect.center().y);
        ui.painter()
            .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
    }
    response
}
