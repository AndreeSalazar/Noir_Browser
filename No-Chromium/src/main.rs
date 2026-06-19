//! Noir Browser - Entry Point
//!
//! Ultra minimalista: UNA SOLA LLAMADA al bootstrap.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use no_chromium::{create_browser, AppConfig};

fn main() {
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
