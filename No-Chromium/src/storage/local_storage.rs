//! localStorage API - Persistencia web por origen

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::path::StoragePaths;

const MAX_SIZE_PER_ORIGIN: usize = 5 * 1024 * 1024; // 5MB

pub struct LocalStorage {
    stores: Arc<Mutex<HashMap<String, HashMap<String, String>>>>,
    base_dir: PathBuf,
}

impl LocalStorage {
    pub fn new(paths: &StoragePaths) -> Self {
        let base = paths.local_storage_dir();
        let _ = fs::create_dir_all(&base);
        Self {
            stores: Arc::new(Mutex::new(HashMap::new())),
            base_dir: base,
        }
    }

    pub fn load(&self) {
        let _ = fs::create_dir_all(&self.base_dir);
        if let Ok(entries) = fs::read_dir(&self.base_dir) {
            let mut stores = self.stores.lock().unwrap();
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".json") {
                        let origin = name.trim_end_matches(".json");
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&content) {
                                stores.insert(origin.to_string(), map);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn save_origin(&self, origin: &str) {
        let stores = self.stores.lock().unwrap();
        if let Some(map) = stores.get(origin) {
            if let Ok(json) = serde_json::to_string_pretty(map) {
                let path = self.base_dir.join(format!("{}.json", sanitize_filename(origin)));
                let _ = fs::write(path, json);
            }
        }
    }

    pub fn get(&self, origin: &str, key: &str) -> Option<String> {
        let stores = self.stores.lock().unwrap();
        stores.get(origin)?.get(key).cloned()
    }

    pub fn set(&self, origin: &str, key: String, value: String) -> Result<(), String> {
        let mut stores = self.stores.lock().unwrap();
        let map = stores.entry(origin.to_string()).or_insert_with(HashMap::new);

        // Check size
        let total_size: usize = map.iter()
            .map(|(k, v)| k.len() + v.len())
            .sum();
        if total_size + key.len() + value.len() > MAX_SIZE_PER_ORIGIN {
            return Err("Quota exceeded".to_string());
        }

        map.insert(key, value);
        Ok(())
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

    pub fn keys(&self, origin: &str) -> Vec<String> {
        let stores = self.stores.lock().unwrap();
        stores.get(origin)
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default()
    }

    pub fn length(&self, origin: &str) -> usize {
        let stores = self.stores.lock().unwrap();
        stores.get(origin).map(|m| m.len()).unwrap_or(0)
    }

    pub fn clear_all(&self) {
        self.stores.lock().unwrap().clear();
    }
}

fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_' => c,
            _ => '_',
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_paths() -> StoragePaths {
        use std::time::{SystemTime, UNIX_EPOCH};
        let mut paths = StoragePaths::new();
        let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        paths.data_dir = std::env::temp_dir().join(format!("noir-localstorage-test-{}", t));
        paths.ensure_dir().unwrap();
        paths
    }

    #[test]
    fn test_local_storage_set_get() {
        let paths = temp_paths();
        let ls = LocalStorage::new(&paths);
        ls.set("https://example.com", "key1".to_string(), "value1".to_string()).unwrap();
        assert_eq!(ls.get("https://example.com", "key1"), Some("value1".to_string()));
    }

    #[test]
    fn test_local_storage_remove() {
        let paths = temp_paths();
        let ls = LocalStorage::new(&paths);
        ls.set("https://test.com", "key".to_string(), "val".to_string()).unwrap();
        ls.remove("https://test.com", "key");
        assert_eq!(ls.get("https://test.com", "key"), None);
    }

    #[test]
    fn test_local_storage_clear() {
        let paths = temp_paths();
        let ls = LocalStorage::new(&paths);
        ls.set("https://a.com", "k".to_string(), "v".to_string()).unwrap();
        ls.clear("https://a.com");
        assert_eq!(ls.length("https://a.com"), 0);
    }

    #[test]
    fn test_local_storage_isolation() {
        let paths = temp_paths();
        let ls = LocalStorage::new(&paths);
        ls.set("https://a.com", "k".to_string(), "1".to_string()).unwrap();
        ls.set("https://b.com", "k".to_string(), "2".to_string()).unwrap();
        assert_eq!(ls.get("https://a.com", "k"), Some("1".to_string()));
        assert_eq!(ls.get("https://b.com", "k"), Some("2".to_string()));
    }

    #[test]
    fn test_local_storage_keys() {
        let paths = temp_paths();
        let ls = LocalStorage::new(&paths);
        ls.set("https://x.com", "a".to_string(), "1".to_string()).unwrap();
        ls.set("https://x.com", "b".to_string(), "2".to_string()).unwrap();
        let keys = ls.keys("https://x.com");
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_local_storage_length() {
        let paths = temp_paths();
        let ls = LocalStorage::new(&paths);
        assert_eq!(ls.length("https://y.com"), 0);
        ls.set("https://y.com", "k".to_string(), "v".to_string()).unwrap();
        assert_eq!(ls.length("https://y.com"), 1);
    }
}
