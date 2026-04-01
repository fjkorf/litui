//! Pure-data AST types for parsed markdown documents.
//!
//! These types represent the complete structure of a litui markdown document
//! after parsing, with no `TokenStream` or proc-macro dependencies. This
//! enables independent testing of the parser.

use std::collections::HashSet;

// ── Inline-level nodes ─────────────────────────────────────

/// Inline content within a paragraph, list item, table cell, or heading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inline {
    /// Plain text with style flags.
    Text {
        text: String,
        bold: bool,
        italic: bool,
        strikethrough: bool,
    },
    /// Backtick-delimited inline code span.
    InlineCode(String),
    /// Hyperlink with display text, URL, and style flags.
    Link {
        text: String,
        url: String,
        bold: bool,
        italic: bool,
        strikethrough: bool,
    },
    /// Image with alt text and URL (resolved to absolute path at parse time).
    Image { alt: String, url: String },
    /// `::class(text)` inline styled span.
    StyledSpan {
        class: String,
        text: String,
        bold: bool,
        italic: bool,
        strikethrough: bool,
    },
    /// `[.class](text)` class-only styled span (link syntax with empty base name).
    ClassSpan {
        classes: Vec<String>,
        text: String,
        bold: bool,
        italic: bool,
        strikethrough: bool,
    },
    /// Widget directive: `[widget_name#id.class](field){config}`.
    Widget(WidgetDirective),
    /// `{field}` reference inside a foreach block.
    ForeachField(String),
}

// ── Widget types ───────────────────────────────────────────

/// A fully parsed widget directive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WidgetDirective {
    pub widget_type: WidgetKind,
    /// The URL portion — field name or literal value.
    pub field: String,
    /// `#id` selector for `ui.push_id()`.
    pub id: Option<String>,
    /// `.class` selectors resolved against frontmatter styles.
    pub classes: Vec<String>,
    /// `{key}` config reference, empty if none.
    pub config_key: String,
}

/// Known widget types that intercept link syntax.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WidgetKind {
    Button,
    Progress,
    Spinner,
    Slider,
    DoubleSlider,
    Checkbox,
    Textedit,
    Textarea,
    Password,
    Dragvalue,
    Display,
    Radio,
    Combobox,
    Color,
    Toggle,
    Selectable,
    Select,
    Log,
    Datepicker,
}

impl WidgetKind {
    /// Parse a widget name string into a `WidgetKind`, if it matches.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "button" => Some(Self::Button),
            "progress" => Some(Self::Progress),
            "spinner" => Some(Self::Spinner),
            "slider" => Some(Self::Slider),
            "double_slider" => Some(Self::DoubleSlider),
            "checkbox" => Some(Self::Checkbox),
            "textedit" => Some(Self::Textedit),
            "textarea" => Some(Self::Textarea),
            "password" => Some(Self::Password),
            "dragvalue" => Some(Self::Dragvalue),
            "display" => Some(Self::Display),
            "radio" => Some(Self::Radio),
            "combobox" => Some(Self::Combobox),
            "color" => Some(Self::Color),
            "toggle" => Some(Self::Toggle),
            "selectable" => Some(Self::Selectable),
            "select" => Some(Self::Select),
            "log" => Some(Self::Log),
            "datepicker" => Some(Self::Datepicker),
            _ => None,
        }
    }

    /// All known widget names, for error messages.
    pub const ALL_NAMES: &[&str] = &[
        "button",
        "progress",
        "spinner",
        "slider",
        "double_slider",
        "checkbox",
        "textedit",
        "textarea",
        "password",
        "dragvalue",
        "display",
        "radio",
        "combobox",
        "color",
        "toggle",
        "selectable",
        "select",
        "log",
        "datepicker",
    ];
}

// ── Widget field / type system ─────────────────────────────

/// The Rust type of a widget's state field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetType {
    F64,
    Bool,
    U32,
    Usize,
    String,
    ByteArray4,
    VecString,
    /// `chrono::NaiveDate` — used by the datepicker widget.
    Date,
}

/// A field inside a foreach row struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowField {
    /// `{field}` text reference — always String, display-only.
    Display(String),
    /// Widget inside foreach — typed, interactive (checkbox → bool, button → u32, etc.).
    Widget {
        name: String,
        ty: WidgetType,
        kind: WidgetKind,
    },
    /// Nested foreach inside a foreach — generates a child row struct + `Vec<ChildRow>`.
    Foreach {
        name: String,
        row_fields: Vec<RowField>,
        is_tree: bool,
    },
}

impl RowField {
    pub fn name(&self) -> &str {
        match self {
            Self::Display(n) | Self::Widget { name: n, .. } | Self::Foreach { name: n, .. } => n,
        }
    }
}

/// A widget field discovered during parsing, collected into a generated state struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WidgetField {
    /// Standard stateful widget (slider, checkbox, textedit, etc.).
    Stateful { name: String, ty: WidgetType },
    /// Foreach collection — generates a row struct + `Vec<RowStruct>`.
    Foreach {
        name: String,
        row_fields: Vec<RowField>,
        /// When true, the row struct gets `children: Vec<Self>` and rendering is recursive.
        is_tree: bool,
    },
}

impl WidgetField {
    /// The field name on the generated state struct.
    pub fn name(&self) -> &str {
        match self {
            Self::Stateful { name, .. } | Self::Foreach { name, .. } => name,
        }
    }

    /// The Rust type, if this is a stateful field (not foreach).
    pub fn ty(&self) -> Option<WidgetType> {
        match self {
            Self::Stateful { ty, .. } => Some(*ty),
            Self::Foreach { .. } => None,
        }
    }
}

// ── Block-level nodes ──────────────────────────────────────

/// Heading level (H1–H4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadingLevel {
    H1,
    H2,
    H3,
    H4,
}

/// A `::key` or `::$field` suffix on paragraphs, headings, or list items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StyleSuffix {
    /// Compile-time style reference: `::key`.
    Static(String),
    /// Runtime style reference: `::$field`.
    Dynamic(String),
}

/// A single list item with its inline content and optional nested blocks.
#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    pub fragments: Vec<Inline>,
    /// Nested blocks (sub-lists, etc.) within this item.
    pub children: Vec<Block>,
    pub style_suffix: Option<StyleSuffix>,
    /// Nesting depth (1 = top level, 2 = nested, etc.) for indent prefix.
    pub depth: usize,
    /// The list kind at this item's level (for correct bullet/number prefix).
    pub kind: ListKind,
}

/// Ordered vs unordered list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListKind {
    Unordered,
    /// Ordered list starting at the given number.
    Ordered(usize),
}

/// A table cell's inline content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableCell {
    pub fragments: Vec<Inline>,
}

/// Column alignment parsed from GFM table syntax (`:---`, `:---:`, `---:`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnAlignment {
    Left,
    Center,
    Right,
}

/// Alignment for `::: horizontal` directive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalAlign {
    /// Default left-to-right (no argument).
    Left,
    /// `::: horizontal center`
    Center,
    /// `::: horizontal right`
    Right,
    /// `::: horizontal space-between` — uses `::: next` to split left/right groups.
    SpaceBetween,
}

/// Block-level directives opened by `:::`.
#[derive(Debug, Clone, PartialEq)]
pub enum Directive {
    Foreach {
        field: String,
        body: Vec<Block>,
        row_fields: Vec<RowField>,
        /// When true, each row has `children: Vec<Self>` and the body renders recursively.
        is_tree: bool,
    },
    If {
        field: String,
        body: Vec<Block>,
    },
    Style {
        field: String,
        body: Vec<Block>,
    },
    Frame {
        style_name: Option<String>,
        body: Vec<Block>,
    },
    Horizontal {
        align: HorizontalAlign,
        /// For `space-between`: two groups split by `::: next`.
        /// For other alignments: single body.
        body: Vec<Block>,
        /// Second group (right side) for `space-between`. Empty for other alignments.
        right_body: Vec<Block>,
    },
    Columns {
        count: usize,
        /// Per-column weights (e.g., `[3, 1, 1]`). Empty means equal weights.
        weights: Vec<usize>,
        columns: Vec<Vec<Block>>,
    },
    /// `::: center` — center-align block content.
    Center {
        body: Vec<Block>,
    },
    /// `::: right` — right-align block content.
    Right {
        body: Vec<Block>,
    },
    /// `::: fill` — stretch content to fill available width.
    Fill {
        body: Vec<Block>,
    },
    /// `::: collapsing` — egui `CollapsingHeader` wrapper.
    Collapsing {
        /// Static title text (used when `title_field` is `None`).
        title: String,
        /// If `Some`, the title comes from this field (e.g., `{name}` in foreach).
        title_field: Option<String>,
        /// Optional bool field on AppState for bidirectional open/close tracking.
        open_field: Option<String>,
        /// Default open state (currently always `false`).
        default_open: bool,
        body: Vec<Block>,
        /// Unique index for generating stable egui IDs.
        collapsing_index: usize,
    },
}

/// Top-level block node in the document.
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    /// Heading with accumulated text (not inline fragments — headings use
    /// raw text since they don't support mixed inline content in the current grammar).
    Heading {
        level: HeadingLevel,
        text: String,
        style_suffix: Option<StyleSuffix>,
    },
    /// Paragraph with mixed inline content.
    Paragraph {
        fragments: Vec<Inline>,
        style_suffix: Option<StyleSuffix>,
    },
    /// Ordered or unordered list.
    List {
        kind: ListKind,
        items: Vec<ListItem>,
    },
    /// Table with header row and body rows.
    Table {
        headers: Vec<TableCell>,
        rows: Vec<Vec<TableCell>>,
        num_columns: usize,
        /// Unique index for generating stable Grid IDs.
        table_index: usize,
        /// Per-column alignment from GFM table syntax.
        alignments: Vec<ColumnAlignment>,
    },
    /// Fenced code block.
    CodeBlock { text: String },
    /// Blockquote — wraps inner blocks with quote bar rendering at the given depth.
    BlockQuote { depth: usize, blocks: Vec<Self> },
    /// Horizontal rule / separator.
    HorizontalRule,
    /// Block directive (`:::` syntax).
    Directive(Directive),
    /// Standalone image (outside table context).
    Image { alt: String, url: String },
    /// Standalone widget (outside table context).
    Widget(WidgetDirective),
    /// Item spacing insertion.
    Spacing(f32),
    /// Item spacing override (`ui.spacing_mut().item_spacing.y`).
    ItemSpacingOverride(f32),
}

// ── Document-level output ──────────────────────────────────

/// The complete parse result for a single markdown document.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub blocks: Vec<Block>,
    pub widget_fields: Vec<WidgetField>,
    /// True if the generated code references `state` (e.g., display widgets).
    pub references_state: bool,
    /// Field names referenced by display widgets.
    pub display_refs: Vec<String>,
    /// True if dynamic `::$field` styling is used, requiring a style lookup table.
    pub needs_style_table: bool,
    /// Widget config keys referenced via `{key}` in widget directives.
    pub used_widget_configs: HashSet<String>,
}
