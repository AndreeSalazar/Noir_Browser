//! Noir Browser - Library entry point

pub mod app;
pub mod bootstrap;
pub mod bridge;
pub mod js_engine_v3;
pub mod media;
pub mod network;
pub mod parsers;
pub mod renderer_trait;
pub mod utils;
pub mod wasm_v2;
pub mod webgpu;

pub use app::AppConfig;
pub use app::AppContext;
pub use bootstrap::{BootstrapError, BootstrapResult};

/// Crea una instancia del navegador
pub fn create_browser(config: AppConfig) -> BootstrapResult<BrowserInstance> {
    tracing::info!("Creating Noir Browser instance");
    Ok(BrowserInstance { config })
}

/// Instancia del navegador
pub struct BrowserInstance {
    config: AppConfig,
}

impl BrowserInstance {
    /// Ejecuta el navegador
    pub fn run(self) -> BootstrapResult<()> {
        crate::bootstrap::run(self.config)
    }

    /// Obtiene la configuración
    pub fn config(&self) -> &AppConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::process_model::ProcessModel;

    #[test]
    fn test_process_model_selection() {
        assert_eq!(ProcessModel::from_available_ram(1024), ProcessModel::SingleProcess);
        assert_eq!(ProcessModel::from_available_ram(3072), ProcessModel::Aggregated);
        assert_eq!(ProcessModel::from_available_ram(6144), ProcessModel::ModerateIsolation);
    }

    #[test]
    fn test_max_renderer_processes() {
        assert_eq!(ProcessModel::SingleProcess.max_renderer_processes(), 1);
        assert_eq!(ProcessModel::Aggregated.max_renderer_processes(), 2);
        assert_eq!(ProcessModel::ModerateIsolation.max_renderer_processes(), 4);
    }

    #[test]
    fn test_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.enable_ultrafast, cfg!(feature = "ultrafast"));
        assert_eq!(config.enable_privacy, cfg!(feature = "privacy"));
    }
}
