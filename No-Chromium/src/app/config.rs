use crate::utils::process_model::ProcessModel;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub process_model: ProcessModel,
    pub enable_privacy: bool,
    pub enable_tor_mode: bool,
    pub enable_ultrafast: bool,
    pub debug_webgpu: bool,
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
            debug_webgpu: false,
            enable_msdf_fonts: false,
            max_tabs: 20,
            cache_size_mb: 512,
        }
    }
}
