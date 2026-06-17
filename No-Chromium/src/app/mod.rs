pub mod config;
pub mod draw;
pub mod glyphs;
pub mod state;
pub mod theme;

use anyhow::Result;
use std::num::NonZeroU32;
use std::rc::Rc;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{WindowAttributes, WindowId};

pub use config::AppConfig;
use draw::{draw_rect, draw_text_noir};
use state::NoirApp;
use theme::*;

impl NoirApp {
    fn draw_frame(&mut self) {
        // Cache all state before mutable borrow
        let display_url = self.display_url();
        let url_color = self.url_text_color();
        let url_bar_empty = self.url_bar.is_empty();
        let url_focused = self.url_focused;
        let url_cursor = self.url_cursor;
        let active_tab = self.active_tab;
        let _tabs_len = self.tabs.len();
        let tab_titles: Vec<String> = self.tabs.iter().map(|t| {
            if t.title.len() > 24 {
                format!("{}...", &t.title[..21])
            } else {
                t.title.clone()
            }
        }).collect();
        let active_url = self.tabs[active_tab].url.clone();

        let (surface, window) = match (&mut self.surface, &self.window) {
            (Some(s), Some(w)) => (s, w),
            _ => return,
        };

        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        let mut buffer = surface.buffer_mut().unwrap();
        let buf = buffer.as_mut();
        let stride = width as usize;

        // Clear
        for pixel in buf.iter_mut() {
            *pixel = BG_CONTENT;
        }

        let w = width as i32;
        let h = height as i32;

        // ═══════════════ TITLE BAR ═══════════════
        draw_rect(buf, stride, 0, 0, w, TITLE_BAR_HEIGHT as i32, BG_TITLEBAR);
        draw_text_noir(buf, stride, w, 14, 9, "Noir Browser", TEXT_DIM, 0.9);

        // Window controls (right side, larger)
        let ctrl_y = 8;
        let ctrl_h = 16;
        let ctrl_w = 20;
        let gap = 2;
        let close_x = w - ctrl_w - 8;
        draw_rect(buf, stride, close_x, ctrl_y, ctrl_w, ctrl_h, CLOSE_RED);
        let min_x = close_x - ctrl_w - gap;
        draw_rect(buf, stride, min_x, ctrl_y, ctrl_w, ctrl_h, YELLOW);
        let max_x = min_x - ctrl_w - gap;
        draw_rect(buf, stride, max_x, ctrl_y, ctrl_w, ctrl_h, GREEN);

        // ═══════════════ TAB BAR ═══════════════
        let tab_y = TITLE_BAR_HEIGHT as i32;
        draw_rect(buf, stride, 0, tab_y, w, TAB_BAR_HEIGHT as i32, BG_TAB_BAR);

        let mut tx = 6i32;
        for (i, title) in tab_titles.iter().enumerate() {
            let tab_w = TAB_WIDTH.min(w - tx - 80);
            if tx + tab_w > w - 80 { break; }

            let ty = tab_y + 3;
            let th = TAB_BAR_HEIGHT as i32 - 6;
            let bg = if i == active_tab { BG_ADDRESS_BAR } else { BG_TAB_BAR };
            draw_rect(buf, stride, tx, ty, tab_w, th, bg);

            // Active tab indicator line
            if i == active_tab {
                draw_rect(buf, stride, tx, ty, tab_w, 2, ACCENT);
            }

            // Tab title
            let text_x = tx + 12;
            let text_y_pos = ty + (th / 2) - 4;
            draw_text_noir(buf, stride, w, text_x, text_y_pos, title, if i == active_tab { TEXT_WHITE } else { TEXT_DIM }, 0.8);

            // Close tab X (small)
            let close_tab_x = tx + tab_w - 18;
            draw_text_noir(buf, stride, w, close_tab_x, text_y_pos, "x", TEXT_DIM, 0.7);

            tx += tab_w + TAB_SPACING;
        }

        // New tab button (+)
        if tx + 30 < w {
            let plus_h = TAB_BAR_HEIGHT as i32 - 10;
            draw_rect(buf, stride, tx + 4, tab_y + 5, 24, plus_h, BTN_BG);
            draw_text_noir(buf, stride, w, tx + 11, tab_y + 9, "+", TEXT_DIM, 1.1);
        }

        // ═══════════════ NAV BAR ═══════════════
        let nav_y = (TITLE_BAR_HEIGHT + TAB_BAR_HEIGHT) as i32;
        draw_rect(buf, stride, 0, nav_y, w, NAV_BAR_HEIGHT as i32, BG_DARK);

        // Separator line between tabs and nav
        draw_rect(buf, stride, 0, nav_y, w, 1, SEPARATOR);

        let btn_h = 32i32;
        let btn_y_pos = nav_y + (NAV_BAR_HEIGHT as i32 - btn_h) / 2;

        // Nav buttons with proper spacing
        let mut bx = NAV_START_X;

        // Back
        draw_rect(buf, stride, bx, btn_y_pos, NAV_BTN_SIZE, btn_h, BTN_BG);
        draw_text_noir(buf, stride, w, bx + 11, btn_y_pos + 8, "<", TEXT_WHITE, 1.0);
        bx += NAV_BTN_SIZE + NAV_BTN_SPACING;

        // Forward
        draw_rect(buf, stride, bx, btn_y_pos, NAV_BTN_SIZE, btn_h, BTN_BG);
        draw_text_noir(buf, stride, w, bx + 11, btn_y_pos + 8, ">", TEXT_WHITE, 1.0);
        bx += NAV_BTN_SIZE + NAV_BTN_SPACING;

        // Reload
        draw_rect(buf, stride, bx, btn_y_pos, NAV_BTN_SIZE, btn_h, BTN_BG);
        draw_text_noir(buf, stride, w, bx + 11, btn_y_pos + 8, "R", TEXT_WHITE, 1.0);
        bx += NAV_BTN_SIZE + NAV_BTN_SPACING;

        // Home
        draw_rect(buf, stride, bx, btn_y_pos, NAV_BTN_SIZE, btn_h, BTN_BG);
        draw_text_noir(buf, stride, w, bx + 11, btn_y_pos + 8, "H", TEXT_WHITE, 1.0);
        bx += NAV_BTN_SIZE + 12;

        // Address bar
        let ab_w = w - bx - 16;
        if ab_w > 50 {
            let ab_bg = if url_focused { BG_ADDRESS_BAR_FOCUS } else { BG_ADDRESS_BAR };
            draw_rect(buf, stride, bx, btn_y_pos, ab_w, btn_h, ab_bg);

            // Lock icon (green padlock) on the right
            if !url_bar_empty {
                let lock_x = bx + ab_w - 28;
                draw_rect(buf, stride, lock_x, btn_y_pos + 9, 8, 12, GREEN);
                draw_rect(buf, stride, lock_x + 1, btn_y_pos + 7, 6, 3, GREEN);
            }

            // URL text
            let text_x = bx + 14;
            draw_text_noir(buf, stride, w, text_x, btn_y_pos + 10, &display_url, url_color, 0.9);

            // Cursor when focused
            if url_focused {
                let cursor_px = (url_cursor as i32) * 8 + text_x;
                draw_rect(buf, stride, cursor_px, btn_y_pos + 8, 1, btn_h - 16, TEXT_WHITE);
            }
        }

        // ═══════════════ CONTENT ═══════════════
        let content_y = TOOLBAR_HEIGHT as i32;
        let content_h = h - content_y;

        if active_url.is_empty() {
            // === NEW TAB PAGE ===
            let center_y = content_y + content_h / 2 - 60;

            // Logo
            draw_text_noir(buf, stride, w, w / 2 - 108, center_y, "NOIR", ACCENT, 3.5);
            draw_text_noir(buf, stride, w, w / 2 - 110, center_y + 55, "BROWSER", TEXT_DIM, 1.8);

            // Tagline
            draw_text_noir(
                buf, stride, w, w / 2 - 155, center_y + 95,
                "Ultra-fast  |  Private  |  Vulkan-powered", TEXT_PLACEHOLDER, 0.85,
            );

            // Quick links - centered, with labels below
            let link_y = center_y + 150;
            let links = [("Google", LINK_GOOGLE), ("GitHub", LINK_GITHUB), ("YouTube", LINK_YOUTUBE), ("Rust", LINK_RUST)];
            let total_w = links.len() as i32 * LINK_CARD_SIZE + (links.len() as i32 - 1) * LINK_CARD_SPACING;
            let start_x = w / 2 - total_w / 2;

            for (i, (name, color)) in links.iter().enumerate() {
                let lx = start_x + i as i32 * (LINK_CARD_SIZE + LINK_CARD_SPACING);

                // Card background
                draw_rect(buf, stride, lx, link_y, LINK_CARD_SIZE, LINK_CARD_SIZE, BG_LINK_CARD);

                // Color accent bar at top
                draw_rect(buf, stride, lx, link_y, LINK_CARD_SIZE, 3, *color);

                // Icon placeholder (colored circle-ish)
                let icon_size = 28;
                let icon_x = lx + (LINK_CARD_SIZE - icon_size) / 2;
                let icon_y = link_y + 22;
                draw_rect(buf, stride, icon_x, icon_y, icon_size, icon_size, *color);

                // Label below card
                let label_x = lx + (LINK_CARD_SIZE as i32 - name.len() as i32 * 7) / 2;
                draw_text_noir(buf, stride, w, label_x, link_y + LINK_CARD_SIZE + 12, name, TEXT_DIM, 0.85);
            }
        } else {
            // Loading page
            draw_text_noir(buf, stride, w, w / 2 - 80, content_y + 40, "Loading...", TEXT_DIM, 1.0);
            draw_text_noir(buf, stride, w, 30, content_y + 80, &active_url, TEXT_PLACEHOLDER, 0.8);
        }

        buffer.present().unwrap();
    }

    fn resolve_url(&self) -> String {
        if self.url_bar.starts_with("http://") || self.url_bar.starts_with("https://") {
            self.url_bar.clone()
        } else if self.url_bar.contains('.') && !self.url_bar.contains(' ') {
            format!("https://{}", self.url_bar)
        } else {
            format!("https://duckduckgo.com/?q={}", self.url_bar.replace(' ', "+"))
        }
    }
}

impl ApplicationHandler for NoirApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = WindowAttributes::default()
            .with_title("Noir Browser")
            .with_inner_size(LogicalSize::new(1280.0, 720.0))
            .with_min_inner_size(LogicalSize::new(800.0, 500.0));

        let window = Rc::new(event_loop.create_window(attrs).unwrap());
        let size = window.inner_size();
        self.width = size.width;
        self.height = size.height;

        let context = softbuffer::Context::new(Rc::clone(&window)).unwrap();
        let surface = softbuffer::Surface::new(&context, Rc::clone(&window)).unwrap();

        self.window = Some(window);
        self.surface = Some(surface);

        tracing::info!("Window created: {}x{}", self.width, self.height);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                tracing::info!("Close requested, shutting down...");
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                self.draw_frame();
            }

            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    self.width = size.width;
                    self.height = size.height;
                    if let Some(surface) = &mut self.surface {
                        surface.resize(
                            NonZeroU32::new(size.width).unwrap(),
                            NonZeroU32::new(size.height).unwrap(),
                        )
                        .unwrap();
                    }
                    self.window.as_ref().unwrap().request_redraw();
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_x = position.x as f32;
                self.mouse_y = position.y as f32;
            }

            WindowEvent::MouseInput { state, button: _, .. } => {
                if state != ElementState::Pressed {
                    return;
                }
                self.handle_click();
                self.window.as_ref().unwrap().request_redraw();
            }

            WindowEvent::KeyboardInput {
                event: KeyEvent { logical_key, state: ElementState::Pressed, .. },
                ..
            } => {
                self.handle_key(&logical_key);
                self.window.as_ref().unwrap().request_redraw();
            }

            _ => {}
        }
    }
}

impl NoirApp {
    fn handle_click(&mut self) {
        let mx = self.mouse_x;
        let my = self.mouse_y;

        let nav_y = (TITLE_BAR_HEIGHT + TAB_BAR_HEIGHT) as f32;
        let btn_h = 32.0f32;
        let btn_y = nav_y + (NAV_BAR_HEIGHT as f32 - btn_h) / 2.0;
        let btn_bottom = btn_y + btn_h;

        // Tab bar clicks
        if my >= TITLE_BAR_HEIGHT as f32 && my <= (TITLE_BAR_HEIGHT + TAB_BAR_HEIGHT) as f32 {
            let mut tx = 6.0f32;
            for i in 0..self.tabs.len() {
                let tab_w = TAB_WIDTH as f32;
                if mx >= tx && mx <= tx + tab_w {
                    self.switch_tab(i);
                    return;
                }
                tx += tab_w + TAB_SPACING as f32;
            }
            // New tab button
            if mx >= tx && mx <= tx + 28.0 {
                self.new_tab();
            }
            return;
        }

        // Nav bar clicks
        if my >= btn_y && my <= btn_bottom {
            let mut bx = NAV_START_X as f32;

            // Back
            if mx >= bx && mx <= bx + NAV_BTN_SIZE as f32 {
                tracing::info!("Back");
                return;
            }
            bx += NAV_BTN_SIZE as f32 + NAV_BTN_SPACING as f32;

            // Forward
            if mx >= bx && mx <= bx + NAV_BTN_SIZE as f32 {
                tracing::info!("Forward");
                return;
            }
            bx += NAV_BTN_SIZE as f32 + NAV_BTN_SPACING as f32;

            // Reload
            if mx >= bx && mx <= bx + NAV_BTN_SIZE as f32 {
                tracing::info!("Reload");
                return;
            }
            bx += NAV_BTN_SIZE as f32 + NAV_BTN_SPACING as f32;

            // Home
            if mx >= bx && mx <= bx + NAV_BTN_SIZE as f32 {
                self.go_home();
                tracing::info!("Home");
                return;
            }
            bx += NAV_BTN_SIZE as f32 + 12.0;

            // Address bar
            if mx >= bx {
                self.url_focused = true;
                self.url_cursor = self.url_bar.len();
                return;
            }
        }

        // Click outside unfocuses address bar
        self.url_focused = false;
    }

    fn handle_key(&mut self, key: &winit::keyboard::Key) {
        if !self.url_focused {
            return;
        }

        match key {
            Key::Named(NamedKey::Backspace) => {
                if self.url_cursor > 0 {
                    self.url_cursor -= 1;
                    self.url_bar.remove(self.url_cursor);
                }
            }
            Key::Named(NamedKey::Delete) => {
                if self.url_cursor < self.url_bar.len() {
                    self.url_bar.remove(self.url_cursor);
                }
            }
            Key::Named(NamedKey::ArrowLeft) => {
                self.url_cursor = self.url_cursor.saturating_sub(1);
            }
            Key::Named(NamedKey::ArrowRight) => {
                if self.url_cursor < self.url_bar.len() {
                    self.url_cursor += 1;
                }
            }
            Key::Named(NamedKey::Home) => self.url_cursor = 0,
            Key::Named(NamedKey::End) => self.url_cursor = self.url_bar.len(),
            Key::Named(NamedKey::Enter) => {
                if !self.url_bar.is_empty() {
                    let url = self.resolve_url();
                    self.navigate(url.clone());
                    tracing::info!("Navigating to: {}", url);
                }
                self.url_focused = false;
            }
            Key::Named(NamedKey::Escape) => {
                self.url_focused = false;
            }
            Key::Character(c) => {
                let ch = c.as_str();
                if !ch.is_empty()
                    && !matches!(
                        key,
                        Key::Named(NamedKey::Shift)
                            | Key::Named(NamedKey::Control)
                            | Key::Named(NamedKey::Alt)
                            | Key::Named(NamedKey::Super)
                    )
                {
                    for chr in ch.chars() {
                        self.url_bar.insert(self.url_cursor, chr);
                        self.url_cursor += 1;
                    }
                }
            }
            _ => {}
        }
    }
}

pub async fn run(config: AppConfig) -> Result<()> {
    tracing::info!(
        "Starting Noir Browser window (model: {:?})",
        config.process_model
    );

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

    let mut app = NoirApp::new();

    event_loop.run_app(&mut app)?;

    Ok(())
}
