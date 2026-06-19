//! Noir Browser - Entry Point
//!
//! Ultra minimalista: UNA SOLA LLAMADA al bootstrap.
//! Usa Tokio runtime en un thread separado para operaciones async.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use no_chromium::{create_browser, AppConfig};

fn main() {
    // Iniciar Tokio runtime en un thread separado para operaciones async
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");

    // Mantener el runtime vivo durante toda la ejecución
    let _guard = rt.enter();

    // Ejecutar el navegador (winit event loop es bloqueante)
    if let Err(e) = create_browser(AppConfig::default()).and_then(|b| b.run()) {
        eprintln!("Fatal error: {}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_appconfig_default() {
        let config = AppConfig::default();
        assert!(config.max_tabs > 0);
        assert!(config.max_tabs <= 100);
        assert!(config.cache_size_mb > 0);
    }

    #[test]
    fn test_create_browser() {
        let config = AppConfig::default();
        let result = no_chromium::create_browser(config);
        assert!(result.is_ok());
    }
}
