//! Configuración de la aplicación

use crate::utils::process_model::ProcessModel;

/// Configuración principal de Noir Browser
#[derive(Clone, Debug)]
pub struct AppConfig {
    /// Modelo de proceso (single, multi, etc.)
    pub process_model: ProcessModel,
    /// Habilitar features de privacidad
    pub enable_privacy: bool,
    /// Habilitar modo Tor
    pub enable_tor_mode: bool,
    /// Habilitar rendering ultra-rápido (WebGPU)
    pub enable_ultrafast: bool,
    /// Debug de WebGPU
    pub debug_webgpu: bool,
    /// Habilitar fuentes MSDF
    pub enable_msdf_fonts: bool,
    /// Máximo número de tabs
    pub max_tabs: u32,
    /// Tamaño de cache en MB
    pub cache_size_mb: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            process_model: ProcessModel::SingleProcess,
            enable_privacy: cfg!(feature = "privacy"),
            enable_tor_mode: false,
            enable_ultrafast: cfg!(feature = "ultrafast"),
            debug_webgpu: false,
            enable_msdf_fonts: false,
            max_tabs: 20,
            cache_size_mb: 512,
        }
    }
}
