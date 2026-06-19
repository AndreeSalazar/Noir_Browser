// Renderer: Módulo base para el proceso de renderizado (parser, layout, DOM).
// Stub implementado para Fase 0.

pub mod html_parser;
pub mod css_cascade;
pub mod layout_engine;
pub mod js_engine;
pub mod text;

// Re-exports para uso externo
pub use html_parser::PageDocument;
pub use layout_engine::RenderBox;
pub use text::{RasterizedAtlas, TextRasterizationOptions};

/// Resultado de renderizado de página
#[derive(Debug)]
pub struct RenderedPage {
    pub atlas: RasterizedAtlas,
    pub boxes: Vec<RenderBox>,
    pub content_height: f32,
}

/// Función stub para renderizar página
pub fn render_page(
    _url: &str,
    _doc: &PageDocument,
    _hitboxes: &mut Vec<crate::browser::navigation::LinkHitbox>,
    _text_opts: TextRasterizationOptions,
    _viewport_width: f32,
    _viewport_height: f32,
    _scroll_offset: f32,
    _tabs_info: &[(String, bool)],
    _focused_input: Option<usize>,
) -> RenderedPage {
    RenderedPage {
        atlas: RasterizedAtlas::new(1280, 720),
        boxes: Vec::new(),
        content_height: 720.0,
    }
}
