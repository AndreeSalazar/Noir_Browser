//! History - Historial de navegación persistente

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use super::path::StoragePaths;

const MAX_HISTORY: usize = 10000;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HistoryEntry {
    pub id: u64,
    pub url: String,
    pub title: String,
    pub visited: u64,
    pub visit_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HistoryError {
    IoError(String),
    JsonError(String),
}

impl std::fmt::Display for HistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HistoryError::IoError(s) => write!(f, "I/O: {}", s),
            HistoryError::JsonError(s) => write!(f, "JSON: {}", s),
        }
    }
}

impl std::error::Error for HistoryError {}

pub struct HistoryManager {
    entries: Arc<Mutex<Vec<HistoryEntry>>>,
    file_path: PathBuf,
}

impl HistoryManager {
    pub fn new(paths: &StoragePaths) -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
            file_path: paths.history_file(),
        }
    }

    pub fn load(&self) -> Result<usize, HistoryError> {
        if !self.file_path.exists() {
            return Ok(0);
        }
        let content = fs::read_to_string(&self.file_path)
            .map_err(|e| HistoryError::IoError(e.to_string()))?;
        let entries: Vec<HistoryEntry> = serde_json::from_str(&content)
            .map_err(|e| HistoryError::JsonError(e.to_string()))?;
        let count = entries.len();
        *self.entries.lock().unwrap() = entries;
        Ok(count)
    }

    pub fn save(&self) -> Result<(), HistoryError> {
        let entries = self.entries.lock().unwrap();
        let json = serde_json::to_string_pretty(&*entries)
            .map_err(|e| HistoryError::JsonError(e.to_string()))?;
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| HistoryError::IoError(e.to_string()))?;
        }
        fs::write(&self.file_path, json)
            .map_err(|e| HistoryError::IoError(e.to_string()))?;
        Ok(())
    }

    /// Registra una visita a una URL
    pub fn visit(&self, url: &str, title: &str) {
        let now = current_time_nanos();
        let mut entries = self.entries.lock().unwrap();

        if let Some(existing) = entries.iter_mut().find(|e| e.url == url) {
            existing.visit_count += 1;
            existing.visited = now;
            if !title.is_empty() {
                existing.title = title.to_string();
            }
        } else {
            entries.push(HistoryEntry {
                id: now,
                url: url.to_string(),
                title: title.to_string(),
                visited: now,
                visit_count: 1,
            });
        }

        // Sort by visited time (newest first)
        entries.sort_by(|a, b| b.visited.cmp(&a.visited));

        // Trim to MAX_HISTORY
        if entries.len() > MAX_HISTORY {
            entries.truncate(MAX_HISTORY);
        }
    }

    pub fn all(&self) -> Vec<HistoryEntry> {
        self.entries.lock().unwrap().clone()
    }

    pub fn recent(&self, limit: usize) -> Vec<HistoryEntry> {
        self.entries.lock().unwrap()
            .iter()
            .take(limit)
            .cloned()
            .collect()
    }

    pub fn search(&self, query: &str) -> Vec<HistoryEntry> {
        let lower = query.to_lowercase();
        self.entries.lock().unwrap()
            .iter()
            .filter(|e| {
                e.url.to_lowercase().contains(&lower)
                || e.title.to_lowercase().contains(&lower)
            })
            .cloned()
            .collect()
    }

    pub fn top_sites(&self, limit: usize) -> Vec<(&str, u32)> {
        let entries = self.entries.lock().unwrap();
        let mut sites: Vec<_> = entries.iter()
            .map(|e| (e.url.clone(), e.visit_count))
            .collect();
        sites.sort_by(|a, b| b.1.cmp(&a.1));
        sites.into_iter()
            .take(limit)
            .map(|(url, count)| (url.leak() as &str, count))
            .collect()
    }

    pub fn clear(&self) {
        self.entries.lock().unwrap().clear();
    }

    pub fn count(&self) -> usize {
        self.entries.lock().unwrap().len()
    }

    pub fn stats(&self) -> HashMap<String, usize> {
        let entries = self.entries.lock().unwrap();
        let mut stats = HashMap::new();
        stats.insert("total".to_string(), entries.len());
        stats.insert("unique".to_string(), entries.len());
        let mut total_visits = 0u32;
        for e in entries.iter() {
            total_visits += e.visit_count;
        }
        stats.insert("visits".to_string(), total_visits as usize);
        stats
    }
}

fn current_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn current_time_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_paths() -> StoragePaths {
        let mut paths = StoragePaths::new();
        paths.data_dir = std::env::temp_dir().join(format!("noir-history-test-{}", current_time()));
        paths.ensure_dir().unwrap();
        paths
    }

    #[test]
    fn test_history_visit_new() {
        let paths = temp_paths();
        let m = HistoryManager::new(&paths);
        m.visit("https://example.com", "Example");
        assert_eq!(m.count(), 1);
    }

    #[test]
    fn test_history_visit_existing() {
        let paths = temp_paths();
        let m = HistoryManager::new(&paths);
        m.visit("https://example.com", "Example");
        m.visit("https://example.com", "Example");
        let entries = m.all();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].visit_count, 2);
    }

    #[test]
    fn test_history_search() {
        let paths = temp_paths();
        let m = HistoryManager::new(&paths);
        m.visit("https://rust-lang.org", "Rust");
        m.visit("https://golang.org", "Go");
        let results = m.search("rust");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_history_recent() {
        let paths = temp_paths();
        let m = HistoryManager::new(&paths);
        m.visit("https://a.com", "A");
        m.visit("https://b.com", "B");
        m.visit("https://c.com", "C");
        let recent = m.recent(2);
        assert_eq!(recent.len(), 2);
        // Most recent 2 (order may vary if timestamps are equal)
        let urls: Vec<&str> = recent.iter().map(|e| e.url.as_str()).collect();
        assert!(urls.contains(&"https://b.com"));
        assert!(urls.contains(&"https://c.com"));
    }

    #[test]
    fn test_history_save_load() {
        let paths = temp_paths();
        let m1 = HistoryManager::new(&paths);
        m1.visit("https://a.com", "A");
        m1.visit("https://b.com", "B");
        m1.save().unwrap();

        let m2 = HistoryManager::new(&paths);
        m2.load().unwrap();
        assert_eq!(m2.count(), 2);
    }

    #[test]
    fn test_history_clear() {
        let paths = temp_paths();
        let m = HistoryManager::new(&paths);
        m.visit("https://a.com", "A");
        m.clear();
        assert_eq!(m.count(), 0);
    }

    #[test]
    fn test_history_stats() {
        let paths = temp_paths();
        let m = HistoryManager::new(&paths);
        m.visit("https://a.com", "A");
        m.visit("https://a.com", "A");
        let stats = m.stats();
        // total = unique URLs = 1
        assert_eq!(stats.get("total"), Some(&1));
        // visits = sum of visit_count = 2
        assert_eq!(stats.get("visits"), Some(&2));
    }
}
