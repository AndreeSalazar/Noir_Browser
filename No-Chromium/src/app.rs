use crate::utils::process_model::ProcessModel;
use anyhow::Result;
use std::num::NonZeroU32;
use std::rc::Rc;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub process_model: ProcessModel,
    pub enable_privacy: bool,
    pub enable_tor_mode: bool,
    pub enable_ultrafast: bool,
    pub debug_vulkan: bool,
    pub enable_msdf_fonts: bool,
    pub max_tabs: u32,
    pub cache_size_mb: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            process_model: ProcessModel::SingleProcess,
            enable_privacy: cfg!(feature = "privacy"),
            enable_tor_mode: false,
            enable_ultrafast: cfg!(feature = "ultrafast"),
            debug_vulkan: false,
            enable_msdf_fonts: false,
            max_tabs: 20,
            cache_size_mb: 512,
        }
    }
}

const TITLE_BAR_HEIGHT: u32 = 40;
const TAB_BAR_HEIGHT: u32 = 36;
const NAV_BAR_HEIGHT: u32 = 44;
const TOOLBAR_HEIGHT: u32 = TITLE_BAR_HEIGHT + TAB_BAR_HEIGHT + NAV_BAR_HEIGHT;

// Noir Dark Theme colors
const BG_DARK: u32 = 0xFF_1a1a2e;
const BG_TITLEBAR: u32 = 0xFF_16213e;
const BG_TAB_BAR: u32 = 0xFF_0f3460;
const BG_ADDRESS_BAR: u32 = 0xFF_202040;
const BG_CONTENT: u32 = 0xFF_12121f;
const TEXT_WHITE: u32 = 0xFF_e0e0e0;
const TEXT_DIM: u32 = 0xFF_888899;
const ACCENT: u32 = 0xFF_e94560;
const BTN_HOVER: u32 = 0xFF_2a2a4a;
const BORDER: u32 = 0xFF_2a2a3e;

struct NoirApp {
    window: Option<Rc<Window>>,
    surface: Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>,
    width: u32,
    height: u32,
    url_bar: String,
    url_cursor: usize,
    url_focused: bool,
    tabs: Vec<TabState>,
    active_tab: usize,
    mouse_x: f32,
    mouse_y: f32,
}

struct TabState {
    title: String,
    url: String,
}

impl Default for TabState {
    fn default() -> Self {
        Self {
            title: "New Tab".into(),
            url: String::new(),
        }
    }
}

impl NoirApp {
    fn new() -> Self {
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

    fn draw_frame(&mut self) {
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
        draw_rect(buf, stride, min_x, btn_y, btn_size, btn_size, 0xFF_888844);
        let max_x = min_x - 24;
        draw_rect(buf, stride, max_x, btn_y, btn_size, btn_size, 0xFF_44aa44);

        // === TAB BAR ===
        let tab_y = TITLE_BAR_HEIGHT as i32;
        draw_rect(buf, stride, 0, tab_y, w, TAB_BAR_HEIGHT as i32, BG_TAB_BAR);

        let tab_width = 180i32;
        let tab_margin = 4i32;
        for (i, tab) in self.tabs.iter().enumerate() {
            let tx = 8 + (i as i32) * (tab_width + tab_margin);
            let ty = tab_y + 4;
            let th = TAB_BAR_HEIGHT as i32 - 8;
            let bg = if i == self.active_tab { BG_ADDRESS_BAR } else { BG_TAB_BAR };
            draw_rect(buf, stride, tx, ty, tab_width, th, bg);

            let title = if tab.title.len() > 22 {
                format!("{}...", &tab.title[..19])
            } else {
                tab.title.clone()
            };
            draw_text_noir(buf, stride, w, tx + 10, ty + 8, &title, TEXT_DIM, 0.8);
        }

        // New tab button (+)
        let plus_x = 8 + (self.tabs.len() as i32) * (tab_width + tab_margin) + 4;
        draw_rect(buf, stride, plus_x, tab_y + 4, 28, TAB_BAR_HEIGHT as i32 - 8, BG_TAB_BAR);
        draw_text_noir(buf, stride, w, plus_x + 9, tab_y + 10, "+", TEXT_WHITE, 1.2);

        // === NAV BAR ===
        let nav_y = (TITLE_BAR_HEIGHT + TAB_BAR_HEIGHT) as i32;
        draw_rect(buf, stride, 0, nav_y, w, NAV_BAR_HEIGHT as i32, BG_DARK);

        // Nav buttons
        let btn_w = 32i32;
        let btn_h = 28i32;
        let btn_y_pos = nav_y + 8;

        // Back button
        draw_rect(buf, stride, 10, btn_y_pos, btn_w, btn_h, BTN_HOVER);
        draw_text_noir(buf, stride, w, 20, btn_y_pos + 6, "<", TEXT_WHITE, 1.0);

        // Forward button
        draw_rect(buf, stride, 48, btn_y_pos, btn_w, btn_h, BTN_HOVER);
        draw_text_noir(buf, stride, w, 58, btn_y_pos + 6, ">", TEXT_WHITE, 1.0);

        // Reload button
        draw_rect(buf, stride, 86, btn_y_pos, btn_w, btn_h, BTN_HOVER);
        draw_text_noir(buf, stride, w, 96, btn_y_pos + 6, "R", TEXT_WHITE, 1.0);

        // Home button
        draw_rect(buf, stride, 124, btn_y_pos, btn_w, btn_h, BTN_HOVER);
        draw_text_noir(buf, stride, w, 134, btn_y_pos + 6, "H", TEXT_WHITE, 1.0);

        // Address bar
        let ab_x = 168;
        let ab_w = w - ab_x - 20;
        draw_rect(buf, stride, ab_x, btn_y_pos, ab_w, btn_h, BG_ADDRESS_BAR);
        draw_rect(buf, stride, ab_x, btn_y_pos, ab_w, 1, BORDER);

        // Address bar text or placeholder
        let display_url = if self.url_bar.is_empty() && !self.url_focused {
            String::from("Search or enter URL...")
        } else if self.url_bar.is_empty() && self.url_focused {
            String::new()
        } else {
            self.url_bar.clone()
        };

        let url_color = if self.url_bar.is_empty() && !self.url_focused {
            TEXT_DIM
        } else {
            TEXT_WHITE
        };

        draw_text_noir(buf, stride, w, ab_x + 12, btn_y_pos + 8, &display_url, url_color, 0.9);

        // Cursor line when focused
        if self.url_focused {
            let cursor_px = (self.url_cursor as i32) * 8 + ab_x + 12;
            draw_rect(buf, stride, cursor_px, btn_y_pos + 6, 1, btn_h - 12, ACCENT);
        }

        // Lock icon
        if !self.url_bar.is_empty() {
            draw_rect(buf, stride, ab_x + ab_w - 20, btn_y_pos + 8, 10, 14, 0xFF_44aa44);
        }

        // Border bottom
        draw_rect(buf, stride, 0, nav_y + NAV_BAR_HEIGHT as i32 - 1, w, 1, BORDER);

        // === CONTENT AREA ===
        let content_y = TOOLBAR_HEIGHT as i32;
        let content_h = h - content_y;
        draw_rect(buf, stride, 0, content_y, w, content_h, BG_CONTENT);

        // Default page content
        if self.tabs[self.active_tab].url.is_empty() {
            let center_y = content_y + content_h / 2 - 40;

            // Noir Browser logo text
            draw_text_noir(buf, stride, w, w / 2 - 100, center_y, "NOIR", ACCENT, 3.0);
            draw_text_noir(
                buf,
                stride,
                w,
                w / 2 - 90,
                center_y + 50,
                "BROWSER",
                TEXT_DIM,
                1.5,
            );
            draw_text_noir(
                buf,
                stride,
                w,
                w / 2 - 130,
                center_y + 90,
                "Ultra-fast | Private | Vulkan-powered",
                TEXT_DIM,
                0.8,
            );

            // Quick links
            let link_y = center_y + 140;
            let links = ["Google", "GitHub", "YouTube", "Rust Lang"];
            let link_colors = [0xFF_4285f4, 0xFF_f0f0f0, 0xFF_FF0000, 0xFF_de5616];
            let spacing = 140;
            let start_x = w / 2 - (links.len() as i32 * spacing) / 2;
            for (i, (name, color)) in links.iter().zip(link_colors.iter()).enumerate() {
                let lx = start_x + (i as i32) * spacing;
                draw_rect(buf, stride, lx, link_y, 100, 100, 0xFF_1a1a3a);
                draw_rect(buf, stride, lx, link_y, 100, 4, *color);
                draw_text_noir(buf, stride, w, lx + 10, link_y + 40, name, TEXT_WHITE, 0.85);
            }
        } else {
            // Show loaded URL
            let url_text = format!("Loading: {}", self.tabs[self.active_tab].url);
            draw_text_noir(buf, stride, w, 20, content_y + 30, &url_text, TEXT_DIM, 0.9);
        }

        buffer.present().unwrap();
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
                let mx = self.mouse_x;
                let my = self.mouse_y;

                // Address bar click
                let nav_y = (TITLE_BAR_HEIGHT + TAB_BAR_HEIGHT) as f32;
                let btn_y = nav_y + 8.0;
                if my >= btn_y && my <= btn_y + 28.0 && mx >= 168.0 {
                    self.url_focused = true;
                    self.url_cursor = self.url_bar.len();
                    self.window.as_ref().unwrap().request_redraw();
                    return;
                }

                // Tab clicks
                if my >= TITLE_BAR_HEIGHT as f32
                    && my <= (TITLE_BAR_HEIGHT + TAB_BAR_HEIGHT) as f32
                {
                    let tab_width = 184.0;
                    let idx = ((mx - 8.0) / tab_width) as usize;
                    if idx < self.tabs.len() {
                        self.active_tab = idx;
                        self.url_bar = self.tabs[idx].url.clone();
                        self.url_cursor = self.url_bar.len();
                        self.url_focused = false;
                    }
                    // New tab button
                    let plus_x = 8.0 + (self.tabs.len() as f32) * tab_width + 4.0;
                    if mx >= plus_x && mx <= plus_x + 28.0 && self.tabs.len() < 20 {
                        self.tabs.push(TabState::default());
                        self.active_tab = self.tabs.len() - 1;
                        self.url_bar.clear();
                        self.url_cursor = 0;
                    }
                    self.window.as_ref().unwrap().request_redraw();
                    return;
                }

                // Back button
                if mx >= 10.0 && mx <= 42.0 && my >= btn_y && my <= btn_y + 28.0 {
                    tracing::info!("Back button clicked");
                }
                // Forward
                if mx >= 48.0 && mx <= 80.0 && my >= btn_y && my <= btn_y + 28.0 {
                    tracing::info!("Forward button clicked");
                }
                // Reload
                if mx >= 86.0 && mx <= 118.0 && my >= btn_y && my <= btn_y + 28.0 {
                    tracing::info!("Reload button clicked");
                }
                // Home
                if mx >= 124.0 && mx <= 156.0 && my >= btn_y && my <= btn_y + 28.0 {
                    self.url_bar.clear();
                    self.url_cursor = 0;
                    self.tabs[self.active_tab].url.clear();
                    self.tabs[self.active_tab].title = "New Tab".into();
                    tracing::info!("Home button clicked");
                }

                // Click outside address bar unfocuses it
                if !(my >= btn_y && my <= btn_y + 28.0 && mx >= 168.0) {
                    self.url_focused = false;
                }
                self.window.as_ref().unwrap().request_redraw();
            }

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key,
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                if !self.url_focused {
                    return;
                }

                match &logical_key {
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
                    Key::Named(NamedKey::Home) => {
                        self.url_cursor = 0;
                    }
                    Key::Named(NamedKey::End) => {
                        self.url_cursor = self.url_bar.len();
                    }
                    Key::Named(NamedKey::Enter) => {
                        if !self.url_bar.is_empty() {
                            let url = if self.url_bar.starts_with("http://")
                                || self.url_bar.starts_with("https://")
                            {
                                self.url_bar.clone()
                            } else if self.url_bar.contains('.') && !self.url_bar.contains(' ') {
                                format!("https://{}", self.url_bar)
                            } else {
                                format!(
                                    "https://duckduckgo.com/?q={}",
                                    self.url_bar.replace(' ', "+")
                                )
                            };
                            self.tabs[self.active_tab].url = url.clone();
                            self.tabs[self.active_tab].title = url.clone();
                            tracing::info!("Navigating to: {}", url);
                        }
                        self.url_focused = false;
                    }
                    Key::Named(NamedKey::Escape) => {
                        self.url_focused = false;
                    }
                    Key::Character(c) => {
                        let ch = c.as_str();
                        if !ch.is_empty() && !matches!(
                            &logical_key,
                            Key::Named(NamedKey::Shift)
                                | Key::Named(NamedKey::Control)
                                | Key::Named(NamedKey::Alt)
                                | Key::Named(NamedKey::Super)
                        ) {
                            for chr in ch.chars() {
                                self.url_bar.insert(self.url_cursor, chr);
                                self.url_cursor += 1;
                            }
                        }
                    }
                    _ => {}
                }
                self.window.as_ref().unwrap().request_redraw();
            }

            WindowEvent::ModifiersChanged(mods) => {
                let _ = mods;
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
    app.url_focused = false;

    event_loop.run_app(&mut app)?;

    Ok(())
}

// === SOFTWARE RENDERING HELPERS ===

fn draw_rect(buf: &mut [u32], stride: usize, x: i32, y: i32, w: i32, h: i32, color: u32) {
    let sw = stride as i32;
    for row in y..y + h {
        if row < 0 || row >= buf.len() as i32 / sw {
            continue;
        }
        for col in x..x + w {
            if col >= 0 && col < sw {
                let idx = (row * sw + col) as usize;
                if idx < buf.len() {
                    buf[idx] = color;
                }
            }
        }
    }
}

fn draw_text_noir(
    buf: &mut [u32],
    stride: usize,
    screen_w: i32,
    x: i32,
    y: i32,
    text: &str,
    color: u32,
    scale: f32,
) {
    let sw = stride as i32;
    let char_w = (7.0 * scale) as i32;
    let _char_h = (10.0 * scale) as i32;
    let spacing = (1.0 * scale) as i32;
    let r = ((color >> 16) & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let b = (color & 0xFF) as u8;

    for (ci, ch) in text.chars().enumerate() {
        let cx = x + ci as i32 * (char_w + spacing);
        if cx + char_w > screen_w {
            break;
        }
        if ch == ' ' {
            continue;
        }
        let glyph = get_glyph_bitmap(ch);
        for gy in 0..glyph.len() {
            for gx in 0..glyph[0].len() {
                if glyph[gy][gx] {
                    for sy in 0..scale as i32 {
                        for sx in 0..scale as i32 {
                            let px = cx + gx as i32 + sx;
                            let py = y + gy as i32 + sy;
                            if px >= 0 && px < sw && py >= 0 {
                                let idx = (py * sw + px) as usize;
                                if idx < buf.len() {
                                    buf[idx] = 0xFF_000000
                                        | ((r as u32) << 16)
                                        | ((g as u32) << 8)
                                        | b as u32;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn get_glyph_bitmap(ch: char) -> &'static [[bool; 6]; 8] {
    match ch {
        'A' => &[
            [false, false, true, true, false, false],
            [false, true, false, false, true, false],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, true, true, true, true, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
        ],
        'B' => &[
            [true, true, true, true, true, false],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, true, true, true, true, false],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, true, true, true, true, false],
        ],
        'C' => &[
            [false, true, true, true, true, false],
            [true, false, false, false, false, true],
            [true, false, false, false, false, false],
            [true, false, false, false, false, false],
            [true, false, false, false, false, false],
            [true, false, false, false, false, false],
            [true, false, false, false, false, true],
            [false, true, true, true, true, false],
        ],
        'D' => &[
            [true, true, true, true, true, false],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, true, true, true, true, false],
        ],
        'E' => &[
            [true, true, true, true, true, true],
            [true, false, false, false, false, false],
            [true, false, false, false, false, false],
            [true, true, true, true, false, false],
            [true, false, false, false, false, false],
            [true, false, false, false, false, false],
            [true, false, false, false, false, false],
            [true, true, true, true, true, true],
        ],
        'F' => &[
            [true, true, true, true, true, true],
            [true, false, false, false, false, false],
            [true, false, false, false, false, false],
            [true, true, true, true, false, false],
            [true, false, false, false, false, false],
            [true, false, false, false, false, false],
            [true, false, false, false, false, false],
            [true, false, false, false, false, false],
        ],
        'G' => &[
            [false, true, true, true, true, false],
            [true, false, false, false, false, true],
            [true, false, false, false, false, false],
            [true, false, false, false, false, false],
            [true, false, true, true, true, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [false, true, true, true, true, false],
        ],
        'H' => &[
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, true, true, true, true, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
        ],
        'I' => &[
            [true, true, true, true, true, true],
            [false, false, true, true, false, false],
            [false, false, true, true, false, false],
            [false, false, true, true, false, false],
            [false, false, true, true, false, false],
            [false, false, true, true, false, false],
            [false, false, true, true, false, false],
            [true, true, true, true, true, true],
        ],
        'L' => &[
            [true, false, false, false, false, false],
            [true, false, false, false, false, false],
            [true, false, false, false, false, false],
            [true, false, false, false, false, false],
            [true, false, false, false, false, false],
            [true, false, false, false, false, false],
            [true, false, false, false, false, false],
            [true, true, true, true, true, true],
        ],
        'N' => &[
            [true, false, false, false, false, true],
            [true, true, false, false, false, true],
            [true, false, true, false, false, true],
            [true, false, false, true, false, true],
            [true, false, false, false, true, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
        ],
        'O' => &[
            [false, true, true, true, true, false],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [false, true, true, true, true, false],
        ],
        'R' => &[
            [true, true, true, true, true, false],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, true, true, true, true, false],
            [true, false, true, false, false, false],
            [true, false, false, true, false, false],
            [true, false, false, false, true, false],
            [true, false, false, false, false, true],
        ],
        'S' => &[
            [false, true, true, true, true, false],
            [true, false, false, false, false, true],
            [true, false, false, false, false, false],
            [false, true, true, true, true, false],
            [false, false, false, false, false, true],
            [false, false, false, false, false, true],
            [true, false, false, false, false, true],
            [false, true, true, true, true, false],
        ],
        'T' => &[
            [true, true, true, true, true, true],
            [false, false, true, true, false, false],
            [false, false, true, true, false, false],
            [false, false, true, true, false, false],
            [false, false, true, true, false, false],
            [false, false, true, true, false, false],
            [false, false, true, true, false, false],
            [false, false, true, true, false, false],
        ],
        'U' => &[
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [false, true, true, true, true, false],
        ],
        'b' => &[
            [true, false, false, false, false, false],
            [true, false, false, false, false, false],
            [true, true, true, true, true, false],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, true, true, true, true, false],
        ],
        'c' => &[
            [false, false, false, false, false, false],
            [false, false, false, false, false, false],
            [false, true, true, true, true, false],
            [true, false, false, false, false, false],
            [true, false, false, false, false, false],
            [true, false, false, false, false, false],
            [true, false, false, false, false, false],
            [false, true, true, true, true, false],
        ],
        'd' => &[
            [false, false, false, false, true, false],
            [false, false, false, false, true, false],
            [false, true, true, true, true, false],
            [true, false, false, false, true, false],
            [true, false, false, false, true, false],
            [true, false, false, false, true, false],
            [true, false, false, false, true, false],
            [false, true, true, true, true, false],
        ],
        'e' => &[
            [false, false, false, false, false, false],
            [false, false, false, false, false, false],
            [false, true, true, true, true, false],
            [true, false, false, false, false, true],
            [true, true, true, true, true, true],
            [true, false, false, false, false, false],
            [true, false, false, false, false, false],
            [false, true, true, true, true, false],
        ],
        'g' => &[
            [false, false, false, false, false, false],
            [false, true, true, true, true, false],
            [true, false, false, false, true, false],
            [true, false, false, false, true, false],
            [false, true, true, true, true, false],
            [false, false, false, false, true, false],
            [false, true, true, true, true, false],
            [true, false, false, false, false, false],
        ],
        'h' => &[
            [true, false, false, false, false, false],
            [true, false, false, false, false, false],
            [true, true, true, true, true, false],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
        ],
        'i' => &[
            [false, false, true, false, false, false],
            [false, false, false, false, false, false],
            [false, true, true, false, false, false],
            [false, false, true, false, false, false],
            [false, false, true, false, false, false],
            [false, false, true, false, false, false],
            [false, false, true, false, false, false],
            [false, true, true, true, false, false],
        ],
        'l' => &[
            [false, true, true, false, false, false],
            [false, false, true, false, false, false],
            [false, false, true, false, false, false],
            [false, false, true, false, false, false],
            [false, false, true, false, false, false],
            [false, false, true, false, false, false],
            [false, false, true, false, false, false],
            [false, true, true, true, false, false],
        ],
        'o' => &[
            [false, false, false, false, false, false],
            [false, false, false, false, false, false],
            [false, true, true, true, true, false],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [false, true, true, true, true, false],
        ],
        'r' => &[
            [false, false, false, false, false, false],
            [false, false, false, false, false, false],
            [true, true, true, true, true, false],
            [true, false, false, false, false, true],
            [true, false, false, false, false, false],
            [true, false, false, false, false, false],
            [true, false, false, false, false, false],
            [true, false, false, false, false, false],
        ],
        's' => &[
            [false, false, false, false, false, false],
            [false, false, false, false, false, false],
            [false, true, true, true, true, false],
            [true, false, false, false, false, false],
            [false, true, true, true, false, false],
            [false, false, false, false, true, false],
            [true, false, false, false, false, false],
            [false, true, true, true, true, false],
        ],
        't' => &[
            [false, false, true, false, false, false],
            [false, true, true, true, false, false],
            [false, false, true, false, false, false],
            [false, false, true, false, false, false],
            [false, false, true, false, false, false],
            [false, false, true, false, false, false],
            [false, false, true, false, true, false],
            [false, false, false, true, false, false],
        ],
        'u' => &[
            [false, false, false, false, false, false],
            [false, false, false, false, false, false],
            [true, false, false, false, true, false],
            [true, false, false, false, true, false],
            [true, false, false, false, true, false],
            [true, false, false, false, true, false],
            [true, false, false, false, true, false],
            [false, true, true, true, false, false],
        ],
        'w' => &[
            [false, false, false, false, false, false],
            [false, false, false, false, false, false],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, true, false, true, true],
            [true, false, true, false, true, true],
            [false, true, false, true, false, false],
        ],
        '<' => &[
            [false, false, false, false, false, false],
            [false, false, false, true, false, false],
            [false, false, true, false, false, false],
            [false, true, false, false, false, false],
            [true, false, false, false, false, false],
            [false, true, false, false, false, false],
            [false, false, true, false, false, false],
            [false, false, false, true, false, false],
        ],
        '>' => &[
            [false, false, false, false, false, false],
            [false, false, true, false, false, false],
            [false, false, false, true, false, false],
            [false, false, false, false, true, false],
            [false, false, false, false, false, true],
            [false, false, false, false, true, false],
            [false, false, false, true, false, false],
            [false, false, true, false, false, false],
        ],
        '+' => &[
            [false, false, false, false, false, false],
            [false, false, true, true, false, false],
            [false, false, true, true, false, false],
            [true, true, true, true, true, true],
            [true, true, true, true, true, true],
            [false, false, true, true, false, false],
            [false, false, true, true, false, false],
            [false, false, false, false, false, false],
        ],
        '.' => &[
            [false, false, false, false, false, false],
            [false, false, false, false, false, false],
            [false, false, false, false, false, false],
            [false, false, false, false, false, false],
            [false, false, false, false, false, false],
            [false, false, false, false, false, false],
            [false, true, true, false, false, false],
            [false, true, true, false, false, false],
        ],
        ':' => &[
            [false, false, false, false, false, false],
            [false, false, false, false, false, false],
            [false, false, true, true, false, false],
            [false, false, true, true, false, false],
            [false, false, false, false, false, false],
            [false, false, true, true, false, false],
            [false, false, true, true, false, false],
            [false, false, false, false, false, false],
        ],
        '/' => &[
            [false, false, false, false, false, true],
            [false, false, false, false, true, false],
            [false, false, false, false, true, false],
            [false, false, false, true, false, false],
            [false, false, false, true, false, false],
            [false, false, true, false, false, false],
            [false, false, true, false, false, false],
            [false, true, false, false, false, false],
        ],
        '=' => &[
            [false, false, false, false, false, false],
            [false, false, false, false, false, false],
            [true, true, true, true, true, true],
            [false, false, false, false, false, false],
            [false, false, false, false, false, false],
            [true, true, true, true, true, true],
            [false, false, false, false, false, false],
            [false, false, false, false, false, false],
        ],
        '-' => &[
            [false, false, false, false, false, false],
            [false, false, false, false, false, false],
            [false, false, false, false, false, false],
            [false, false, false, false, false, false],
            [true, true, true, true, true, true],
            [false, false, false, false, false, false],
            [false, false, false, false, false, false],
            [false, false, false, false, false, false],
        ],
        ' ' => &[[false; 6]; 8],
        _ => &[
            [true, true, true, true, true, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, false, false, false, false, true],
            [true, true, true, true, true, true],
        ],
    }
}
