//! Service Worker - Registro, ciclo de vida, fetch event handler
//!
//! Simula un service worker para caching offline y push notifications.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WorkerState {
    Installing,
    Installed,
    Activating,
    Activated,
    Redundant,
}

#[derive(Debug, Clone)]
pub struct WorkerRegistration {
    pub id: u32,
    pub scope: String,
    pub script_url: String,
    pub state: WorkerState,
    pub registered_at: u64,
    pub last_update: u64,
}

impl WorkerRegistration {
    pub fn new(id: u32, scope: &str, script_url: &str) -> Self {
        let now = current_time();
        Self {
            id,
            scope: scope.to_string(),
            script_url: script_url.to_string(),
            state: WorkerState::Installed,
            registered_at: now,
            last_update: now,
        }
    }

    pub fn activate(&mut self) {
        self.state = WorkerState::Activated;
    }

    pub fn unregister(&mut self) {
        self.state = WorkerState::Redundant;
    }

    pub fn is_active(&self) -> bool {
        self.state == WorkerState::Activated
    }
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub url: String,
    pub response: String,
    pub content_type: String,
    pub cached_at: u64,
    pub expires_at: u64,
}

impl CacheEntry {
    pub fn is_expired(&self) -> bool {
        current_time() > self.expires_at
    }
}

#[derive(Debug, Clone)]
pub struct PushMessage {
    pub id: u32,
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
    pub data: Option<String>,
    pub received_at: u64,
}

pub struct ServiceWorkerManager {
    registrations: HashMap<u32, WorkerRegistration>,
    caches: HashMap<String, Vec<CacheEntry>>,
    push_messages: Vec<PushMessage>,
    next_id: u32,
    next_msg_id: u32,
    cache_name_counter: u32,
}

impl ServiceWorkerManager {
    pub fn new() -> Self {
        Self {
            registrations: HashMap::new(),
            caches: HashMap::new(),
            push_messages: Vec::new(),
            next_id: 1,
            next_msg_id: 1,
            cache_name_counter: 1,
        }
    }

    /// Registra un service worker para un scope
    pub fn register(&mut self, scope: &str, script_url: &str) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let reg = WorkerRegistration::new(id, scope, script_url);
        self.registrations.insert(id, reg);
        id
    }

    pub fn unregister(&mut self, id: u32) -> bool {
        if let Some(reg) = self.registrations.get_mut(&id) {
            reg.unregister();
            true
        } else {
            false
        }
    }

    pub fn activate(&mut self, id: u32) -> bool {
        if let Some(reg) = self.registrations.get_mut(&id) {
            reg.activate();
            true
        } else {
            false
        }
    }

    pub fn get_registration(&self, id: u32) -> Option<&WorkerRegistration> {
        self.registrations.get(&id)
    }

    pub fn get_for_scope(&self, scope: &str) -> Option<&WorkerRegistration> {
        self.registrations.values().find(|r| r.scope == scope && r.is_active())
    }

    pub fn registrations(&self) -> Vec<&WorkerRegistration> {
        self.registrations.values().collect()
    }

    pub fn count(&self) -> usize {
        self.registrations.iter().filter(|(_, r)| r.is_active()).count()
    }

    /// Abre (crea) un cache
    pub fn open_cache(&mut self) -> String {
        let name = format!("cache-v{}", self.cache_name_counter);
        self.cache_name_counter += 1;
        self.caches.entry(name.clone()).or_default();
        name
    }

    /// Almacena un response en cache
    pub fn put(&mut self, cache_name: &str, url: &str, response: &str, content_type: &str, ttl_seconds: u64) {
        let entry = CacheEntry {
            url: url.to_string(),
            response: response.to_string(),
            content_type: content_type.to_string(),
            cached_at: current_time(),
            expires_at: current_time() + ttl_seconds,
        };
        self.caches.entry(cache_name.to_string()).or_default().push(entry);
    }

    /// Recupera un response de cache
    pub fn match_cache(&self, cache_name: &str, url: &str) -> Option<String> {
        self.caches.get(cache_name)
            .and_then(|entries| {
                entries.iter()
                    .find(|e| e.url == url && !e.is_expired())
                    .map(|e| e.response.clone())
            })
    }

    /// Elimina un cache
    pub fn delete_cache(&mut self, cache_name: &str) -> bool {
        self.caches.remove(cache_name).is_some()
    }

    /// Lista de caches
    pub fn cache_names(&self) -> Vec<String> {
        self.caches.keys().cloned().collect()
    }

    /// Empuja un mensaje (notificación)
    pub fn push(&mut self, title: &str, body: &str, icon: Option<&str>, data: Option<&str>) -> u32 {
        let id = self.next_msg_id;
        self.next_msg_id += 1;
        self.push_messages.push(PushMessage {
            id,
            title: title.to_string(),
            body: body.to_string(),
            icon: icon.map(|s| s.to_string()),
            data: data.map(|s| s.to_string()),
            received_at: current_time(),
        });
        id
    }

    /// Pop un mensaje (cuando se muestra)
    pub fn pop_push(&mut self) -> Option<PushMessage> {
        if self.push_messages.is_empty() {
            None
        } else {
            Some(self.push_messages.remove(0))
        }
    }

    pub fn push_count(&self) -> usize {
        self.push_messages.len()
    }

    pub fn total_registrations(&self) -> usize {
        self.registrations.len()
    }
}

impl Default for ServiceWorkerManager {
    fn default() -> Self { Self::new() }
}

fn current_time() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registration_new() {
        let r = WorkerRegistration::new(1, "/", "/sw.js");
        assert_eq!(r.id, 1);
        assert!(!r.is_active());
    }

    #[test]
    fn test_registration_activate() {
        let mut r = WorkerRegistration::new(1, "/", "/sw.js");
        r.activate();
        assert!(r.is_active());
    }

    #[test]
    fn test_registration_unregister() {
        let mut r = WorkerRegistration::new(1, "/", "/sw.js");
        r.unregister();
        assert!(!r.is_active());
    }

    #[test]
    fn test_manager_register() {
        let mut m = ServiceWorkerManager::new();
        let id = m.register("/", "/sw.js");
        assert!(m.get_registration(id).is_some());
    }

    #[test]
    fn test_manager_activate() {
        let mut m = ServiceWorkerManager::new();
        let id = m.register("/", "/sw.js");
        assert!(m.activate(id));
        assert!(m.get_registration(id).unwrap().is_active());
    }

    #[test]
    fn test_manager_unregister() {
        let mut m = ServiceWorkerManager::new();
        let id = m.register("/", "/sw.js");
        assert!(m.unregister(id));
    }

    #[test]
    fn test_manager_count() {
        let mut m = ServiceWorkerManager::new();
        m.register("/", "/sw1.js");
        m.register("/app", "/sw2.js");
        assert_eq!(m.total_registrations(), 2);
    }

    #[test]
    fn test_cache_put_match() {
        let mut m = ServiceWorkerManager::new();
        let cache = m.open_cache();
        m.put(&cache, "https://x.com/data", "response body", "text/html", 3600);
        assert_eq!(m.match_cache(&cache, "https://x.com/data"), Some("response body".to_string()));
    }

    #[test]
    fn test_cache_not_found() {
        let mut m = ServiceWorkerManager::new();
        let cache = m.open_cache();
        assert!(m.match_cache(&cache, "https://missing.com").is_none());
    }

    #[test]
    fn test_cache_delete() {
        let mut m = ServiceWorkerManager::new();
        let cache = m.open_cache();
        assert!(m.delete_cache(&cache));
        assert!(!m.delete_cache(&cache));
    }

    #[test]
    fn test_cache_names() {
        let mut m = ServiceWorkerManager::new();
        m.open_cache();
        m.open_cache();
        m.open_cache();
        assert_eq!(m.cache_names().len(), 3);
    }

    #[test]
    fn test_push() {
        let mut m = ServiceWorkerManager::new();
        let id = m.push("Title", "Body", Some("/icon.png"), Some("data"));
        assert_eq!(id, 1);
        assert_eq!(m.push_count(), 1);
    }

    #[test]
    fn test_pop_push() {
        let mut m = ServiceWorkerManager::new();
        m.push("T1", "B1", None, None);
        m.push("T2", "B2", None, None);
        let msg = m.pop_push().unwrap();
        assert_eq!(msg.title, "T1");
        assert_eq!(m.push_count(), 1);
    }

    #[test]
    fn test_pop_empty() {
        let mut m = ServiceWorkerManager::new();
        assert!(m.pop_push().is_none());
    }

    #[test]
    fn test_get_for_scope() {
        let mut m = ServiceWorkerManager::new();
        m.register("/app", "/sw1.js");
        m.register("/api", "/sw2.js");
        let id = m.register("/app2", "/sw3.js");
        m.activate(id);
        assert!(m.get_for_scope("/app2").is_some());
    }

    #[test]
    fn test_cache_multiple_entries() {
        let mut m = ServiceWorkerManager::new();
        let cache = m.open_cache();
        m.put(&cache, "u1", "r1", "text/html", 60);
        m.put(&cache, "u2", "r2", "text/css", 60);
        m.put(&cache, "u3", "r3", "application/json", 60);
        assert_eq!(m.match_cache(&cache, "u1"), Some("r1".to_string()));
        assert_eq!(m.match_cache(&cache, "u2"), Some("r2".to_string()));
        assert_eq!(m.match_cache(&cache, "u3"), Some("r3".to_string()));
    }
}
