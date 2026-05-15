use crate::browser::history::HistoryStore;
use crate::browser::page::{render_page, PageDocument};
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
    history: Vec<String>,
    history_index: usize,
    history_store: HistoryStore,
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

        let mut history_store = HistoryStore::load();
        history_store.record_visit(initial_url);

        Self {
            current_url: initial_url.to_string(),
            history: vec![initial_url.to_string()],
            history_index: 0,
            history_store,
            link_hitboxes: Vec::new(),
            style,
            document: None,
            scroll_offset: 0.0,
            content_height: 0.0,
        }
    }

    pub fn current_url(&self) -> &str {
        &self.current_url
    }

    pub fn set_pending_url(&mut self, url: &str) {
        self.current_url = url.to_string();
        self.document = None;
        self.scroll_offset = 0.0;
        self.content_height = 0.0;
        self.link_hitboxes.clear();
    }

    pub fn navigate_new(&mut self, url: &str) {
        if self.history.get(self.history_index).map(String::as_str) != Some(url) {
            self.history.truncate(self.history_index + 1);
            self.history.push(url.to_string());
            self.history_index = self.history.len() - 1;
        }
        self.set_pending_url(url);
    }

    pub fn reload(&mut self) -> String {
        let url = self.current_url.clone();
        self.set_pending_url(&url);
        url
    }

    pub fn go_back(&mut self) -> Option<String> {
        if self.history_index == 0 {
            return None;
        }

        self.history_index -= 1;
        let url = self.history[self.history_index].clone();
        self.set_pending_url(&url);
        Some(url)
    }

    pub fn go_forward(&mut self) -> Option<String> {
        if self.history_index + 1 >= self.history.len() {
            return None;
        }

        self.history_index += 1;
        let url = self.history[self.history_index].clone();
        self.set_pending_url(&url);
        Some(url)
    }

    pub fn accept_loaded_document(
        &mut self,
        url: String,
        document: PageDocument,
        text_options: TextRasterizationOptions,
        viewport_width: f32,
        viewport_height: f32,
    ) -> Option<RasterizedAtlas> {
        if url != self.current_url {
            return None;
        }

        self.document = Some(document);
        if let Some(summary) = self.document.as_ref().and_then(PageDocument::media_summary) {
            println!("[Media] {}", summary);
        }
        self.history_store.record_visit(&self.current_url);
        self.scroll_offset = 0.0;
        Some(self.render_current_page(text_options, viewport_width, viewport_height))
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
    ) -> Option<RasterizedAtlas> {
        self.document.as_ref()?;
        let max_scroll = (self.content_height - viewport_height).max(0.0);
        self.scroll_offset = self.scroll_offset.clamp(0.0, max_scroll);
        Some(self.render_current_page(text_options, viewport_width, viewport_height))
    }

    pub fn rerender_with_address(
        &mut self,
        address_text: &str,
        text_options: TextRasterizationOptions,
        viewport_width: f32,
        viewport_height: f32,
    ) -> Option<RasterizedAtlas> {
        let document = self.document.as_ref()?;
        let rendered = render_page(
            address_text,
            document,
            &mut self.link_hitboxes,
            text_options,
            viewport_width,
            viewport_height,
            self.scroll_offset,
        );
        self.content_height = rendered.content_height;
        Some(rendered.atlas)
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
