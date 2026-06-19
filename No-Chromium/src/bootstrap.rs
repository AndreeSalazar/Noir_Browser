//! Noir Browser - Bootstrap
//!
//! Punto de entrada limpio y modular del navegador.
//! Toda la inicialización se hace aquí en pasos claros y separados.

use std::error::Error;
use std::process;

use crate::app::AppConfig;
pub use crate::bootstrap::error::BootstrapError;
pub use crate::bootstrap::error::BootstrapResult;
use crate::bootstrap::logger::Logger;

/// Bootstrap principal - inicializa el navegador completo
pub fn run(config: AppConfig) -> BootstrapResult<()> {
    // Paso 1: Inicializar logging
    Logger::init(&config)?;

    // Paso 2: Validar configuración
    validate_config(&config)?;

    // Paso 3: Crear y ejecutar la aplicación
    crate::app::execute(config).map_err(|e| BootstrapError::Application(e.to_string()))
}

/// Valida la configuración antes de iniciar
fn validate_config(config: &AppConfig) -> BootstrapResult<()> {
    if config.max_tabs == 0 {
        return Err(BootstrapError::InvalidConfig(
            "max_tabs must be greater than 0".to_string()
        ));
    }
    if config.max_tabs > 100 {
        return Err(BootstrapError::InvalidConfig(
            "max_tabs cannot exceed 100".to_string()
        ));
    }
    if config.cache_size_mb > 1024 {
        return Err(BootstrapError::InvalidConfig(
            "cache_size_mb cannot exceed 1024MB".to_string()
        ));
    }
    Ok(())
}

/// Maneja errores fatales con mensaje amigable
pub fn handle_fatal_error(err: Box<dyn Error>) -> ! {
    eprintln!("💥 Fatal error: {}", err);
    eprintln!("\nFor support, visit: https://github.com/noir-browser/noir");
    process::exit(1);
}

// Módulos privados del bootstrap
mod logger;
pub mod error;

