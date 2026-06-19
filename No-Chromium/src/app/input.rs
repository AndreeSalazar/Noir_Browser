use super::state::NoirApp;
use super::theme::*;
use crate::network::fetch::HttpFetcher;
use crate::parsers::layout::hit_test_link;
use std::sync::{Arc, Mutex};
use winit::keyboard::{Key, NamedKey};

impl NoirApp {
    pub fn handle_click(&mut self) {
        let mx = self.mouse_x;
        let my = self.mouse_y;
        let w = self.width as f32;

        if my <= TITLE_BAR_HEIGHT as f32 {
            self.handle_title_bar_click(mx, w);
            return;
        }

        if my >= TITLE_BAR_HEIGHT as f32 && my <= (TITLE_BAR_HEIGHT + TAB_BAR_HEIGHT) as f32 {
            self.handle_tab_bar_click(mx);
            return;
        }

        let nav_y = (TITLE_BAR_HEIGHT + TAB_BAR_HEIGHT) as f32;
        let btn_h = 32.0f32;
        let btn_y = nav_y + (NAV_BAR_HEIGHT as f32 - btn_h) / 2.0;
        let btn_bottom = btn_y + btn_h;

        if my >= btn_y && my <= btn_bottom {
            self.handle_nav_bar_click(mx, w);
            return;
        }

        let content_top = TOOLBAR_HEIGHT as f32;
        if my > content_top {
            self.handle_content_click(mx, my);
            return;
        }

        self.url_focused = false;
    }

    fn handle_title_bar_click(&mut self, mx: f32, w: f32) {
        let ctrl_w = 46.0f32;

        let close_x = w - ctrl_w;
        if mx >= close_x {
            self.should_close = true;
            return;
        }

        let max_x = w - ctrl_w * 2.0;
        if mx >= max_x && mx < close_x {
            if let Some(window) = &self.window {
                self.is_maximized = !self.is_maximized;
                window.set_maximized(self.is_maximized);
            }
            return;
        }

        let min_x = w - ctrl_w * 3.0;
        if mx >= min_x && mx < max_x {
            if let Some(window) = &self.window {
                window.set_minimized(true);
            }
            return;
        }

        if mx < min_x {
            if let Some(window) = &self.window {
                let _ = window.drag_window();
            }
        }
    }

    fn handle_tab_bar_click(&mut self, mx: f32) {
        let mut tx = 6.0f32;
        for i in 0..self.tabs.len() {
            let tab_w = TAB_WIDTH as f32;
            if mx >= tx && mx <= tx + tab_w {
                self.switch_tab(i);
                return;
            }
            tx += tab_w + TAB_SPACING as f32;
        }
        if mx >= tx && mx <= tx + 28.0 {
            self.new_tab();
        }
    }

    fn handle_nav_bar_click(&mut self, mx: f32, w: f32) {
        let mut bx = NAV_START_X as f32;

        if self.click_back_button(mx, bx) { return; }
        bx += NAV_BTN_SIZE as f32 + NAV_BTN_SPACING as f32;

        if self.click_forward_button(mx, bx) { return; }
        bx += NAV_BTN_SIZE as f32 + NAV_BTN_SPACING as f32;

        if self.click_reload_button(mx, bx) { return; }
        bx += NAV_BTN_SIZE as f32 + NAV_BTN_SPACING as f32;

        if self.click_home_button(mx, bx) { return; }
        bx += NAV_BTN_SIZE as f32 + 12.0;

        if mx >= bx && mx <= w {
            self.url_focused = true;
            self.url_cursor = self.url_bar.len();
        }
    }

    fn click_back_button(&mut self, mx: f32, bx: f32) -> bool {
        if mx >= bx && mx <= bx + NAV_BTN_SIZE as f32 {
            if self.history_index > 0 {
                self.history_index -= 1;
                let url = self.history[self.history_index].clone();
                self.url_bar = url.clone();
                self.url_cursor = self.url_bar.len();
                self.navigate(url);
            }
            return true;
        }
        false
    }

    fn click_forward_button(&mut self, mx: f32, bx: f32) -> bool {
        if mx >= bx && mx <= bx + NAV_BTN_SIZE as f32 {
            if self.history_index < self.history.len().saturating_sub(1) {
                self.history_index += 1;
                let url = self.history[self.history_index].clone();
                self.url_bar = url.clone();
                self.url_cursor = self.url_bar.len();
                self.navigate(url);
            }
            return true;
        }
        false
    }

    fn click_reload_button(&mut self, mx: f32, bx: f32) -> bool {
        if mx >= bx && mx <= bx + NAV_BTN_SIZE as f32 {
            let url = self.tabs[self.active_tab].url.clone();
            if !url.is_empty() {
                self.navigate(url);
            }
            return true;
        }
        false
    }

    fn click_home_button(&mut self, mx: f32, bx: f32) -> bool {
        if mx >= bx && mx <= bx + NAV_BTN_SIZE as f32 {
            self.go_home();
            return true;
        }
        false
    }

    fn handle_content_click(&mut self, mx: f32, my: f32) {
        let layout_blocks = self.tabs[self.active_tab].layout_blocks.clone();
        let scroll_y = self.tabs[self.active_tab].scroll_y;
        if let Some(href) = hit_test_link(&layout_blocks, mx, my, scroll_y) {
            tracing::info!("Link clicked: {}", href);
            self.url_bar = href.clone();
            self.url_cursor = self.url_bar.len();
            self.navigate(href);
            self.window.as_ref().unwrap().request_redraw();
        }
    }

    pub fn handle_key(&mut self, key: &Key, ctrl: bool) {
        if ctrl {
            self.handle_ctrl_key(key);
            return;
        }

        if !self.url_focused {
            self.handle_unfocused_key(key);
            return;
        }

        self.handle_url_input_key(key);
    }

    fn handle_ctrl_key(&mut self, key: &Key) {
        match key {
            Key::Character(c) => match c.as_str() {
                "t" | "T" => { self.new_tab(); }
                "w" | "W" => { self.close_current_tab(); }
                "l" | "L" => { self.focus_url_bar(); }
                "r" | "R" => { self.reload_current_tab(); }
                "d" | "D" => { self.new_tab(); }
                _ => {}
            },
            Key::Named(NamedKey::Tab) => { self.cycle_tab(); }
            _ => {}
        }
    }

    fn close_current_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active_tab);
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len().saturating_sub(1);
            }
            self.url_bar = self.tabs[self.active_tab].url.clone();
            self.url_cursor = self.url_bar.len();
        }
    }

    fn focus_url_bar(&mut self) {
        self.url_focused = true;
        self.url_cursor = self.url_bar.len();
    }

    fn reload_current_tab(&mut self) {
        let url = self.tabs[self.active_tab].url.clone();
        if !url.is_empty() {
            self.navigate(url);
        }
    }

    fn cycle_tab(&mut self) {
        if self.tabs.len() > 1 {
            let next = (self.active_tab + 1) % self.tabs.len();
            self.switch_tab(next);
        }
    }

    fn handle_unfocused_key(&mut self, key: &Key) {
        match key {
            Key::Named(NamedKey::F5) => { self.reload_current_tab(); }
            Key::Named(NamedKey::F11) => { self.toggle_maximized(); }
            _ => {}
        }
    }

    fn toggle_maximized(&mut self) {
        self.is_maximized = !self.is_maximized;
        if let Some(window) = &self.window {
            window.set_maximized(self.is_maximized);
        }
    }

    fn handle_url_input_key(&mut self, key: &Key) {
        match key {
            Key::Named(NamedKey::Backspace) => { self.url_backspace(); }
            Key::Named(NamedKey::Delete) => { self.url_delete(); }
            Key::Named(NamedKey::ArrowLeft) => { self.url_cursor_left(); }
            Key::Named(NamedKey::ArrowRight) => { self.url_cursor_right(); }
            Key::Named(NamedKey::Home) => { self.url_cursor = 0; }
            Key::Named(NamedKey::End) => { self.url_cursor = self.url_bar.len(); }
            Key::Named(NamedKey::Enter) => { self.url_submit(); }
            Key::Named(NamedKey::Escape) => { self.url_focused = false; }
            Key::Character(c) => { self.url_insert(c.as_str()); }
            _ => {}
        }
    }

    fn url_backspace(&mut self) {
        if self.url_cursor > 0 {
            self.url_cursor -= 1;
            self.url_bar.remove(self.url_cursor);
        }
    }

    fn url_delete(&mut self) {
        if self.url_cursor < self.url_bar.len() {
            self.url_bar.remove(self.url_cursor);
        }
    }

    fn url_cursor_left(&mut self) {
        self.url_cursor = self.url_cursor.saturating_sub(1);
    }

    fn url_cursor_right(&mut self) {
        if self.url_cursor < self.url_bar.len() {
            self.url_cursor += 1;
        }
    }

    fn url_submit(&mut self) {
        if !self.url_bar.is_empty() {
            let url = self.resolve_url();
            self.navigate(url.clone());
            tracing::info!("Navigating to: {}", url);

            let fetcher = HttpFetcher::new();
            let url_clone = url.clone();
            let result_holder: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
            let result_clone = result_holder.clone();

            tokio::spawn(async move {
                match fetcher.get(&url_clone).await {
                    Ok(result) => {
                        tracing::info!("Fetched {} ({} bytes, status {})", url_clone, result.body.len(), result.status);
                        if let Ok(mut guard) = result_clone.lock() {
                            *guard = Some(result.body);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Fetch failed: {}", e);
                        if let Ok(mut guard) = result_clone.lock() {
                            *guard = Some(format!("<html><head><title>Error</title></head><body><h1>Failed to load</h1><p>{}</p><p>URL: {}</p></body></html>", e, url_clone));
                        }
                    }
                }
            });

            self.fetch_result = Some(result_holder);
        }
        self.url_focused = false;
    }

    fn url_insert(&mut self, ch: &str) {
        if !ch.is_empty() {
            for chr in ch.chars() {
                self.url_bar.insert(self.url_cursor, chr);
                self.url_cursor += 1;
            }
        }
    }
}
