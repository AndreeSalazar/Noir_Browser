use std::rc::Rc;
use std::sync::{Arc, Mutex};
use winit::window::Window;

use super::theme::*;
use crate::parsers::page_document::PageDocument;
use crate::parsers::layout::LayoutBlock;

pub struct TabState {
    pub title: String,
    pub url: String,
    pub page: Option<PageDocument>,
    pub layout_blocks: Vec<LayoutBlock>,
    pub scroll_y: f32,
    pub content_height: f32,
}

impl Default for TabState {
    fn default() -> Self {
        Self {
            title: "New Tab".into(),
            url: String::new(),
            page: None,
            layout_blocks: Vec::new(),
            scroll_y: 0.0,
            content_height: 0.0,
        }
    }
}

pub struct NoirApp {
    pub window: Option<Rc<Window>>,
    pub surface: Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>,
    pub width: u32,
    pub height: u32,
    pub url_bar: String,
    pub url_cursor: usize,
    pub url_focused: bool,
    pub tabs: Vec<TabState>,
    pub active_tab: usize,
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub is_maximized: bool,
    pub should_close: bool,
    pub fetching: bool,
    pub fetch_result: Option<Arc<Mutex<Option<String>>>>,
}

impl NoirApp {
    pub fn new() -> Self {
        let mut tabs = Vec::new();
        tabs.push(TabState::default());
        Self {
            window: None,
            surface: None,
            width: 1280,
            height: 720,
            url_bar: String::new(),
            url_cursor: 0,
            url_focused: false,
            tabs,
            active_tab: 0,
            mouse_x: 0.0,
            mouse_y: 0.0,
            is_maximized: false,
            should_close: false,
            fetching: false,
            fetch_result: None,
        }
    }

    pub fn navigate(&mut self, url: String) {
        self.tabs[self.active_tab].url = url.clone();
        self.tabs[self.active_tab].title = url;
        self.tabs[self.active_tab].page = None;
        self.tabs[self.active_tab].layout_blocks.clear();
        self.tabs[self.active_tab].scroll_y = 0.0;
        self.tabs[self.active_tab].content_height = 0.0;
        self.url_focused = false;
        self.fetching = true;
        self.fetch_result = None;
    }

    pub fn go_home(&mut self) {
        self.url_bar.clear();
        self.url_cursor = 0;
        self.tabs[self.active_tab].url.clear();
        self.tabs[self.active_tab].title = "New Tab".into();
        self.tabs[self.active_tab].page = None;
        self.tabs[self.active_tab].layout_blocks.clear();
        self.tabs[self.active_tab].scroll_y = 0.0;
        self.tabs[self.active_tab].content_height = 0.0;
        self.fetching = false;
        self.fetch_result = None;
    }

    pub fn new_tab(&mut self) {
        use super::config::AppConfig;
        let max = AppConfig::default().max_tabs as usize;
        if self.tabs.len() < max {
            self.tabs.push(TabState::default());
            self.active_tab = self.tabs.len() - 1;
            self.url_bar.clear();
            self.url_cursor = 0;
        }
    }

    pub fn switch_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_tab = index;
            self.url_bar = self.tabs[index].url.clone();
            self.url_cursor = self.url_bar.len();
            self.url_focused = false;
        }
    }

    pub fn scroll(&mut self, delta: f32) {
        let tab = &mut self.tabs[self.active_tab];
        tab.scroll_y = (tab.scroll_y - delta * 40.0).max(0.0);
        let max_scroll = (tab.content_height - self.height as f32 + TOOLBAR_HEIGHT as f32).max(0.0);
        tab.scroll_y = tab.scroll_y.min(max_scroll);
    }

    pub fn display_url(&self) -> String {
        if self.url_bar.is_empty() && !self.url_focused {
            "Search or enter URL...".into()
        } else {
            self.url_bar.clone()
        }
    }

    pub fn url_text_color(&self) -> u32 {
        if self.url_bar.is_empty() && !self.url_focused {
            TEXT_DIM
        } else {
            TEXT_WHITE
        }
    }
}
