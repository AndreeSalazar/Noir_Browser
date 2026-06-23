//! DOM Bindings - Bindings que WASM puede usar para interactuar con el DOM
//!
//! Proporciona imports que WASM puede usar:
//! - console: console.log, console.warn, console.error
//! - window: window.alert, window.prompt
//! - document: document.createElement, document.querySelector
//! - fetch: HTTP requests

use std::collections::HashMap;
use crate::wasm_host::imports::{ImportValue, ImportResolver};
use crate::wasm_host::WasmValue;

/// Registra todos los DOM bindings en un resolver
pub struct DomBindings;

impl DomBindings {
    pub fn register_all(resolver: &mut ImportResolver) {
        Self::register_console(resolver);
        Self::register_window(resolver);
        Self::register_document(resolver);
        Self::register_fetch(resolver);
    }

    fn register_console(resolver: &mut ImportResolver) {
        // console.log(ptr, len) - logs string from memory
        resolver.register_fn("console", "log", |args| {
            // Simplified: would read string from memory
            // For now, just return
            let _ = args;
            Ok(vec![])
        });

        resolver.register_fn("console", "warn", |args| {
            let _ = args;
            Ok(vec![])
        });

        resolver.register_fn("console", "error", |args| {
            let _ = args;
            Ok(vec![])
        });

        resolver.register_fn("console", "info", |args| {
            let _ = args;
            Ok(vec![])
        });
    }

    fn register_window(resolver: &mut ImportResolver) {
        // window.alert
        resolver.register_fn("window", "alert", |args| {
            let _ = args;
            Ok(vec![])
        });

        // window.confirm
        resolver.register_fn("window", "confirm", |_args| {
            Ok(vec![WasmValue::I32(1)]) // true
        });

        // window.prompt
        resolver.register_fn("window", "prompt", |_args| {
            Ok(vec![WasmValue::I32(0)]) // null
        });

        // window.setTimeout
        resolver.register_fn("window", "setTimeout", |_args| {
            Ok(vec![WasmValue::I32(1)]) // dummy id
        });

        // window.clearTimeout
        resolver.register_fn("window", "clearTimeout", |_args| {
            Ok(vec![])
        });
    }

    fn register_document(resolver: &mut ImportResolver) {
        // document.createElement
        resolver.register_fn("document", "createElement", |_args| {
            Ok(vec![WasmValue::I32(0)]) // dummy element id
        });

        // document.querySelector
        resolver.register_fn("document", "querySelector", |_args| {
            Ok(vec![WasmValue::I32(0)]) // null
        });

        // document.querySelectorAll
        resolver.register_fn("document", "querySelectorAll", |_args| {
            Ok(vec![WasmValue::I32(0)]) // empty list
        });

        // document.getElementById
        resolver.register_fn("document", "getElementById", |_args| {
            Ok(vec![WasmValue::I32(0)]) // null
        });

        // document.body
        resolver.register_fn("document", "getBody", |_args| {
            Ok(vec![WasmValue::I32(0)]) // dummy body id
        });
    }

    fn register_fetch(resolver: &mut ImportResolver) {
        // fetch(url_ptr, url_len) -> response_handle
        resolver.register_fn("fetch", "send", |_args| {
            Ok(vec![WasmValue::I32(0)]) // dummy response
        });
    }

    /// Lista de bindings disponibles
    pub fn list_bindings() -> HashMap<String, Vec<String>> {
        let mut map = HashMap::new();
        map.insert("console".to_string(), vec!["log".to_string(), "warn".to_string(),
            "error".to_string(), "info".to_string()]);
        map.insert("window".to_string(), vec!["alert".to_string(), "confirm".to_string(),
            "prompt".to_string(), "setTimeout".to_string(), "clearTimeout".to_string()]);
        map.insert("document".to_string(), vec!["createElement".to_string(),
            "querySelector".to_string(), "querySelectorAll".to_string(),
            "getElementById".to_string(), "getBody".to_string()]);
        map.insert("fetch".to_string(), vec!["send".to_string()]);
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_all() {
        let mut resolver = ImportResolver::new();
        DomBindings::register_all(&mut resolver);
        assert!(resolver.count() > 10);
    }

    #[test]
    fn test_console_log_registered() {
        let mut resolver = ImportResolver::new();
        DomBindings::register_all(&mut resolver);
        assert!(resolver.resolve("console", "log").is_some());
    }

    #[test]
    fn test_window_alert_registered() {
        let mut resolver = ImportResolver::new();
        DomBindings::register_all(&mut resolver);
        assert!(resolver.resolve("window", "alert").is_some());
    }

    #[test]
    fn test_document_query_registered() {
        let mut resolver = ImportResolver::new();
        DomBindings::register_all(&mut resolver);
        assert!(resolver.resolve("document", "querySelector").is_some());
    }

    #[test]
    fn test_list_bindings() {
        let bindings = DomBindings::list_bindings();
        assert!(bindings.contains_key("console"));
        assert!(bindings.contains_key("window"));
        assert!(bindings.contains_key("document"));
        assert!(bindings.contains_key("fetch"));
    }
}
