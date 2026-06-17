use std::rc::Rc;
use winit::window::Window;

use super::theme::*;

pub struct TabState {
    pub title: String,
    pub url: String,
}

impl Default for TabState {
    fn default() -> Self {
        Self {
            title: "New Tab".into(),
            url: String::new(),
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
        }
    }

    pub fn navigate(&mut self, url: String) {
        self.tabs[self.active_tab].url = url.clone();
        self.tabs[self.active_tab].title = url;
        self.url_focused = false;
    }

    pub fn go_home(&mut self) {
        self.url_bar.clear();
        self.url_cursor = 0;
        self.tabs[self.active_tab].url.clear();
        self.tabs[self.active_tab].title = "New Tab".into();
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
