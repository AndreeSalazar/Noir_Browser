//! Bookmarks - Marcadores persistentes
//!
//! Guarda/carga marcadores desde JSON en disco.

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use super::path::StoragePaths;

/// Un marcador
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Bookmark {
    pub id: u64,
    pub title: String,
    pub url: String,
    pub folder: String,
    pub created: u64,
    pub favicon: Option<String>,
}

impl Bookmark {
    pub fn new(title: String, url: String) -> Self {
        Self {
            id: current_time(),
            title,
            url,
            folder: "default".to_string(),
            created: current_time(),
            favicon: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BookmarkError {
    IoError(String),
    JsonError(String),
    NotFound,
}

impl std::fmt::Display for BookmarkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BookmarkError::IoError(s) => write!(f, "I/O error: {}", s),
            BookmarkError::JsonError(s) => write!(f, "JSON error: {}", s),
            BookmarkError::NotFound => write!(f, "Bookmark not found"),
        }
    }
}

impl std::error::Error for BookmarkError {}

/// Manager de bookmarks
pub struct BookmarkManager {
    bookmarks: Arc<Mutex<Vec<Bookmark>>>,
    file_path: PathBuf,
}

impl BookmarkManager {
    pub fn new(paths: &StoragePaths) -> Self {
        Self {
            bookmarks: Arc::new(Mutex::new(Vec::new())),
            file_path: paths.bookmarks_file(),
        }
    }

    /// Carga bookmarks desde disco
    pub fn load(&self) -> Result<usize, BookmarkError> {
        if !self.file_path.exists() {
            return Ok(0);
        }
        let content = fs::read_to_string(&self.file_path)
            .map_err(|e| BookmarkError::IoError(e.to_string()))?;
        let bookmarks: Vec<Bookmark> = serde_json::from_str(&content)
            .map_err(|e| BookmarkError::JsonError(e.to_string()))?;
        let count = bookmarks.len();
        *self.bookmarks.lock().unwrap() = bookmarks;
        Ok(count)
    }

    /// Guarda bookmarks a disco
    pub fn save(&self) -> Result<(), BookmarkError> {
        let bookmarks = self.bookmarks.lock().unwrap();
        let json = serde_json::to_string_pretty(&*bookmarks)
            .map_err(|e| BookmarkError::JsonError(e.to_string()))?;
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| BookmarkError::IoError(e.to_string()))?;
        }
        fs::write(&self.file_path, json)
            .map_err(|e| BookmarkError::IoError(e.to_string()))?;
        Ok(())
    }

    /// Añade un bookmark
    pub fn add(&self, title: String, url: String) -> u64 {
        let bookmark = Bookmark::new(title, url);
        let id = bookmark.id;
        self.bookmarks.lock().unwrap().push(bookmark);
        id
    }

    /// Elimina un bookmark por ID
    pub fn remove(&self, id: u64) -> Result<(), BookmarkError> {
        let mut bookmarks = self.bookmarks.lock().unwrap();
        let pos = bookmarks.iter().position(|b| b.id == id)
            .ok_or(BookmarkError::NotFound)?;
        bookmarks.remove(pos);
        Ok(())
    }

    /// Obtiene todos los bookmarks
    pub fn all(&self) -> Vec<Bookmark> {
        self.bookmarks.lock().unwrap().clone()
    }

    /// Busca por URL
    pub fn find_by_url(&self, url: &str) -> Option<Bookmark> {
        self.bookmarks.lock().unwrap()
            .iter()
            .find(|b| b.url == url)
            .cloned()
    }

    /// Cuenta total
    pub fn count(&self) -> usize {
        self.bookmarks.lock().unwrap().len()
    }

    /// Busca por título
    pub fn search(&self, query: &str) -> Vec<Bookmark> {
        let lower = query.to_lowercase();
        self.bookmarks.lock().unwrap()
            .iter()
            .filter(|b| {
                b.title.to_lowercase().contains(&lower)
                || b.url.to_lowercase().contains(&lower)
            })
            .cloned()
            .collect()
    }
}

fn current_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_paths() -> StoragePaths {
        let mut paths = StoragePaths::new();
        paths.data_dir = std::env::temp_dir().join(format!("noir-bookmarks-test-{}", current_time()));
        paths.ensure_dir().unwrap();
        paths
    }

    #[test]
    fn test_bookmark_creation() {
        let b = Bookmark::new("Test".to_string(), "https://test.com".to_string());
        assert_eq!(b.title, "Test");
        assert_eq!(b.url, "https://test.com");
    }

    #[test]
    fn test_bookmark_manager_add() {
        let paths = temp_paths();
        let m = BookmarkManager::new(&paths);
        let id = m.add("Example".to_string(), "https://example.com".to_string());
        assert_eq!(m.count(), 1);
        assert!(id > 0);
    }

    #[test]
    fn test_bookmark_manager_remove() {
        let paths = temp_paths();
        let m = BookmarkManager::new(&paths);
        let id = m.add("Test".to_string(), "https://test.com".to_string());
        m.remove(id).unwrap();
        assert_eq!(m.count(), 0);
    }

    #[test]
    fn test_bookmark_manager_find_by_url() {
        let paths = temp_paths();
        let m = BookmarkManager::new(&paths);
        m.add("Test".to_string(), "https://test.com".to_string());
        let found = m.find_by_url("https://test.com");
        assert!(found.is_some());
    }

    #[test]
    fn test_bookmark_manager_search() {
        let paths = temp_paths();
        let m = BookmarkManager::new(&paths);
        m.add("Rust Lang".to_string(), "https://rust-lang.org".to_string());
        m.add("Go Lang".to_string(), "https://golang.org".to_string());
        let results = m.search("rust");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_bookmark_save_load() {
        let paths = temp_paths();
        let m1 = BookmarkManager::new(&paths);
        m1.add("Example".to_string(), "https://example.com".to_string());
        m1.add("Rust".to_string(), "https://rust-lang.org".to_string());
        m1.save().unwrap();

        let m2 = BookmarkManager::new(&paths);
        let count = m2.load().unwrap();
        assert_eq!(count, 2);
        assert_eq!(m2.count(), 2);
    }

    #[test]
    fn test_bookmark_load_empty() {
        let paths = temp_paths();
        let m = BookmarkManager::new(&paths);
        let count = m.load().unwrap();
        assert_eq!(count, 0);
    }
}
