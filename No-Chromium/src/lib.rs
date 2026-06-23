//! Noir Browser - Library entry point
//!
//! Modular architecture with backward-compatible aliases.
//!
//! | New name     | Maps to           | Description              |
//! |--------------|-------------------|--------------------------|
//! | `core`       | `app::context`/`app::config` | Core types  |
//! | `browser`    | `app::*`          | Browser UI panels        |
//! | `html`       | `parsers::*`      | HTML parsing             |
//! | `css`        | `parsers::*`      | CSS parsing              |
//! | `layout`     | `parsers::*`      | Layout engine            |
//! | `media`      | `media`           | Media (sub-modules)      |
//! | `js`         | `js_engine_v3`    | JavaScript engine        |
//! | `wasm`       | `wasm_host`       | WebAssembly              |
//! | `gpu`        | `webgpu`          | WebGPU                   |
//! | `security`   | `security`        | Security                 |
//! | `ui`         | `ui`              | UI components            |
//! | `features`   | `features`        | User features            |
//! | `devtools`   | `devtools` + extras | DevTools               |
//! | `storage`    | `storage`         | Persistence              |
//! | `network`    | `network`         | Network                  |
//!
//! Both old and new names are valid.

// ===== Original module names (canonical paths) =====
pub mod app;
pub mod bootstrap;
pub mod bridge;
pub mod devtools;
pub mod features;
pub mod js_engine_v3;
pub mod media;
pub mod network;
pub mod parsers;
pub mod renderer_trait;
pub mod security;
pub mod storage;
pub mod ui;
pub mod utils;
pub mod wasm_host;
pub mod webgpu;

// ===== New module aliases (re-exports for ergonomic access) =====

/// Core types - re-exports
pub mod core {
    pub use crate::app::context::*;
    pub use crate::app::config::*;
}

/// Browser UI panels
pub mod browser {
    pub use crate::app::config::*;
    pub use crate::app::context::*;
    pub use crate::app::event_loop::*;
    pub use crate::app::navigation::*;
    pub use crate::app::state::*;
    pub use crate::app::draw::*;
    pub use crate::app::glyphs::*;
}

/// HTML parsing
pub mod html {
    pub mod parser {
        pub use crate::parsers::dom_tree::*;
        pub use crate::parsers::dom_native::*;
        pub use crate::parsers::html_elements::*;
        pub use crate::parsers::js_lexer::*;
    }
    pub mod extract {
        pub use crate::parsers::page_document::*;
        pub use crate::parsers::resource_loader::*;
        pub use crate::parsers::webidl_bridge::*;
    }
}

/// CSS
pub mod css {
    pub use crate::parsers::css_simple::*;
    pub use crate::parsers::css_lexer::*;
    pub use crate::parsers::css_engine::*;
    pub use crate::parsers::style_collector::*;
}

/// Layout
pub mod layout {
    pub use crate::parsers::layout::*;
    pub use crate::parsers::flexbox::*;
}

/// Media sub-modules
pub mod media_alias {
    pub mod images {
        pub use crate::media::image_support::*;
        pub use crate::media::image_manager::*;
    }
    pub mod video {
        pub use crate::media::video::*;
        pub use crate::media::pipeline::*;
        pub use crate::media::video_texture::*;
        pub use crate::media::mse::*;
        pub mod decoder {
            pub use crate::media::frame::*;
            pub use crate::media::yuv_gpu::*;
            pub use crate::media::video_codecs::*;
        }
        pub mod streaming {
            pub use crate::media::hls::*;
            pub use crate::media::dash::*;
            pub use crate::media::mp4::*;
            pub use crate::media::http_range::*;
        }
    }
    pub mod audio {
        pub use crate::media::audio::*;
        pub use crate::media::audio_playback::*;
    }
    pub mod subtitles {
        pub use crate::media::webvtt::*;
    }
}

/// JS engine alias
pub mod js {
    pub use crate::js_engine_v3::*;
    pub mod interpreter {
        pub use crate::js_engine_v3::Interpreter;
    }
}

/// WASM alias
pub mod wasm {
    pub use crate::wasm_host::*;
}

/// GPU alias
pub mod gpu {
    pub use crate::webgpu::*;
}

/// Security sub-modules
pub mod security_alias {
    pub mod ad_blocker {
        pub use crate::ui::ad_blocker::*;
    }
}

/// Privacy
pub mod privacy {
}

/// UI sub-modules
pub mod ui_alias {
    pub mod text {
        pub mod markdown {
            pub use crate::ui::markdown::*;
        }
        pub mod shaper {
            pub use crate::ui::text_shaping::*;
        }
    }
    pub mod interaction {
        pub mod click_feedback {
            pub use crate::ui::click_feedback::*;
        }
        pub mod highlight {
            pub use crate::ui::element_highlight::*;
        }
    }
    pub mod widgets {
        pub mod card {
            pub use crate::ui::card_layout::*;
        }
        pub mod list {
            pub use crate::ui::list_render::*;
        }
        pub mod table {
            pub use crate::ui::table_render::*;
        }
    }
    pub mod css {
        pub mod grid {
            pub use crate::ui::css_grid::*;
        }
        pub mod containment {
            pub use crate::ui::css_containment::*;
        }
    }
    pub mod player {
        pub mod controls {
            pub use crate::media::player_ui::*;
        }
    }
}

/// Features sub-modules
pub mod features_alias {
    pub mod search {
        pub mod find {
            pub use crate::features::find_in_page::*;
        }
    }
    pub mod reader {
        pub mod reader {
            pub use crate::features::reader_mode::*;
        }
    }
    pub mod screenshots {
        pub mod screenshot {
            pub use crate::features::screenshot::*;
        }
    }
    pub mod printing {
        pub mod pdf {
            pub use crate::features::print_pdf::*;
        }
    }
    pub mod pwa {
        pub mod install {
            pub use crate::features::pwa::*;
        }
    }
    pub mod downloads {
        pub mod manager {
            pub use crate::devtools::form_fill::*;
        }
    }
    pub mod passwords {
        pub mod manager {
            pub use crate::features::password_manager::*;
        }
    }
    pub mod autofill {
        pub mod form {
            pub use crate::devtools::form_fill::*;
        }
    }
    pub mod permissions {
        pub mod permissions {
            pub use crate::features::permissions::*;
        }
    }
    pub mod tabs {
        pub mod groups {
            pub use crate::features::tab_groups::*;
        }
    }
    pub mod favorites {
        pub mod favorites {
            pub use crate::features::favorites::*;
        }
    }
}

/// DevTools
pub mod devtools_alias {
    pub use crate::devtools::*;
    pub use crate::features::network_monitor::NetworkMonitor;
    pub use crate::features::service_worker::ServiceWorkerManager;
}

// ===== Public re-exports =====

pub use app::{AppConfig, AppContext};
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
