//! Logger - Inicialización del sistema de logging

use crate::app::AppConfig;
use crate::bootstrap::error::BootstrapError;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Inicializa el sistema de logging
pub struct Logger;

impl Logger {
    pub fn init(config: &AppConfig) -> Result<(), BootstrapError> {
        let filter = if config.debug_webgpu {
            EnvFilter::new("noir=debug,wgpu=info")
        } else {
            EnvFilter::new("noir=info")
        };

        let layer = fmt::layer()
            .with_target(true)
            .with_thread_ids(false)
            .with_file(false)
            .with_line_number(false);

        tracing_subscriber::registry()
            .with(filter)
            .with(layer)
            .try_init()
            .map_err(|e| BootstrapError::LoggerInit(e.to_string()))?;

        tracing::info!("Logger initialized");
        Ok(())
    }
}
