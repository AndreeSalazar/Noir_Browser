//! Noir Browser - Biblioteca Pública
//!
//! Este módulo expone la API pública del navegador para:
//! - Integración con componentes externos
//! - Testing modular
//! - Plugins y extensiones futuras
//!
//! 🧬 Arquitectura: Chrome × Tor × Vulkan

// === MÓDULOS PÚBLICOS ===

/// Coordinador principal de la aplicación
pub mod app;

/// Gestión de navegador: tabs, navegación, historial
pub mod browser;

/// Motor de renderizado: parser, layout, JS engine
pub mod renderer;

/// Motor Vulkan ultra-fast para renderizado GPU
pub mod vulkan_engine;

/// Módulo de red: fetch, proxy SOCKS5, DNS-over-HTTPS
pub mod network;

/// Utilidades compartidas: IPC, memory management, helpers
pub mod utils;

/// Módulo de privacidad (feature-gated)
#[cfg(feature = "privacy")]
pub mod privacy;

// === TIPOS PÚBLICOS RE-EXPORTADOS ===

pub use crate::browser::TabId;
// pub use crate::renderer::RenderContext; // TODO: Implementar en Fase 1
pub use crate::vulkan_engine::FrameInfo;
// pub use crate::network::{Request, Response, NetworkError}; // TODO: Implementar en Fase 5

#[cfg(feature = "privacy")]
pub use crate::browser::privacy::{FirstPartyIsolation, FingerprintSeed};

// === CONFIGURACIÓN PÚBLICA ===

// Re-export ProcessModel desde utils
pub use crate::utils::ProcessModel;

/// Configuración principal de la aplicación
#[derive(Clone, Debug)]
pub struct AppConfig {
    pub process_model: ProcessModel,
    pub enable_privacy: bool,
    pub enable_tor_mode: bool,
    pub enable_ultrafast: bool,
    pub max_tabs: usize,
    pub cache_size_mb: usize,
    pub enable_debug: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            process_model: ProcessModel::from_available_ram(crate::utils::detect_available_ram()),
            enable_privacy: true,
            enable_tor_mode: false,
            enable_ultrafast: cfg!(feature = "ultrafast"),
            max_tabs: 10,
            cache_size_mb: 256,
            enable_debug: cfg!(debug_assertions),
        }
    }
}

// === FUNCIONES DE ALTO NIVEL ===

/// Crea una nueva instancia del navegador con la configuración especificada
/// 
/// # Ejemplo
/// ```no_run
/// use noir_browser::{create_browser, AppConfig, ProcessModel};
/// 
/// # async fn example() -> anyhow::Result<()> {
/// let config = AppConfig {
///     process_model: ProcessModel::FullIsolation,
///     enable_privacy: true,
///     enable_tor_mode: false,
///     ..Default::default()
/// };
/// 
/// create_browser(config).await?;
/// # Ok(())
/// # }
/// ```
pub async fn create_browser(_config: AppConfig) -> anyhow::Result<BrowserInstance> {
    // Delegar al coordinador interno
    // Stub para Fase 0 - delegar al app::run() en modo single-process
    #[cfg(not(test))]
    {
        crate::app::run()?;
    }
    Ok(BrowserInstance { _private: () })
}

/// Instancia activa del navegador
#[derive(Debug)]
pub struct BrowserInstance {
    _private: (),
}

impl BrowserInstance {
    /// Abre una nueva pestaña con la URL especificada
    pub async fn open_tab(&self, _url: &str) -> anyhow::Result<TabId> {
        // Implementación delegada al browser coordinator
        unimplemented!()
    }
    
    /// Cierra una pestaña por su ID
    pub async fn close_tab(&self, _tab_id: TabId) -> anyhow::Result<()> {
        unimplemented!()
    }
    
    /// Navega a una URL en una pestaña existente
    pub async fn navigate(&self, _tab_id: TabId, _url: &str) -> anyhow::Result<()> {
        unimplemented!()
    }
}

// === UTILIDADES PARA TESTING ===

#[cfg(test)]
pub mod test_utils {
    use super::*;
    
    /// Crea una configuración de test con recursos mínimos
    pub fn test_config() -> AppConfig {
        AppConfig {
            process_model: ProcessModel::SingleProcess,
            enable_privacy: false,
            enable_tor_mode: false,
            enable_ultrafast: false,
            max_tabs: 2,
            cache_size_mb: 64,
            ..Default::default()
        }
    }
    
    /// Mock de Vulkan para tests sin GPU
    #[cfg(feature = "ultrafast")]
    pub struct MockVulkanEngine;
    
    #[cfg(feature = "ultrafast")]
    impl MockVulkanEngine {
        pub fn new() -> Self { Self }
        pub async fn initialize(&self) -> anyhow::Result<()> { Ok(()) }
    }
}

// === METADATOS DE LA CRATE ===

/// Versión semántica del navegador
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Descripción del proyecto
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// Autores del proyecto
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

/// Obtiene información de build-time sobre las features activas
pub fn active_features() -> &'static [&'static str] {
    &[
        #[cfg(feature = "ultrafast")]
        "ultrafast",
        #[cfg(feature = "privacy")]
        "privacy",
        #[cfg(feature = "tor_mode")]
        "tor_mode",
        #[cfg(feature = "msdf_fonts")]
        "msdf_fonts",
        #[cfg(feature = "debug_vulkan")]
        "debug_vulkan",
        #[cfg(feature = "fallback_vulkano")]
        "fallback_vulkano",
        #[cfg(feature = "video_decode")]
        "video_decode",
        #[cfg(feature = "local_search")]
        "local_search",
    ]
}
