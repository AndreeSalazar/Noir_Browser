pub mod config;
pub mod draw;
pub mod glyphs;
pub mod state;
pub mod theme;

use anyhow::Result;
use std::num::NonZeroU32;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{WindowAttributes, WindowId};

pub use config::AppConfig;
use draw::{draw_rect, draw_text_noir, measure_text_width};
use state::NoirApp;
use theme::*;

use crate::network::fetch::HttpFetcher;
use crate::parsers::page_document::PageDocument;
use crate::parsers::layout::{layout_page, total_content_height, hit_test_link};

impl NoirApp {
    fn draw_frame(&mut self) {
        let display_url = self.display_url();
        let url_color = self.url_text_color();
        let url_bar_empty = self.url_bar.is_empty();
        let url_focused = self.url_focused;
        let active_tab = self.active_tab;
        let tab_titles: Vec<String> = self.tabs.iter().map(|t| {
            if t.title.len() > 20 {
                format!("{}...", &t.title[..17])
            } else {
                t.title.clone()
            }
        }).collect();
        let active_url = self.tabs[active_tab].url.clone();
        let layout_blocks = self.tabs[active_tab].layout_blocks.clone();
        let scroll_y = self.tabs[active_tab].scroll_y;

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

        for pixel in buf.iter_mut() {
            *pixel = BG_CONTENT;
        }

        let w = width as i32;
        let h = height as i32;

        // ═══════ TITLE BAR (custom, no OS) ═══════
        draw_rect(buf, stride, 0, 0, w, TITLE_BAR_HEIGHT as i32, BG_TITLEBAR);

        // App icon
        draw_rect(buf, stride, 10, 10, 14, 14, ACCENT);

        // Title text
        draw_text_noir(buf, stride, w, 30, 11, "Noir Browser", TEXT_DIM, 1.0);

        // Window controls (Chrome-style, full height)
        let ctrl_w = 46;
        let ctrl_h = TITLE_BAR_HEIGHT as i32;

        let min_x = w - ctrl_w * 3;
        draw_rect(buf, stride, min_x, 0, ctrl_w, ctrl_h, BTN_BG);
        draw_text_noir(buf, stride, w, min_x + 18, 11, "-", TEXT_DIM, 1.2);

        let max_x = w - ctrl_w * 2;
        draw_rect(buf, stride, max_x, 0, ctrl_w, ctrl_h, BTN_BG);
        draw_rect(buf, stride, max_x + 17, 11, 10, 10, TEXT_DIM);
        draw_rect(buf, stride, max_x + 18, 12, 8, 8, BG_TITLEBAR);

        let close_x = w - ctrl_w;
        draw_rect(buf, stride, close_x, 0, ctrl_w, ctrl_h, CLOSE_RED);
        draw_text_noir(buf, stride, w, close_x + 17, 11, "X", TEXT_WHITE, 1.0);

        // ═══════ TAB BAR ═══════
        let tab_y = TITLE_BAR_HEIGHT as i32;
        draw_rect(buf, stride, 0, tab_y, w, TAB_BAR_HEIGHT as i32, BG_TAB_BAR);

        let mut tx = 4i32;
        for (i, title) in tab_titles.iter().enumerate() {
            let tab_w = TAB_WIDTH.min(w - tx - 100);
            if tx + tab_w > w - 100 { break; }

            let ty = tab_y + 4;
            let th = TAB_BAR_HEIGHT as i32 - 8;

            if i == active_tab {
                draw_rect(buf, stride, tx, ty, tab_w, th, BG_ADDRESS_BAR);
                draw_rect(buf, stride, tx, ty, tab_w, 2, ACCENT);
            } else {
                draw_rect(buf, stride, tx, ty, tab_w, th, BG_TAB_BAR);
            }

            // Tab icon dot
            draw_rect(buf, stride, tx + 8, ty + (th / 2) - 3, 6, 6, ACCENT);

            let text_color = if i == active_tab { TEXT_WHITE } else { TEXT_DIM };
            draw_text_noir(buf, stride, w, tx + 20, ty + (th / 2) - 4, title, text_color, 0.9);

            // Close X
            draw_text_noir(buf, stride, w, tx + tab_w - 18, ty + (th / 2) - 4, "x", TEXT_DIM, 0.8);

            tx += tab_w + TAB_SPACING;
        }

        // New tab button
        if tx + 34 < w {
            draw_rect(buf, stride, tx + 2, tab_y + 7, 28, TAB_BAR_HEIGHT as i32 - 14, BTN_BG);
            draw_text_noir(buf, stride, w, tx + 10, tab_y + 11, "+", TEXT_DIM, 1.2);
        }

        // ═══════ NAV BAR ═══════
        let nav_y = (TITLE_BAR_HEIGHT + TAB_BAR_HEIGHT) as i32;
        draw_rect(buf, stride, 0, nav_y, w, NAV_BAR_HEIGHT as i32, BG_DARK);

        let btn_h = 34i32;
        let btn_y_pos = nav_y + (NAV_BAR_HEIGHT as i32 - btn_h) / 2;
        let mut bx = NAV_START_X;

        // Back
        draw_rect(buf, stride, bx, btn_y_pos, NAV_BTN_SIZE, btn_h, BTN_BG);
        draw_text_noir(buf, stride, w, bx + 13, btn_y_pos + 9, "<", TEXT_WHITE, 1.2);
        bx += NAV_BTN_SIZE + NAV_BTN_SPACING;

        // Forward
        draw_rect(buf, stride, bx, btn_y_pos, NAV_BTN_SIZE, btn_h, BTN_BG);
        draw_text_noir(buf, stride, w, bx + 13, btn_y_pos + 9, ">", TEXT_WHITE, 1.2);
        bx += NAV_BTN_SIZE + NAV_BTN_SPACING;

        // Reload
        draw_rect(buf, stride, bx, btn_y_pos, NAV_BTN_SIZE, btn_h, BTN_BG);
        draw_text_noir(buf, stride, w, bx + 13, btn_y_pos + 9, "R", TEXT_WHITE, 1.2);
        bx += NAV_BTN_SIZE + NAV_BTN_SPACING;

        // Home
        draw_rect(buf, stride, bx, btn_y_pos, NAV_BTN_SIZE, btn_h, BTN_BG);
        draw_text_noir(buf, stride, w, bx + 13, btn_y_pos + 9, "H", TEXT_WHITE, 1.2);
        bx += NAV_BTN_SIZE + 14;

        // ═══════ ADDRESS BAR ═══════
        let ab_w = w - bx - 16;
        if ab_w > 80 {
            let ab_bg = if url_focused { BG_ADDRESS_BAR_FOCUS } else { BG_ADDRESS_BAR };
            draw_rect(buf, stride, bx, btn_y_pos, ab_w, btn_h, ab_bg);

            let text_x = bx + 14;
            let text_y = btn_y_pos + (btn_h / 2) - 5;

            if url_focused || !url_bar_empty {
                draw_text_noir(buf, stride, w, text_x, text_y, &display_url, url_color, 1.0);

                if url_focused {
                    let cursor_px = text_x + measure_text_width(&display_url, 1.0) + 2;
                    draw_rect(buf, stride, cursor_px, text_y, 2, 10, TEXT_WHITE);
                }
            } else {
                draw_text_noir(buf, stride, w, text_x, text_y, "Search or enter URL...", TEXT_PLACEHOLDER, 1.0);
            }

            if !url_bar_empty {
                let lock_x = bx + ab_w - 30;
                let lock_y = btn_y_pos + (btn_h / 2) - 5;
                draw_rect(buf, stride, lock_x, lock_y + 3, 8, 7, GREEN);
                draw_rect(buf, stride, lock_x + 1, lock_y, 6, 5, GREEN);
            }
        }

        // ═══════ CONTENT ═══════
        let content_y = TOOLBAR_HEIGHT as i32;
        let content_h = h - content_y;

        if active_url.is_empty() {
            // New tab page
            let center_y = content_y + content_h / 2 - 80;

            draw_text_noir(buf, stride, w, w / 2 - 115, center_y, "NOIR", ACCENT, 3.5);
            draw_text_noir(buf, stride, w, w / 2 - 130, center_y + 55, "BROWSER", TEXT_DIM, 2.0);

            draw_text_noir(
                buf, stride, w, w / 2 - 170, center_y + 100,
                "Ultra-fast  |  Private  |  Vulkan-powered", TEXT_PLACEHOLDER, 1.0,
            );

            // Quick links
            let link_y = center_y + 160;
            let links = [
                ("Google", LINK_GOOGLE),
                ("GitHub", LINK_GITHUB),
                ("YouTube", LINK_YOUTUBE),
                ("Rust", LINK_RUST),
            ];
            let total_w = links.len() as i32 * LINK_CARD_SIZE + (links.len() as i32 - 1) * LINK_CARD_SPACING;
            let start_x = w / 2 - total_w / 2;

            for (i, (name, color)) in links.iter().enumerate() {
                let lx = start_x + i as i32 * (LINK_CARD_SIZE + LINK_CARD_SPACING);

                draw_rect(buf, stride, lx, link_y, LINK_CARD_SIZE, LINK_CARD_SIZE, BG_LINK_CARD);
                draw_rect(buf, stride, lx, link_y, LINK_CARD_SIZE, 3, *color);

                let icon_size = 24;
                let icon_x = lx + (LINK_CARD_SIZE - icon_size) / 2;
                let icon_y = link_y + 24;
                draw_rect(buf, stride, icon_x, icon_y, icon_size, icon_size, *color);

                let label_w = name.len() as i32 * 7;
                let label_x = lx + (LINK_CARD_SIZE - label_w) / 2;
                draw_text_noir(buf, stride, w, label_x, link_y + LINK_CARD_SIZE + 14, name, TEXT_DIM, 1.0);
            }
        } else if self.fetching {
            // Loading state
            draw_text_noir(buf, stride, w, w / 2 - 50, content_y + 40, "Loading...", TEXT_DIM, 1.2);
            draw_text_noir(buf, stride, w, 30, content_y + 80, &active_url, TEXT_PLACEHOLDER, 1.0);
        } else if !layout_blocks.is_empty() {
            // Render layout blocks
            render_layout_blocks(buf, stride, w, content_y, content_h, &layout_blocks, scroll_y);
        } else {
            draw_text_noir(buf, stride, w, w / 2 - 50, content_y + 40, "Empty", TEXT_DIM, 1.2);
        }

        // Scroll indicator
        if layout_blocks.is_empty() == false {
            let total_h = layout_blocks.iter().map(|b| b.y + b.h).fold(0.0f32, f32::max);
            if total_h > content_h as f32 {
                let view_ratio = content_h as f32 / total_h;
                let scroll_ratio = scroll_y / (total_h - content_h as f32).max(1.0);
                let bar_h = (content_h as f32 * view_ratio).max(20.0);
                let bar_y = content_y as f32 + scroll_ratio * (content_h as f32 - bar_h);
                draw_rect(buf, stride, w - 6, bar_y as i32, 4, bar_h as i32, 0x40FFFFFF);
            }
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

fn render_layout_blocks(
    buf: &mut [u32],
    stride: usize,
    screen_w: i32,
    content_y: i32,
    content_h: i32,
    blocks: &[crate::parsers::layout::LayoutBlock],
    scroll_y: f32,
) {
    use theme::*;
    use draw::{draw_rect, draw_text_noir};

    for block in blocks {
        let screen_block_y = block.y - scroll_y + content_y as f32;

        // Skip blocks outside viewport
        if screen_block_y + block.h < content_y as f32 - 10.0 {
            continue;
        }
        if screen_block_y > content_y as f32 + content_h as f32 + 10.0 {
            continue;
        }

        // Draw background if present
        if let Some(bg) = &block.bg_color {
            let bg_u32 = rgba_to_u32(bg[0], bg[1], bg[2], bg[3]);
            draw_rect(
                buf,
                stride,
                block.x as i32 - 4,
                screen_block_y as i32 - 2,
                (block.w + block.padding_left + 8.0) as i32,
                (block.h + block.padding_top + 4.0) as i32,
                bg_u32,
            );
        }

        // Draw link underline
        if block.is_link {
            let underline_y = screen_block_y as i32 + block.h as i32 - 1;
            draw_rect(
                buf,
                stride,
                block.x as i32,
                underline_y,
                block.w as i32,
                1,
                0xFF6699FF,
            );
        }

        // Draw text
        let color_u32 = rgba_to_u32(block.color[0], block.color[1], block.color[2], block.color[3]);
        let font_scale = block.font_size / 14.0;
        draw_text_noir(
            buf,
            stride,
            screen_w,
            block.x as i32,
            screen_block_y as i32,
            &block.text,
            color_u32,
            font_scale,
        );
    }
}

fn rgba_to_u32(r: f32, g: f32, b: f32, a: f32) -> u32 {
    let ri = (r * 255.0) as u32;
    let gi = (g * 255.0) as u32;
    let bi = (b * 255.0) as u32;
    let ai = (a * 255.0) as u32;
    (ai << 24) | (ri << 16) | (gi << 8) | bi
}

impl ApplicationHandler for NoirApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = WindowAttributes::default()
            .with_title("Noir Browser")
            .with_inner_size(LogicalSize::new(1280.0, 720.0))
            .with_min_inner_size(LogicalSize::new(800.0, 500.0))
            .with_decorations(false);

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
                if self.should_close {
                    event_loop.exit();
                    return;
                }
                self.window.as_ref().unwrap().request_redraw();
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_amount = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                };
                self.scroll(scroll_amount);
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

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.should_close {
            event_loop.exit();
        }

        // Check if fetch completed
        if self.fetching {
            let mut done = false;
            if let Some(result) = &self.fetch_result {
                if let Ok(guard) = result.try_lock() {
                    if let Some(html) = guard.as_ref() {
                        let url = self.tabs[self.active_tab].url.clone();
                        tracing::info!("Parsing HTML for {} ({} bytes)", url, html.len());
                        let page = PageDocument::from_html(&url, html);
                        let title = page.title.clone();
                        let num_links = page.links.len();
                        let viewport_w = self.width as f32;
                        let blocks = layout_page(&page, viewport_w);
                        let content_h = total_content_height(&blocks);
                        tracing::info!("Parsed: title='{}', {} links, {} layout blocks, content height: {:.0}", title, num_links, blocks.len(), content_h);
                        self.tabs[self.active_tab].page = Some(page);
                        self.tabs[self.active_tab].layout_blocks = blocks;
                        self.tabs[self.active_tab].content_height = content_h;
                        self.tabs[self.active_tab].title = if !title.is_empty() { title } else { url };
                        done = true;
                    }
                }
            }
            if done {
                self.fetching = false;
                self.fetch_result = None;
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
        }
    }
}

impl NoirApp {
    fn handle_click(&mut self) {
        let mx = self.mouse_x;
        let my = self.mouse_y;
        let w = self.width as f32;

        // Window controls (title bar)
        if my <= TITLE_BAR_HEIGHT as f32 {
            let ctrl_w = 46.0f32;

            // Close button
            let close_x = w - ctrl_w;
            if mx >= close_x {
                self.should_close = true;
                return;
            }

            // Maximize button
            let max_x = w - ctrl_w * 2.0;
            if mx >= max_x && mx < close_x {
                if let Some(window) = &self.window {
                    self.is_maximized = !self.is_maximized;
                    window.set_maximized(self.is_maximized);
                }
                return;
            }

            // Minimize button
            let min_x = w - ctrl_w * 3.0;
            if mx >= min_x && mx < max_x {
                if let Some(window) = &self.window {
                    window.set_minimized(true);
                }
                return;
            }

            // Title bar drag area (not on controls)
            if mx < min_x {
                if let Some(window) = &self.window {
                    let _ = window.drag_window();
                }
                return;
            }
        }

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

        // Content area click - check for link hit
        let content_top = TOOLBAR_HEIGHT as f32;
        if my > content_top {
            let layout_blocks = self.tabs[self.active_tab].layout_blocks.clone();
            let scroll_y = self.tabs[self.active_tab].scroll_y;
            if let Some(href) = hit_test_link(&layout_blocks, mx, my, scroll_y) {
                tracing::info!("Link clicked: {}", href);
                self.url_bar = href.clone();
                self.url_cursor = self.url_bar.len();
                self.navigate(href);
                self.window.as_ref().unwrap().request_redraw();
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
                            }
                        }
                    });

                    self.fetch_result = Some(result_holder);
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
