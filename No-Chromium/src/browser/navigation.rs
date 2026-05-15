use crate::browser::page::{load_page_document, render_page, PageDocument};
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
    document: Option<PageDocument>,
    scroll_offset: f32,
    content_height: f32,
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
            document: None,
            scroll_offset: 0.0,
            content_height: 0.0,
        }
    }

    pub fn load_current_page(
        &mut self,
        text_options: TextRasterizationOptions,
        viewport_width: f32,
        viewport_height: f32,
    ) -> RasterizedAtlas {
        if self.document.is_none() {
            self.document = Some(load_page_document(&self.current_url));
            self.scroll_offset = 0.0;
        }

        self.render_current_page(text_options, viewport_width, viewport_height)
    }

    pub fn navigate_to(
        &mut self,
        url: &str,
        text_options: TextRasterizationOptions,
        viewport_width: f32,
        viewport_height: f32,
    ) -> RasterizedAtlas {
        self.current_url = url.to_string();
        self.document = Some(load_page_document(&self.current_url));
        self.scroll_offset = 0.0;
        self.render_current_page(text_options, viewport_width, viewport_height)
    }

    pub fn scroll_by(
        &mut self,
        delta_y: f32,
        text_options: TextRasterizationOptions,
        viewport_width: f32,
        viewport_height: f32,
    ) -> Option<RasterizedAtlas> {
        let max_scroll = (self.content_height - viewport_height).max(0.0);
        let previous = self.scroll_offset;
        self.scroll_offset = (self.scroll_offset + delta_y).clamp(0.0, max_scroll);

        if (self.scroll_offset - previous).abs() < f32::EPSILON {
            return None;
        }

        Some(self.render_current_page(text_options, viewport_width, viewport_height))
    }

    pub fn rerender_current_page(
        &mut self,
        text_options: TextRasterizationOptions,
        viewport_width: f32,
        viewport_height: f32,
    ) -> RasterizedAtlas {
        let max_scroll = (self.content_height - viewport_height).max(0.0);
        self.scroll_offset = self.scroll_offset.clamp(0.0, max_scroll);
        self.render_current_page(text_options, viewport_width, viewport_height)
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

    fn render_current_page(
        &mut self,
        text_options: TextRasterizationOptions,
        viewport_width: f32,
        viewport_height: f32,
    ) -> RasterizedAtlas {
        let document = self
            .document
            .as_ref()
            .expect("Browser document should be loaded before rendering");
        let rendered = render_page(
            &self.current_url,
            document,
            &mut self.link_hitboxes,
            text_options,
            viewport_width,
            viewport_height,
            self.scroll_offset,
        );
        self.content_height = rendered.content_height;

        let max_scroll = (self.content_height - viewport_height).max(0.0);
        if self.scroll_offset > max_scroll {
            self.scroll_offset = max_scroll;
            let rendered = render_page(
                &self.current_url,
                document,
                &mut self.link_hitboxes,
                text_options,
                viewport_width,
                viewport_height,
                self.scroll_offset,
            );
            self.content_height = rendered.content_height;
            rendered.atlas
        } else {
            rendered.atlas
        }
    }
}
