// Text rendering stubs for Fase 0
// Stub implemented to resolve import errors

/// Options for text rasterization
#[derive(Clone, Debug, Default)]
pub struct TextRasterizationOptions {
    pub font_size: f32,
    pub color: [u8; 4], // RGBA
    pub msdf_enabled: bool,
}

/// Rasterized text atlas for GPU upload
#[derive(Clone, Debug)]
pub struct RasterizedAtlas {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, // RGBA pixels
    pub content_height: f32,
}

impl RasterizedAtlas {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0u8; (width * height * 4) as usize],
            content_height: 0.0,
        }
    }
}
