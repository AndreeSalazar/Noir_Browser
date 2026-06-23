//! sessionStorage API - Persistencia web por sesión (no persiste)

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct SessionStorage {
    stores: Arc<Mutex<HashMap<String, HashMap<String, String>>>>,
}

impl SessionStorage {
    pub fn new() -> Self {
        Self {
            stores: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&self, origin: &str, key: &str) -> Option<String> {
        let stores = self.stores.lock().unwrap();
        stores.get(origin)?.get(key).cloned()
    }

    pub fn set(&self, origin: &str, key: String, value: String) {
        let mut stores = self.stores.lock().unwrap();
        stores.entry(origin.to_string())
            .or_insert_with(HashMap::new)
            .insert(key, value);
    }

    pub fn remove(&self, origin: &str, key: &str) {
        let mut stores = self.stores.lock().unwrap();
        if let Some(map) = stores.get_mut(origin) {
            map.remove(key);
        }
    }

    pub fn clear(&self, origin: &str) {
        let mut stores = self.stores.lock().unwrap();
        stores.remove(origin);
    }

    pub fn clear_all(&self) {
        self.stores.lock().unwrap().clear();
    }

    pub fn length(&self, origin: &str) -> usize {
        let stores = self.stores.lock().unwrap();
        stores.get(origin).map(|m| m.len()).unwrap_or(0)
    }

    pub fn keys(&self, origin: &str) -> Vec<String> {
        let stores = self.stores.lock().unwrap();
        stores.get(origin)
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default()
    }
}

impl Default for SessionStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_storage_set_get() {
        let ss = SessionStorage::new();
        ss.set("https://test.com", "key".to_string(), "value".to_string());
        assert_eq!(ss.get("https://test.com", "key"), Some("value".to_string()));
    }

    #[test]
    fn test_session_storage_remove() {
        let ss = SessionStorage::new();
        ss.set("https://a.com", "k".to_string(), "v".to_string());
        ss.remove("https://a.com", "k");
        assert_eq!(ss.get("https://a.com", "k"), None);
    }

    #[test]
    fn test_session_storage_clear_origin() {
        let ss = SessionStorage::new();
        ss.set("https://a.com", "k".to_string(), "v".to_string());
        ss.set("https://b.com", "k".to_string(), "v".to_string());
        ss.clear("https://a.com");
        assert_eq!(ss.length("https://a.com"), 0);
        assert_eq!(ss.length("https://b.com"), 1);
    }

    #[test]
    fn test_session_storage_clear_all() {
        let ss = SessionStorage::new();
        ss.set("https://a.com", "k".to_string(), "v".to_string());
        ss.clear_all();
        assert_eq!(ss.length("https://a.com"), 0);
    }

    #[test]
    fn test_session_storage_isolation() {
        let ss = SessionStorage::new();
        ss.set("https://a.com", "k".to_string(), "1".to_string());
        ss.set("https://b.com", "k".to_string(), "2".to_string());
        assert_eq!(ss.get("https://a.com", "k"), Some("1".to_string()));
        assert_eq!(ss.get("https://b.com", "k"), Some("2".to_string()));
    }
}
