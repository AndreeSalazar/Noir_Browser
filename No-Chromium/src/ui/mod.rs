//! UI Module - User interface components
//!
//! - click_feedback: Highlights, cursor types, focus rings
//! - element_highlight: Resalta elementos interactivos
//! - markdown: Rendering de markdown
//! - card_layout: Card-style rendering
//! - list_render: Listas ordenadas/no ordenadas con bullets

#![allow(dead_code)]

pub mod click_feedback;
pub mod element_highlight;
pub mod markdown;
pub mod card_layout;
pub mod list_render;

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
