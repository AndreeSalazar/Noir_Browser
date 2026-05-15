use crate::parsers::css_engine::ComputedStyle;
use crate::render::text::{RasterizedAtlas, TextRasterizationOptions};

#[derive(Debug, Clone)]
pub struct LinkHitbox {
    pub url: String,
    pub y_min: f32,
    pub y_max: f32,
}

pub struct BrowserState {
    current_url: String,
    link_hitboxes: Vec<LinkHitbox>,
    style: ComputedStyle,
}

impl BrowserState {
    pub fn new(initial_url: &str) -> Self {
        let mut style = ComputedStyle::default();
        style.background_color = Some("#1a1a1a".to_string());
        style.width = Some("100%".to_string());
        style.height = Some("100%".to_string());

        Self {
            current_url: initial_url.to_string(),
            link_hitboxes: Vec::new(),
            style,
        }
    }

    pub fn load_current_page(&mut self, text_options: TextRasterizationOptions) -> RasterizedAtlas {
        crate::browser::page::load_page(&self.current_url, &mut self.link_hitboxes, text_options)
    }

    pub fn navigate_to(&mut self, url: &str, text_options: TextRasterizationOptions) -> RasterizedAtlas {
        self.current_url = url.to_string();
        self.load_current_page(text_options)
    }

    pub fn link_at_y(&self, y: f32) -> Option<String> {
        self.link_hitboxes
            .iter()
            .find(|link| y >= link.y_min && y <= link.y_max)
            .map(|link| link.url.clone())
    }

    pub fn style(&self) -> &ComputedStyle {
        &self.style
    }
}
