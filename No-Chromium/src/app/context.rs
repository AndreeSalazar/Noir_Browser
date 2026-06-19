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
