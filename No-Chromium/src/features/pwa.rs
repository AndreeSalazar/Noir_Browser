//! PWA - Progressive Web Apps
//!
//! Soporte para manifest.json, service workers, e instalación.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct WebManifest {
    pub name: String,
    pub short_name: String,
    pub start_url: String,
    pub display: String,
    pub background_color: String,
    pub theme_color: String,
    pub icons: Vec<ManifestIcon>,
}

#[derive(Debug, Clone)]
pub struct ManifestIcon {
    pub src: String,
    pub sizes: String,
    pub icon_type: String,
}

impl WebManifest {
    /// Parsea un manifest.json
    pub fn from_json(json: &str) -> Result<Self, String> {
        // Simplified JSON parsing
        let name = extract_json_string(json, "name").unwrap_or_default();
        let short_name = extract_json_string(json, "short_name").unwrap_or_default();
        let start_url = extract_json_string(json, "start_url").unwrap_or("/".to_string());
        let display = extract_json_string(json, "display").unwrap_or("browser".to_string());
        let background_color = extract_json_string(json, "background_color").unwrap_or("#ffffff".to_string());
        let theme_color = extract_json_string(json, "theme_color").unwrap_or("#000000".to_string());

        Ok(Self {
            name,
            short_name,
            start_url,
            display,
            background_color,
            theme_color,
            icons: Vec::new(),
        })
    }

    /// Crea un manifest por defecto
    pub fn default_for(url: &str) -> Self {
        Self {
            name: "Noir App".to_string(),
            short_name: "App".to_string(),
            start_url: url.to_string(),
            display: "standalone".to_string(),
            background_color: "#0E0E14".to_string(),
            theme_color: "#FF3344".to_string(),
            icons: Vec::new(),
        }
    }

    /// Genera el JSON
    pub fn to_json(&self) -> String {
        let mut s = String::from("{\n");
        s.push_str(&format!("  \"name\": \"{}\",\n", self.name));
        s.push_str(&format!("  \"short_name\": \"{}\",\n", self.short_name));
        s.push_str(&format!("  \"start_url\": \"{}\",\n", self.start_url));
        s.push_str(&format!("  \"display\": \"{}\",\n", self.display));
        s.push_str(&format!("  \"background_color\": \"{}\",\n", self.background_color));
        s.push_str(&format!("  \"theme_color\": \"{}\"\n", self.theme_color));
        s.push_str("}");
        s
    }
}

fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let search = format!("\"{}\":", key);
    let pos = json.find(&search)?;
    let rest = &json[pos + search.len()..];
    let rest = rest.trim_start();
    if !rest.starts_with('"') {
        return None;
    }
    let rest = &rest[1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

#[derive(Debug, Clone)]
pub struct ServiceWorker {
    pub scope: String,
    pub script_url: String,
    pub state: ServiceWorkerState,
    pub registrations: Vec<ServiceWorkerRegistration>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ServiceWorkerState {
    Installing,
    Installed,
    Activating,
    Activated,
    Redundant,
}

#[derive(Debug, Clone)]
pub struct ServiceWorkerRegistration {
    pub id: u64,
    pub scope: String,
    pub script_url: String,
    pub registered_at: u64,
}

pub struct PwaManager {
    manifest: Arc<Mutex<Option<WebManifest>>>,
    workers: Arc<Mutex<Vec<ServiceWorker>>>,
    installable: Arc<Mutex<bool>>,
}

impl PwaManager {
    pub fn new() -> Self {
        Self {
            manifest: Arc::new(Mutex::new(None)),
            workers: Arc::new(Mutex::new(Vec::new())),
            installable: Arc::new(Mutex::new(false)),
        }
    }

    /// Detecta manifest en HTML
    pub fn detect_manifest(&self, html: &str) -> Option<String> {
        // Busca <link rel="manifest" href="...">
        let lower = html.to_lowercase();
        let pos = lower.find("rel=\"manifest\"")?;
        let after = &html[pos..];
        let href_pos = after.find("href=")?;
        let rest = &after[href_pos + 5..];
        let rest = rest.trim_start().trim_start_matches('\'').trim_start_matches('"');
        let end = rest.find(|c: char| c == '"' || c == '\'' || c == '>')?;
        Some(rest[..end].to_string())
    }

    /// Registra manifest
    pub fn set_manifest(&self, manifest: WebManifest) {
        *self.manifest.lock().unwrap() = Some(manifest);
        *self.installable.lock().unwrap() = true;
    }

    /// Obtiene manifest actual
    pub fn get_manifest(&self) -> Option<WebManifest> {
        self.manifest.lock().unwrap().clone()
    }

    /// Registra un service worker
    pub fn register_worker(&self, script_url: &str, scope: &str) -> u64 {
        let mut workers = self.workers.lock().unwrap();
        let id = workers.len() as u64 + 1;
        workers.push(ServiceWorker {
            scope: scope.to_string(),
            script_url: script_url.to_string(),
            state: ServiceWorkerState::Installing,
            registrations: vec![ServiceWorkerRegistration {
                id,
                scope: scope.to_string(),
                script_url: script_url.to_string(),
                registered_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            }],
        });
        id
    }

    /// Actualiza estado de un service worker
    pub fn update_worker_state(&self, scope: &str, state: ServiceWorkerState) {
        let mut workers = self.workers.lock().unwrap();
        if let Some(w) = workers.iter_mut().find(|w| w.scope == scope) {
            w.state = state;
        }
    }

    /// Verifica si una página es instalable como PWA
    pub fn is_installable(&self) -> bool {
        *self.installable.lock().unwrap()
    }

    /// Lista service workers
    pub fn workers(&self) -> Vec<ServiceWorker> {
        self.workers.lock().unwrap().clone()
    }
}

impl Default for PwaManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_from_json() {
        let json = r##"{
            "name": "Test App",
            "short_name": "Test",
            "start_url": "/",
            "display": "standalone",
            "background_color": "#fff",
            "theme_color": "#000"
        }"##;
        let m = WebManifest::from_json(json).unwrap();
        assert_eq!(m.name, "Test App");
        assert_eq!(m.short_name, "Test");
        assert_eq!(m.start_url, "/");
    }

    #[test]
    fn test_manifest_default() {
        let m = WebManifest::default_for("https://test.com/");
        assert_eq!(m.name, "Noir App");
        assert_eq!(m.start_url, "https://test.com/");
    }

    #[test]
    fn test_manifest_to_json() {
        let m = WebManifest::default_for("https://test.com/");
        let json = m.to_json();
        assert!(json.contains("Noir App"));
        assert!(json.contains("https://test.com/"));
    }

    #[test]
    fn test_extract_json_string() {
        let json = r#"{"name": "Test", "age": 30}"#;
        assert_eq!(extract_json_string(json, "name"), Some("Test".to_string()));
    }

    #[test]
    fn test_pwa_manager_new() {
        let m = PwaManager::new();
        assert!(!m.is_installable());
        assert_eq!(m.workers().len(), 0);
    }

    #[test]
    fn test_pwa_detect_manifest() {
        let m = PwaManager::new();
        let html = r#"<html><head><link rel="manifest" href="/manifest.json"></head></html>"#;
        let detected = m.detect_manifest(html);
        assert_eq!(detected, Some("/manifest.json".to_string()));
    }

    #[test]
    fn test_pwa_set_manifest() {
        let m = PwaManager::new();
        let manifest = WebManifest::default_for("https://test.com/");
        m.set_manifest(manifest);
        assert!(m.is_installable());
    }

    #[test]
    fn test_pwa_register_worker() {
        let m = PwaManager::new();
        let id = m.register_worker("/sw.js", "/");
        assert!(id > 0);
        assert_eq!(m.workers().len(), 1);
    }

    #[test]
    fn test_pwa_update_worker_state() {
        let m = PwaManager::new();
        m.register_worker("/sw.js", "/");
        m.update_worker_state("/", ServiceWorkerState::Activated);
        assert_eq!(m.workers()[0].state, ServiceWorkerState::Activated);
    }
}
