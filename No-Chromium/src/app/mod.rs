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
        let display_url = self.display_url();
        let url_color = self.url_text_color();
        let url_bar_empty = self.url_bar.is_empty();
        let url_focused = self.url_focused;
        let url_cursor = self.url_cursor;
        let active_tab = self.active_tab;
        let tabs_len = self.tabs.len();
        let tab_titles: Vec<String> = self.tabs.iter().map(|t| {
            if t.title.len() > 22 {
                format!("{}...", &t.title[..19])
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

        // Clear background
        for pixel in buf.iter_mut() {
            *pixel = BG_CONTENT;
        }

        let w = width as i32;
        let h = height as i32;

        // === TITLE BAR ===
        draw_rect(buf, stride, 0, 0, w, TITLE_BAR_HEIGHT as i32, BG_TITLEBAR);
        draw_text_noir(buf, stride, w, 12, 12, "Noir Browser", TEXT_WHITE, 1.0);

        // Window controls (top right)
        let btn_size = 14;
        let btn_y = 13;
        let close_x = w - 20;
        draw_rect(buf, stride, close_x, btn_y, btn_size, btn_size, ACCENT);
        let min_x = close_x - 24;
        draw_rect(buf, stride, min_x, btn_y, btn_size, btn_size, YELLOW);
        let max_x = min_x - 24;
        draw_rect(buf, stride, max_x, btn_y, btn_size, btn_size, GREEN);

        // === TAB BAR ===
        let tab_y = TITLE_BAR_HEIGHT as i32;
        draw_rect(buf, stride, 0, tab_y, w, TAB_BAR_HEIGHT as i32, BG_TAB_BAR);

        let tab_width = 180i32;
        let tab_margin = 4i32;
        for (i, title) in tab_titles.iter().enumerate() {
            let tx = 8 + (i as i32) * (tab_width + tab_margin);
            let ty = tab_y + 4;
            let th = TAB_BAR_HEIGHT as i32 - 8;
            let bg = if i == active_tab { BG_ADDRESS_BAR } else { BG_TAB_BAR };
            draw_rect(buf, stride, tx, ty, tab_width, th, bg);
            draw_text_noir(buf, stride, w, tx + 10, ty + 8, title, TEXT_DIM, 0.8);
        }

        // New tab button (+)
        let plus_x = 8 + (tabs_len as i32) * (tab_width + tab_margin) + 4;
        draw_rect(buf, stride, plus_x, tab_y + 4, 28, TAB_BAR_HEIGHT as i32 - 8, BG_TAB_BAR);
        draw_text_noir(buf, stride, w, plus_x + 9, tab_y + 10, "+", TEXT_WHITE, 1.2);

        // === NAV BAR ===
        let nav_y = (TITLE_BAR_HEIGHT + TAB_BAR_HEIGHT) as i32;
        draw_rect(buf, stride, 0, nav_y, w, NAV_BAR_HEIGHT as i32, BG_DARK);

        let btn_w = 32i32;
        let btn_h = 28i32;
        let btn_y_pos = nav_y + 8;

        // Navigation buttons
        draw_rect(buf, stride, 10, btn_y_pos, btn_w, btn_h, BTN_HOVER);
        draw_text_noir(buf, stride, w, 20, btn_y_pos + 6, "<", TEXT_WHITE, 1.0);

        draw_rect(buf, stride, 48, btn_y_pos, btn_w, btn_h, BTN_HOVER);
        draw_text_noir(buf, stride, w, 58, btn_y_pos + 6, ">", TEXT_WHITE, 1.0);

        draw_rect(buf, stride, 86, btn_y_pos, btn_w, btn_h, BTN_HOVER);
        draw_text_noir(buf, stride, w, 96, btn_y_pos + 6, "R", TEXT_WHITE, 1.0);

        draw_rect(buf, stride, 124, btn_y_pos, btn_w, btn_h, BTN_HOVER);
        draw_text_noir(buf, stride, w, 134, btn_y_pos + 6, "H", TEXT_WHITE, 1.0);

        // Address bar
        let ab_x = 168;
        let ab_w = w - ab_x - 20;
        draw_rect(buf, stride, ab_x, btn_y_pos, ab_w, btn_h, BG_ADDRESS_BAR);
        draw_rect(buf, stride, ab_x, btn_y_pos, ab_w, 1, BORDER);

        draw_text_noir(buf, stride, w, ab_x + 12, btn_y_pos + 8, &display_url, url_color, 0.9);

        // Cursor line when focused
        if url_focused {
            let cursor_px = (url_cursor as i32) * 8 + ab_x + 12;
            draw_rect(buf, stride, cursor_px, btn_y_pos + 6, 1, btn_h - 12, ACCENT);
        }

        // Lock icon
        if !url_bar_empty {
            draw_rect(buf, stride, ab_x + ab_w - 20, btn_y_pos + 8, 10, 14, GREEN);
        }

        // Border bottom
        draw_rect(buf, stride, 0, nav_y + NAV_BAR_HEIGHT as i32 - 1, w, 1, BORDER);

        // === CONTENT AREA ===
        let content_y = TOOLBAR_HEIGHT as i32;
        let content_h = h - content_y;
        draw_rect(buf, stride, 0, content_y, w, content_h, BG_CONTENT);

        if active_url.is_empty() {
            // Default new tab page
            let center_y = content_y + content_h / 2 - 40;
            draw_text_noir(buf, stride, w, w / 2 - 100, center_y, "NOIR", ACCENT, 3.0);
            draw_text_noir(buf, stride, w, w / 2 - 90, center_y + 50, "BROWSER", TEXT_DIM, 1.5);
            draw_text_noir(
                buf, stride, w, w / 2 - 130, center_y + 90,
                "Ultra-fast | Private | Vulkan-powered", TEXT_DIM, 0.8,
            );

            // Quick links
            let link_y = center_y + 140;
            let links = ["Google", "GitHub", "YouTube", "Rust Lang"];
            let link_colors = [LINK_GOOGLE, LINK_GITHUB, LINK_YOUTUBE, LINK_RUST];
            let spacing = 140;
            let start_x = w / 2 - (links.len() as i32 * spacing) / 2;
            for (i, (name, color)) in links.iter().zip(link_colors.iter()).enumerate() {
                let lx = start_x + (i as i32) * spacing;
                draw_rect(buf, stride, lx, link_y, 100, 100, BG_LINK_CARD);
                draw_rect(buf, stride, lx, link_y, 100, 4, *color);
                draw_text_noir(buf, stride, w, lx + 10, link_y + 40, name, TEXT_WHITE, 0.85);
            }
        } else {
            let url_text = format!("Loading: {}", active_url);
            draw_text_noir(buf, stride, w, 20, content_y + 30, &url_text, TEXT_DIM, 0.9);
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
        let btn_y = nav_y + 8.0;

        // Address bar click
        if my >= btn_y && my <= btn_y + 28.0 && mx >= 168.0 {
            self.url_focused = true;
            self.url_cursor = self.url_bar.len();
            return;
        }

        // Tab bar clicks
        if my >= TITLE_BAR_HEIGHT as f32 && my <= (TITLE_BAR_HEIGHT + TAB_BAR_HEIGHT) as f32 {
            let tab_width = 184.0;
            let idx = ((mx - 8.0) / tab_width) as usize;
            if idx < self.tabs.len() {
                self.switch_tab(idx);
                return;
            }
            // New tab button
            let plus_x = 8.0 + (self.tabs.len() as f32) * tab_width + 4.0;
            if mx >= plus_x && mx <= plus_x + 28.0 {
                self.new_tab();
            }
            return;
        }

        // Nav buttons
        if my >= btn_y && my <= btn_y + 28.0 {
            if mx >= 10.0 && mx <= 42.0 {
                tracing::info!("Back");
            } else if mx >= 48.0 && mx <= 80.0 {
                tracing::info!("Forward");
            } else if mx >= 86.0 && mx <= 118.0 {
                tracing::info!("Reload");
            } else if mx >= 124.0 && mx <= 156.0 {
                self.go_home();
                tracing::info!("Home");
            }
            return;
        }

        // Click outside address bar unfocuses
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
