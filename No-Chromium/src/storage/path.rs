//! Storage Paths - Rutas de archivos para persistencia
//!
//! Centraliza todas las rutas de archivos de almacenamiento.
//! Usa el directorio de datos del usuario.

use std::path::PathBuf;

/// Rutas de archivos de almacenamiento
#[derive(Debug, Clone)]
pub struct StoragePaths {
    pub data_dir: PathBuf,
}

impl StoragePaths {
    /// Crea las rutas usando el directorio de datos del SO
    pub fn new() -> Self {
        let data_dir = Self::default_data_dir();
        Self { data_dir }
    }

    fn default_data_dir() -> PathBuf {
        // Windows: %APPDATA%/NoirBrowser
        // Linux: ~/.local/share/noir-browser
        // macOS: ~/Library/Application Support/Noir Browser
        #[cfg(target_os = "windows")]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                let mut path = PathBuf::from(appdata);
                path.push("NoirBrowser");
                return path;
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(home) = std::env::var("HOME") {
                let mut path = PathBuf::from(home);
                path.push(".local/share/noir-browser");
                return path;
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(home) = std::env::var("HOME") {
                let mut path = PathBuf::from(home);
                path.push("Library/Application Support/Noir Browser");
                return path;
            }
        }

        // Fallback
        PathBuf::from("./noir-browser-data")
    }

    /// Crea el directorio si no existe
    pub fn ensure_dir(&self) -> std::io::Result<()> {
        if !self.data_dir.exists() {
            std::fs::create_dir_all(&self.data_dir)?;
        }
        Ok(())
    }

    pub fn bookmarks_file(&self) -> PathBuf {
        let mut p = self.data_dir.clone();
        p.push("bookmarks.json");
        p
    }

    pub fn history_file(&self) -> PathBuf {
        let mut p = self.data_dir.clone();
        p.push("history.json");
        p
    }

    pub fn settings_file(&self) -> PathBuf {
        let mut p = self.data_dir.clone();
        p.push("settings.json");
        p
    }

    pub fn tabs_file(&self) -> PathBuf {
        let mut p = self.data_dir.clone();
        p.push("session.json");
        p
    }

    pub fn local_storage_dir(&self) -> PathBuf {
        let mut p = self.data_dir.clone();
        p.push("local_storage");
        p
    }

    pub fn downloads_dir(&self) -> PathBuf {
        let mut p = self.data_dir.clone();
        p.push("downloads");
        p
    }
}

impl Default for StoragePaths {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_paths_creation() {
        let paths = StoragePaths::new();
        assert!(!paths.data_dir.as_os_str().is_empty());
    }

    #[test]
    fn test_storage_paths_file_names() {
        let paths = StoragePaths::new();
        assert!(paths.bookmarks_file().ends_with("bookmarks.json"));
        assert!(paths.history_file().ends_with("history.json"));
        assert!(paths.settings_file().ends_with("settings.json"));
    }

    #[test]
    fn test_storage_paths_ensure_dir() {
        let mut paths = StoragePaths::new();
        paths.data_dir = std::env::temp_dir().join("noir-test-storage");
        let result = paths.ensure_dir();
        assert!(result.is_ok());
        assert!(paths.data_dir.exists());
    }
}
