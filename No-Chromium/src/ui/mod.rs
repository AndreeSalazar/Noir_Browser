//! UI Module - User interface components
//!
//! - click_feedback: Highlights, cursor types, focus rings
//! - element_highlight: Resalta elementos interactivos

#![allow(dead_code)]

pub mod click_feedback;
pub mod element_highlight;

pub use click_feedback::{
    CursorType, InteractiveRole, InteractiveBox, FocusRing,
    ClickEffect, ClickFeedback, InteractionState,
};
pub use element_highlight::{
    HighlightCategory, Highlight, HighlightMode, ElementHighlighter,
};
