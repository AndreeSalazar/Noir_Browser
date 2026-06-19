//! AppContext - Estado principal de la aplicación
//!
//! Maneja todo el estado de la aplicación: ventana, tabs, renderers, etc.

use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use winit::window::Window;

use super::config::AppConfig;
use super::input;
use super::navigation;
use super::renderer;
use super::state::TabState;
use crate::network::fetch::HttpFetcher;

/// Mensaje de consola (log, warn, error, info)
#[derive(Clone, Debug)]
pub struct ConsoleMessage {
    pub level: ConsoleLevel,
    pub text: String,
    pub timestamp: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConsoleLevel {
    Log,
    Warn,
    Error,
    Info,
}

/// Contexto principal de la aplicación
pub struct AppContext {
    // Configuración
    pub config: AppConfig,

    // Window
    pub window: Option<Rc<Window>>,
    pub surface: Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>,
    pub width: u32,
    pub height: u32,

    // UI State
    pub url_bar: String,
    pub url_cursor: usize,
    pub url_focused: bool,
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub is_maximized: bool,
    pub should_close: bool,
    pub is_hovering_link: bool,
    pub load_progress: f32,
    pub console_open: bool,
    pub console_messages: Vec<ConsoleMessage>,
    pub find_open: bool,
    pub find_query: String,
    pub shortcuts_open: bool,
    pub loading_anim_frame: u32,

    // Tabs
    pub tabs: Vec<TabState>,
    pub active_tab: usize,
    pub next_tab_id: u64,

    // History
    pub history: Vec<String>,
    pub history_index: usize,

    // Modifiers
    pub modifiers: winit::keyboard::ModifiersState,

    // Fetching
    pub fetching: bool,
    pub fetch_result: Option<Arc<Mutex<Option<String>>>>,
    pub fetch_error: Option<String>,

    // HTTP
    pub http_fetcher: HttpFetcher,
}

impl AppContext {
    /// Crea un nuevo contexto con la configuración dada
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            window: None,
            surface: None,
            width: 1280,
            height: 720,
            url_bar: String::new(),
            url_cursor: 0,
            url_focused: false,
            mouse_x: 0.0,
            mouse_y: 0.0,
            is_maximized: false,
            should_close: false,
            is_hovering_link: false,
            load_progress: 0.0,
            console_open: false,
            console_messages: Vec::new(),
            find_open: false,
            find_query: String::new(),
            shortcuts_open: false,
            loading_anim_frame: 0,
            tabs: vec![Self::create_initial_tab()],
            active_tab: 0,
            next_tab_id: 2,
            history: Vec::new(),
            history_index: 0,
            modifiers: winit::keyboard::ModifiersState::empty(),
            fetching: false,
            fetch_result: None,
            fetch_error: None,
            http_fetcher: HttpFetcher::new(),
        }
    }

    /// Inicializa los subsistemas
    pub fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Initializing Noir Browser");
        tracing::info!("Config: {} max tabs, {}MB cache", self.config.max_tabs, self.config.cache_size_mb);
        Ok(())
    }

    /// Crea la pestaña inicial
    fn create_initial_tab() -> TabState {
        let mut tab = TabState::default();
        tab.tab_id = 1;
        tab
    }

    /// Maneja un click del mouse
    pub fn handle_click(&mut self) {
        input::handle_click(self);
    }

    /// Maneja una tecla presionada
    pub fn handle_key(&mut self, key: &winit::keyboard::Key, ctrl: bool) {
        input::handle_key(self, key, ctrl);
    }

    /// Dibuja un frame
    pub fn draw_frame(&mut self) {
        renderer::draw(self);
    }

    /// Recalcula el layout del tab activo
    pub fn recalculate_layout(&mut self) {
        let active = self.active_tab;
        if let Some(page) = &self.tabs[active].page {
            let viewport_w = self.width as f32;
            let blocks = crate::parsers::layout::layout_page(page, viewport_w);
            let content_h = crate::parsers::layout::total_content_height(&blocks);
            self.tabs[active].layout_blocks = blocks;
            self.tabs[active].content_height = content_h;
            self.tabs[active].scroll_y = 0.0;
        }
    }

    /// Actualiza el estado de hover
    pub fn update_hover(&mut self) {
        let mx = self.mouse_x;
        let my = self.mouse_y;
        let active = self.active_tab;
        let blocks = self.tabs[active].layout_blocks.clone();
        let scroll_y = self.tabs[active].scroll_y;
        self.is_hovering_link = crate::parsers::layout::hit_test_link(
            &blocks, mx, my, scroll_y
        ).is_some();
    }

    /// Avanza el frame de animación
    pub fn tick_animation(&mut self) {
        self.loading_anim_frame = self.loading_anim_frame.wrapping_add(1);
    }

    /// Agrega un mensaje a la consola
    pub fn console_log(&mut self, level: ConsoleLevel, text: String) {
        self.console_messages.push(ConsoleMessage {
            level,
            text,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        });
        if self.console_messages.len() > 500 {
            self.console_messages.remove(0);
        }
    }

    /// Procesa el resultado de un fetch
    pub fn process_fetch_result(&mut self) {
        navigation::process_fetch_result(self);
    }

    /// Procesa timers pendientes
    pub fn process_pending_timers(&mut self) {
        navigation::process_pending_timers(self);
    }

    /// Procesa imágenes pendientes
    pub fn process_image_dirty(&self) {
        if crate::media::take_image_dirty() {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }

    /// Navega a una URL
    pub fn navigate(&mut self, url: String) {
        navigation::navigate(self, url);
    }

    /// Va a la página de inicio
    pub fn go_home(&mut self) {
        navigation::go_home(self);
    }

    /// Resuelve una URL a su forma completa
    pub fn resolve_url(&self) -> String {
        navigation::resolve_url(&self.url_bar)
    }

    /// Crea una nueva pestaña
    pub fn new_tab(&mut self) {
        let max = self.config.max_tabs as usize;
        if self.tabs.len() < max {
            let mut tab = TabState::default();
            tab.tab_id = self.next_tab_id;
            self.next_tab_id += 1;
            self.tabs.push(tab);
            self.active_tab = self.tabs.len() - 1;
            self.url_bar.clear();
            self.url_cursor = 0;
        }
    }

    /// Cierra la pestaña actual
    pub fn close_current_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active_tab);
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len().saturating_sub(1);
            }
            self.url_bar = self.tabs[self.active_tab].url.clone();
            self.url_cursor = self.url_bar.len();
        }
    }

    /// Cambia a una pestaña específica
    pub fn switch_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_tab = index;
            self.url_bar = self.tabs[index].url.clone();
            self.url_cursor = self.url_bar.len();
        }
    }
}
