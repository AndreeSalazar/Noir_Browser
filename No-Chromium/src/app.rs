use crate::utils::process_model::ProcessModel;
use anyhow::Result;
use std::sync::Arc;
use tokio::runtime::Runtime;

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
            // Usamos SingleProcess como fallback seguro
            process_model: ProcessModel::SingleProcess,
            enable_privacy: true,
            enable_tor_mode: false,
            enable_ultrafast: false,
            debug_vulkan: false,
            enable_msdf_fonts: false,
            max_tabs: 20,
            cache_size_mb: 512,
        }
    }
}

pub async fn run(_config: AppConfig, _runtime: Arc<Runtime>) -> Result<()> {
    tracing::info!("🚀 Application loop started...");
    // Aquí irá el loop principal de la aplicación
    Ok(())
}
