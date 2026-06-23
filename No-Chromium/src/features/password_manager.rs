//! Password Manager - Guardar/auto-fill passwords

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::storage::path::StoragePaths;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PasswordEntry {
    pub username: String,
    pub password: String,
    pub domain: String,
    pub created: u64,
    pub last_used: u64,
    pub use_count: u32,
}

/// Password guardado
#[derive(Debug, Clone)]
pub struct SavedPassword {
    pub id: u64,
    pub site: String,
    pub username: String,
    pub created: u64,
}

pub struct PasswordManager {
    entries: Arc<Mutex<HashMap<String, PasswordEntry>>>,
    file_path: std::path::PathBuf,
}

impl PasswordManager {
    pub fn new(paths: &StoragePaths) -> Self {
        let mut path = paths.data_dir.clone();
        path.push("passwords.json");
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
            file_path: path,
        }
    }

    /// Guarda una credencial
    pub fn save(&self, domain: &str, username: &str, password: &str) -> bool {
        let now = current_time();
        let entry = PasswordEntry {
            username: username.to_string(),
            password: password.to_string(),
            domain: domain.to_string(),
            created: now,
            last_used: now,
            use_count: 1,
        };
        self.entries.lock().unwrap().insert(domain.to_string(), entry);
        self.save_to_disk().is_ok()
    }

    /// Obtiene password para un dominio
    pub fn get(&self, domain: &str) -> Option<PasswordEntry> {
        let mut entries = self.entries.lock().unwrap();
        if let Some(entry) = entries.get_mut(domain) {
            entry.last_used = current_time();
            entry.use_count += 1;
            Some(entry.clone())
        } else {
            None
        }
    }

    /// Elimina password
    pub fn remove(&self, domain: &str) -> bool {
        self.entries.lock().unwrap().remove(domain).is_some()
    }

    /// Lista todos los dominios guardados (sin password)
    pub fn list(&self) -> Vec<SavedPassword> {
        self.entries.lock().unwrap().iter().enumerate().map(|(i, (k, v))| {
            SavedPassword {
                id: i as u64 + 1,
                site: k.clone(),
                username: v.username.clone(),
                created: v.created,
            }
        }).collect()
    }

    /// Cuenta total
    pub fn count(&self) -> usize {
        self.entries.lock().unwrap().len()
    }

    /// Genera un password aleatorio
    pub fn generate_password(length: usize) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
        let mut password = String::new();
        for i in 0..length {
            let mut hasher = DefaultHasher::new();
            current_time().hash(&mut hasher);
            i.hash(&mut hasher);
            let h = hasher.finish();
            let idx = (h as usize) % chars.len();
            password.push(chars.chars().nth(idx).unwrap());
        }
        password
    }

    /// Carga desde disco
    pub fn load(&self) -> Result<(), String> {
        if !self.file_path.exists() {
            return Ok(());
        }
        let content = std::fs::read_to_string(&self.file_path)
            .map_err(|e| e.to_string())?;
        let loaded: HashMap<String, PasswordEntry> = serde_json::from_str(&content)
            .map_err(|e| e.to_string())?;
        *self.entries.lock().unwrap() = loaded;
        Ok(())
    }

    /// Guarda a disco
    pub fn save_to_disk(&self) -> Result<(), String> {
        if let Some(parent) = self.file_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let entries = self.entries.lock().unwrap();
        let json = serde_json::to_string_pretty(&*entries)
            .map_err(|e| e.to_string())?;
        std::fs::write(&self.file_path, json).map_err(|e| e.to_string())?;
        Ok(())
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
    use crate::storage::path::StoragePaths;

    fn temp_paths() -> StoragePaths {
        use std::time::{SystemTime, UNIX_EPOCH};
        let mut paths = StoragePaths::new();
        let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        paths.data_dir = std::env::temp_dir().join(format!("noir-passwords-test-{}", t));
        paths.ensure_dir().unwrap();
        paths
    }

    #[test]
    fn test_password_manager_creation() {
        let m = PasswordManager::new(&temp_paths());
        assert_eq!(m.count(), 0);
    }

    #[test]
    fn test_save_and_get() {
        let m = PasswordManager::new(&temp_paths());
        m.save("example.com", "user@test.com", "secret123");
        let entry = m.get("example.com").unwrap();
        assert_eq!(entry.username, "user@test.com");
        assert_eq!(entry.password, "secret123");
    }

    #[test]
    fn test_use_count() {
        let m = PasswordManager::new(&temp_paths());
        m.save("example.com", "user", "pass");
        m.get("example.com");
        m.get("example.com");
        let entry = m.get("example.com").unwrap();
        assert_eq!(entry.use_count, 4); // 1 initial + 3 gets
    }

    #[test]
    fn test_remove() {
        let m = PasswordManager::new(&temp_paths());
        m.save("example.com", "user", "pass");
        assert!(m.remove("example.com"));
        assert!(m.get("example.com").is_none());
    }

    #[test]
    fn test_list() {
        let m = PasswordManager::new(&temp_paths());
        m.save("example.com", "u1", "p1");
        m.save("test.com", "u2", "p2");
        let list = m.list();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_generate_password() {
        let pwd = PasswordManager::generate_password(16);
        assert_eq!(pwd.len(), 16);
    }

    #[test]
    fn test_save_load_disk() {
        let paths = temp_paths();
        let m1 = PasswordManager::new(&paths);
        m1.save("example.com", "user", "pass");
        m1.save_to_disk().unwrap();

        let m2 = PasswordManager::new(&paths);
        m2.load().unwrap();
        let entry = m2.get("example.com").unwrap();
        assert_eq!(entry.username, "user");
    }
}
