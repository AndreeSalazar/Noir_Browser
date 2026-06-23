//! UI Module - User interface components
//!
//! - click_feedback: Highlights, cursor types, focus rings

#![allow(dead_code)]

pub mod click_feedback;

pub use click_feedback::{
    CursorType, InteractiveRole, InteractiveBox, FocusRing,
    ClickEffect, ClickFeedback, InteractionState,
};
