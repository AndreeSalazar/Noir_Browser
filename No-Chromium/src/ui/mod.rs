//! UI Module - User interface components
//!
//! - click_feedback: Highlights, cursor types, focus rings
//! - element_highlight: Resalta elementos interactivos
//! - markdown: Rendering de markdown
//! - card_layout: Card-style rendering
//! - list_render: Listas ordenadas/no ordenadas con bullets
//! - table_render: Tablas HTML
//! - css_grid: CSS Grid layout (display: grid)

#![allow(dead_code)]

pub mod click_feedback;
pub mod element_highlight;
pub mod markdown;
pub mod card_layout;
pub mod list_render;
pub mod table_render;
pub mod css_grid;
pub mod css_containment;

pub use click_feedback::{
    CursorType, InteractiveRole, InteractiveBox, FocusRing,
    ClickEffect, ClickFeedback, InteractionState,
};
pub use element_highlight::{
    HighlightCategory, Highlight, HighlightMode, ElementHighlighter,
};
pub use markdown::{
    MarkdownStyle, MarkdownSegment, MarkdownLine, MarkdownRenderer,
};
pub use card_layout::{
    Card, CardLayout, CardMetadata, CardGrid, format_views_short, format_age,
};
pub use list_render::{
    List, ListType, ListItem, ListRenderer, BulletStyle,
};
pub use table_render::{
    Table, TableRow, TableCell, TableSection, TableSectionData, TableRenderer, CellAlign,
};
pub use css_grid::{
    GridTrackSize, GridTemplate, GridAutoFlow, GridPlacement, GridItem, CssGrid,
};
pub use css_containment::{
    ContainmentType, ContainingBlock, ContainmentOptimizer, ContainmentStats,
};
