//! Settings - Configuración persistente del usuario

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::path::StoragePaths;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Theme {
    Dark,
    Light,
    System,
}

impl Theme {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "light" => Theme::Light,
            "system" => Theme::System,
            _ => Theme::Dark,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Dark => "dark",
            Theme::Light => "light",
            Theme::System => "system",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    pub theme: Theme,
    pub zoom: f32,
    pub homepage: String,
    pub search_engine: String,
    pub block_third_party_cookies: bool,
    pub enable_dnt: bool,
    pub enable_js: bool,
    pub enable_images: bool,
    pub enable_smooth_scroll: bool,
    pub user_agent: String,
    pub download_dir: String,
    pub font_size: u32,
    pub language: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            zoom: 1.0,
            homepage: "noir://newtab".to_string(),
            search_engine: "duckduckgo".to_string(),
            block_third_party_cookies: true,
            enable_dnt: true,
            enable_js: true,
            enable_images: true,
            enable_smooth_scroll: true,
            user_agent: "NoirBrowser/0.4 (Rust; no-chromium)".to_string(),
            download_dir: String::new(),
            font_size: 14,
            language: "es".to_string(),
        }
    }
}

pub struct SettingsManager {
    settings: Arc<Mutex<Settings>>,
    file_path: PathBuf,
}

impl SettingsManager {
    pub fn new(paths: &StoragePaths) -> Self {
        Self {
            settings: Arc::new(Mutex::new(Settings::default())),
            file_path: paths.settings_file(),
        }
    }

    pub fn load(&self) -> Result<(), String> {
        if !self.file_path.exists() {
            return Ok(());
        }
        let content = fs::read_to_string(&self.file_path)
            .map_err(|e| e.to_string())?;
        let loaded: Settings = serde_json::from_str(&content)
            .map_err(|e| e.to_string())?;
        *self.settings.lock().unwrap() = loaded;
        Ok(())
    }

    pub fn save(&self) -> Result<(), String> {
        let settings = self.settings.lock().unwrap();
        let json = serde_json::to_string_pretty(&*settings)
            .map_err(|e| e.to_string())?;
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(&self.file_path, json).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get(&self) -> Settings {
        self.settings.lock().unwrap().clone()
    }

    pub fn set(&self, new_settings: Settings) {
        *self.settings.lock().unwrap() = new_settings;
    }

    pub fn update<F: FnOnce(&mut Settings)>(&self, f: F) {
        let mut s = self.settings.lock().unwrap();
        f(&mut s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_paths() -> StoragePaths {
        use std::time::{SystemTime, UNIX_EPOCH};
        let mut paths = StoragePaths::new();
        let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        paths.data_dir = std::env::temp_dir().join(format!("noir-settings-test-{}", t));
        paths.ensure_dir().unwrap();
        paths
    }

    #[test]
    fn test_settings_default() {
        let s = Settings::default();
        assert_eq!(s.theme, Theme::Dark);
        assert_eq!(s.zoom, 1.0);
        assert!(s.enable_js);
    }

    #[test]
    fn test_theme_parsing() {
        assert_eq!(Theme::from_str("dark"), Theme::Dark);
        assert_eq!(Theme::from_str("light"), Theme::Light);
        assert_eq!(Theme::from_str("system"), Theme::System);
        assert_eq!(Theme::from_str("invalid"), Theme::Dark);
    }

    #[test]
    fn test_settings_save_load() {
        let paths = temp_paths();
        let m1 = SettingsManager::new(&paths);
        m1.update(|s| {
            s.theme = Theme::Light;
            s.zoom = 1.5;
        });
        m1.save().unwrap();

        let m2 = SettingsManager::new(&paths);
        m2.load().unwrap();
        let loaded = m2.get();
        assert_eq!(loaded.theme, Theme::Light);
        assert_eq!(loaded.zoom, 1.5);
    }

    #[test]
    fn test_settings_load_default_if_not_exists() {
        let paths = temp_paths();
        let m = SettingsManager::new(&paths);
        m.load().unwrap();
        let s = m.get();
        assert_eq!(s.theme, Theme::Dark);
    }
}
