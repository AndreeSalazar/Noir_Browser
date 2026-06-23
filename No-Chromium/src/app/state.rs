//! Tab State - Estado de una pestaña
//!
//! Solo contiene el estado de las pestañas.
//! El estado global de la aplicación está en `context.rs`.

use crate::app::scroll::ScrollState;
use crate::js_engine_v3::Interpreter;
use crate::parsers::layout::LayoutItem;
use crate::parsers::page_document::PageDocument;

/// Estado de una pestaña individual
pub struct TabState {
    pub title: String,
    pub url: String,
    pub page: Option<PageDocument>,
    pub layout_blocks: Vec<LayoutItem>,
    /// FASE A4: Smooth scrolling state (con inercia y clamping)
    pub scroll: ScrollState,
    /// Legacy scroll_y (sincronizado con scroll.offset_y)
    pub scroll_y: f32,
    pub content_height: f32,
    pub js_engine: Interpreter,
    pub tab_id: u64,
}

impl Default for TabState {
    fn default() -> Self {
        Self {
            title: "New Tab".to_string(),
            url: String::new(),
            page: None,
            layout_blocks: Vec::new(),
            scroll: ScrollState::new(0.0),
            scroll_y: 0.0,
            content_height: 0.0,
            js_engine: Interpreter::new(),
            tab_id: 0,
        }
    }
}
