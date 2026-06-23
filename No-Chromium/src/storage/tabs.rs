//! Tabs Persistence - Guardar/cargar estado de tabs

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::path::StoragePaths;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TabSnapshot {
    pub url: String,
    pub title: String,
    pub scroll_y: f32,
    pub active: bool,
    pub created: u64,
}

pub struct TabPersistence {
    file_path: PathBuf,
}

impl TabPersistence {
    pub fn new(paths: &StoragePaths) -> Self {
        Self {
            file_path: paths.tabs_file(),
        }
    }

    pub fn save(&self, tabs: &[TabSnapshot]) -> Result<(), String> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(tabs).map_err(|e| e.to_string())?;
        fs::write(&self.file_path, json).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load(&self) -> Result<Vec<TabSnapshot>, String> {
        if !self.file_path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&self.file_path).map_err(|e| e.to_string())?;
        let tabs: Vec<TabSnapshot> = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        Ok(tabs)
    }

    pub fn clear(&self) -> Result<(), String> {
        if self.file_path.exists() {
            fs::remove_file(&self.file_path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

impl TabSnapshot {
    pub fn new(url: String, title: String) -> Self {
        Self {
            url,
            title,
            scroll_y: 0.0,
            active: false,
            created: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_paths() -> StoragePaths {
        use std::time::{SystemTime, UNIX_EPOCH};
        let mut paths = StoragePaths::new();
        let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        paths.data_dir = std::env::temp_dir().join(format!("noir-tabs-test-{}", t));
        paths.ensure_dir().unwrap();
        paths
    }

    #[test]
    fn test_tab_snapshot_creation() {
        let tab = TabSnapshot::new("https://test.com".to_string(), "Test".to_string());
        assert_eq!(tab.url, "https://test.com");
        assert!(!tab.active);
    }

    #[test]
    fn test_tabs_save_load() {
        let paths = temp_paths();
        let p1 = TabPersistence::new(&paths);
        let tabs = vec![
            TabSnapshot::new("https://a.com".to_string(), "A".to_string()),
            TabSnapshot::new("https://b.com".to_string(), "B".to_string()),
        ];
        p1.save(&tabs).unwrap();

        let p2 = TabPersistence::new(&paths);
        let loaded = p2.load().unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].url, "https://a.com");
    }

    #[test]
    fn test_tabs_clear() {
        let paths = temp_paths();
        let p = TabPersistence::new(&paths);
        let tabs = vec![TabSnapshot::new("https://a.com".to_string(), "A".to_string())];
        p.save(&tabs).unwrap();
        p.clear().unwrap();
        let loaded = p.load().unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_tabs_load_empty() {
        let paths = temp_paths();
        let p = TabPersistence::new(&paths);
        let loaded = p.load().unwrap();
        assert!(loaded.is_empty());
    }
}
