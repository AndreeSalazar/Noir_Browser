use crate::browser::history::HistoryStore;
use crate::browser::page::{render_page, PageDocument, RenderBox};
use crate::parsers::css_engine::ComputedStyle;
use crate::render::text::{RasterizedAtlas, TextRasterizationOptions};

#[derive(Debug, Clone)]
pub struct LinkHitbox {
    pub href: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub is_input: bool,
    pub is_submit: bool,
    pub fragment_idx: usize,
}

#[derive(Debug, Clone)]
pub struct Tab {
    pub url: String,
    pub history: Vec<String>,
    pub history_index: usize,
    pub document: Option<PageDocument>,
    pub layout_boxes: Vec<RenderBox>,
    pub link_hitboxes: Vec<LinkHitbox>,
    pub scroll_offset: f32,
    pub content_height: f32,
    pub focused_input_idx: Option<usize>,
}

impl Tab {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            history: vec![url.to_string()],
            history_index: 0,
            document: None,
            layout_boxes: Vec::new(),
            link_hitboxes: Vec::new(),
            scroll_offset: 0.0,
            content_height: 0.0,
            focused_input_idx: None,
        }
    }
}

pub struct BrowserState {
    pub tabs: Vec<Tab>,
    pub active_tab_index: usize,
    pub history_store: HistoryStore,
    style: ComputedStyle,
}

impl BrowserState {
    pub fn new(initial_url: &str) -> Self {
        let mut style = ComputedStyle::default();
        style.background_color = Some("#1a1a1a".to_string());
        style.width = Some("100%".to_string());
        style.height = Some("100%".to_string());

        let mut history_store = HistoryStore::load();
        history_store.record_visit(initial_url);

        let initial_tab = Tab::new(initial_url);

        Self {
            tabs: vec![initial_tab],
            active_tab_index: 0,
            history_store,
            style,
        }
    }

    pub fn current_tab(&self) -> &Tab {
        &self.tabs[self.active_tab_index]
    }

    pub fn current_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab_index]
    }

    pub fn current_url(&self) -> &str {
        &self.current_tab().url
    }

    pub fn set_pending_url(&mut self, url: &str) {
        let tab = self.current_tab_mut();
        tab.url = url.to_string();
        tab.document = None;
        tab.scroll_offset = 0.0;
        tab.content_height = 0.0;
        tab.link_hitboxes.clear();
        tab.layout_boxes.clear();
        tab.focused_input_idx = None;
    }

    pub fn navigate_new(&mut self, url: &str) {
        let tab = self.current_tab_mut();
        if tab.history.get(tab.history_index).map(String::as_str) != Some(url) {
            tab.history.truncate(tab.history_index + 1);
            tab.history.push(url.to_string());
            tab.history_index = tab.history.len() - 1;
        }
        self.set_pending_url(url);
    }

    pub fn reload(&mut self) -> String {
        let url = self.current_url().to_string();
        self.set_pending_url(&url);
        url
    }

    pub fn go_back(&mut self) -> Option<String> {
        let tab = self.current_tab_mut();
        if tab.history_index == 0 {
            return None;
        }

        tab.history_index -= 1;
        let url = tab.history[tab.history_index].clone();
        self.set_pending_url(&url);
        Some(url)
    }

    pub fn go_forward(&mut self) -> Option<String> {
        let tab = self.current_tab_mut();
        if tab.history_index + 1 >= tab.history.len() {
            return None;
        }

        tab.history_index += 1;
        let url = tab.history[tab.history_index].clone();
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
        if url != self.current_url() {
            return None;
        }

        self.style = document.computed_style();
        let tab_url = {
            let tab = self.current_tab_mut();
            tab.document = Some(document);
            if let Some(summary) = tab.document.as_ref().and_then(PageDocument::media_summary) {
                println!("[Media] {}", summary);
            }
            tab.scroll_offset = 0.0;
            tab.url.clone()
        };
        self.history_store.record_visit(&tab_url);
        Some(self.render_current_page(text_options, viewport_width, viewport_height))
    }

    pub fn scroll_by(
        &mut self,
        delta_y: f32,
        text_options: TextRasterizationOptions,
        viewport_width: f32,
        viewport_height: f32,
    ) -> Option<RasterizedAtlas> {
        let tab = self.current_tab_mut();
        let max_scroll = (tab.content_height - viewport_height).max(0.0);
        let previous = tab.scroll_offset;
        tab.scroll_offset = (tab.scroll_offset + delta_y).clamp(0.0, max_scroll);

        if (tab.scroll_offset - previous).abs() < f32::EPSILON {
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
        let tab = self.current_tab_mut();
        tab.document.as_ref()?;
        let max_scroll = (tab.content_height - viewport_height).max(0.0);
        tab.scroll_offset = tab.scroll_offset.clamp(0.0, max_scroll);
        Some(self.render_current_page(text_options, viewport_width, viewport_height))
    }

    pub fn rerender_with_address(
        &mut self,
        address_text: &str,
        text_options: TextRasterizationOptions,
        viewport_width: f32,
        viewport_height: f32,
    ) -> Option<RasterizedAtlas> {
        let tabs_info: Vec<(String, bool)> = self.tabs.iter().enumerate().map(|(i, tab)| {
            (tab.url.clone(), i == self.active_tab_index)
        }).collect();
        let active_tab_index = self.active_tab_index;
        let tab = &mut self.tabs[active_tab_index];
        let document = tab.document.as_ref()?;
        let rendered = render_page(
            address_text,
            document,
            &mut tab.link_hitboxes,
            text_options,
            viewport_width,
            viewport_height,
            tab.scroll_offset,
            &tabs_info,
            tab.focused_input_idx,
        );
        tab.content_height = rendered.content_height;
        tab.layout_boxes = rendered.boxes;
        Some(rendered.atlas)
    }


    pub fn style(&self) -> &ComputedStyle {
        &self.style
    }

    pub fn layout_boxes(&self) -> &[RenderBox] {
        &self.current_tab().layout_boxes
    }

    pub fn open_tab(&mut self, url: &str) {
        let new_tab = Tab::new(url);
        self.tabs.push(new_tab);
        self.active_tab_index = self.tabs.len() - 1;
        self.style = ComputedStyle::default();
    }

    pub fn close_tab(&mut self, index: usize) -> bool {
        if self.tabs.len() <= 1 {
            self.tabs[0] = Tab::new("https://example.com");
            self.active_tab_index = 0;
            self.style = ComputedStyle::default();
            return true;
        }

        self.tabs.remove(index);
        if self.active_tab_index >= self.tabs.len() {
            self.active_tab_index = self.tabs.len() - 1;
        }
        self.switch_tab(self.active_tab_index);
        true
    }

    pub fn switch_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_tab_index = index;
            if let Some(doc) = &self.tabs[index].document {
                self.style = doc.computed_style();
            } else {
                let mut style = ComputedStyle::default();
                style.background_color = Some("#1a1a1a".to_string());
                style.width = Some("100%".to_string());
                style.height = Some("100%".to_string());
                self.style = style;
            }
        }
    }

    fn render_current_page(
        &mut self,
        text_options: TextRasterizationOptions,
        viewport_width: f32,
        viewport_height: f32,
    ) -> RasterizedAtlas {
        let tabs_info: Vec<(String, bool)> = self.tabs.iter().enumerate().map(|(i, tab)| {
            (tab.url.clone(), i == self.active_tab_index)
        }).collect();
        let active_url = self.current_url().to_string();
        let active_tab_index = self.active_tab_index;
        let tab = &mut self.tabs[active_tab_index];
        let document = tab
            .document
            .as_ref()
            .expect("Browser document should be loaded before rendering");
        let rendered = render_page(
            &active_url,
            document,
            &mut tab.link_hitboxes,
            text_options,
            viewport_width,
            viewport_height,
            tab.scroll_offset,
            &tabs_info,
            tab.focused_input_idx,
        );
        tab.content_height = rendered.content_height;
        tab.layout_boxes = rendered.boxes;

        let max_scroll = (tab.content_height - viewport_height).max(0.0);
        if tab.scroll_offset > max_scroll {
            tab.scroll_offset = max_scroll;
            let rendered = render_page(
                &active_url,
                document,
                &mut tab.link_hitboxes,
                text_options,
                viewport_width,
                viewport_height,
                tab.scroll_offset,
                &tabs_info,
                tab.focused_input_idx,
            );
            let tab = &mut self.tabs[active_tab_index];
            tab.content_height = rendered.content_height;
            tab.layout_boxes = rendered.boxes;
            rendered.atlas
        } else {
            rendered.atlas
        }
    }

    pub fn handle_page_char(&mut self, ch: char) -> bool {
        let tab = self.current_tab_mut();
        if let Some(focused_idx) = tab.focused_input_idx {
            if let Some(doc) = tab.document.as_mut() {
                if let Some(mut current_val) = doc.get_input_value(focused_idx) {
                    current_val.push(ch);
                    doc.set_input_value(focused_idx, current_val);
                    return true;
                }
            }
        }
        false
    }

    pub fn handle_page_backspace(&mut self) -> bool {
        let tab = self.current_tab_mut();
        if let Some(focused_idx) = tab.focused_input_idx {
            if let Some(doc) = tab.document.as_mut() {
                if let Some(mut current_val) = doc.get_input_value(focused_idx) {
                    current_val.pop();
                    doc.set_input_value(focused_idx, current_val);
                    return true;
                }
            }
        }
        false
    }

    pub fn handle_page_return(&mut self) -> Option<String> {
        let tab = self.current_tab_mut();
        if let Some(focused_idx) = tab.focused_input_idx {
            // Submit form for the focused input
            tab.focused_input_idx = None;
            return self.submit_form_at(focused_idx);
        }
        None
    }

    pub fn handle_page_click(&mut self, x: f32, y: f32) -> PageClickResult {
        let tab = self.current_tab_mut();
        
        let mut hit = None;
        for link in &tab.link_hitboxes {
            if x >= link.x && x <= link.x + link.w && y >= link.y && y <= link.y + link.h {
                hit = Some(link.clone());
                break;
            }
        }

        if let Some(link) = hit {
            if link.is_input {
                tab.focused_input_idx = Some(link.fragment_idx);
                PageClickResult::InputFocused
            } else if link.is_submit {
                tab.focused_input_idx = None;
                if let Some(submit_url) = self.submit_form_at(link.fragment_idx) {
                    PageClickResult::Submit(submit_url)
                } else {
                    PageClickResult::None
                }
            } else {
                tab.focused_input_idx = None;
                PageClickResult::Navigate(link.href.clone())
            }
        } else {
            tab.focused_input_idx = None;
            PageClickResult::None
        }
    }

    pub fn submit_form_at(&self, element_idx: usize) -> Option<String> {
        let tab = self.current_tab();
        let doc = tab.document.as_ref()?;
        
        let mut form_action = None;
        if let Some(super::page::LayoutFragment::Text(frag)) = doc.fragments.get(element_idx) {
            form_action = frag.form_action.clone();
        }

        let action_url_str = form_action?;
        let mut params = Vec::new();

        for frag in &doc.fragments {
            if let super::page::LayoutFragment::Text(f) = frag {
                if f.is_input && f.form_action.as_ref() == Some(&action_url_str) && !f.input_name.is_empty() {
                    params.push((f.input_name.clone(), f.input_value.clone()));
                }
            }
        }

        if let Ok(mut url) = url::Url::parse(&action_url_str) {
            let mut query = url.query_pairs().into_owned().collect::<Vec<_>>();
            for (k, v) in params {
                query.push((k, v));
            }
            url.set_query(None);
            {
                let mut serializer = url.query_pairs_mut();
                for (k, v) in query {
                    serializer.append_pair(&k, &v);
                }
            }
            Some(url.to_string())
        } else {
            Some(action_url_str)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PageClickResult {
    Navigate(String),
    InputFocused,
    Submit(String),
    None,
}
