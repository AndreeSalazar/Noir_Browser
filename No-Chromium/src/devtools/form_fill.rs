//! Form Fill - Auto-fill y autosave de formularios
//!
//! Recuerda valores ingresados en formularios para auto-rellenar después.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::storage::path::StoragePaths;

#[derive(Debug, Clone)]
pub struct FormField {
    pub name: String,
    pub field_type: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct FilledField {
    pub domain: String,
    pub field_name: String,
    pub value: String,
    pub use_count: u32,
    pub last_used: u64,
}

pub struct FormFillManager {
    entries: Arc<Mutex<HashMap<(String, String), FilledField>>>,
    file_path: std::path::PathBuf,
}

impl FormFillManager {
    pub fn new(paths: &StoragePaths) -> Self {
        let mut path = paths.data_dir.clone();
        path.push("form_fill.json");
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
            file_path: path,
        }
    }

    /// Guarda un valor de campo
    pub fn save(&self, domain: &str, field_name: &str, value: &str) {
        let key = (domain.to_string(), field_name.to_string());
        let mut entries = self.entries.lock().unwrap();
        let now = current_time();
        entries.entry(key.clone()).and_modify(|e| {
            e.value = value.to_string();
            e.use_count += 1;
            e.last_used = now;
        }).or_insert(FilledField {
            domain: domain.to_string(),
            field_name: field_name.to_string(),
            value: value.to_string(),
            use_count: 1,
            last_used: now,
        });
    }

    /// Obtiene un valor guardado
    pub fn get(&self, domain: &str, field_name: &str) -> Option<String> {
        self.entries.lock().unwrap()
            .get(&(domain.to_string(), field_name.to_string()))
            .map(|e| e.value.clone())
    }

    /// Elimina un campo
    pub fn remove(&self, domain: &str, field_name: &str) {
        self.entries.lock().unwrap()
            .remove(&(domain.to_string(), field_name.to_string()));
    }

    /// Elimina todos los campos de un dominio
    pub fn clear_domain(&self, domain: &str) {
        let mut entries = self.entries.lock().unwrap();
        entries.retain(|(d, _), _| d != domain);
    }

    /// Auto-rellena un formulario para un dominio
    pub fn autofill(&self, domain: &str, fields: &mut Vec<FormField>) {
        let entries = self.entries.lock().unwrap();
        for field in fields.iter_mut() {
            if let Some(saved) = entries.get(&(domain.to_string(), field.name.clone())) {
                if field.value.is_empty() {
                    field.value = saved.value.clone();
                }
            }
        }
    }

    /// Lista todos los campos para un dominio
    pub fn for_domain(&self, domain: &str) -> Vec<FilledField> {
        self.entries.lock().unwrap()
            .iter()
            .filter(|((d, _), _)| d == domain)
            .map(|(_, v)| v.clone())
            .collect()
    }

    /// Cuenta total
    pub fn count(&self) -> usize {
        self.entries.lock().unwrap().len()
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

    /// Carga desde disco
    pub fn load(&self) -> Result<(), String> {
        if !self.file_path.exists() {
            return Ok(());
        }
        let content = std::fs::read_to_string(&self.file_path)
            .map_err(|e| e.to_string())?;
        let loaded: HashMap<(String, String), FilledField> = serde_json::from_str(&content)
            .map_err(|e| e.to_string())?;
        *self.entries.lock().unwrap() = loaded;
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

    fn temp_paths() -> StoragePaths {
        use std::time::{SystemTime, UNIX_EPOCH};
        let mut paths = StoragePaths::new();
        let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        paths.data_dir = std::env::temp_dir().join(format!("noir-formfill-test-{}", t));
        paths.ensure_dir().unwrap();
        paths
    }

    #[test]
    fn test_save_and_get() {
        let m = FormFillManager::new(&temp_paths());
        m.save("example.com", "email", "user@test.com");
        assert_eq!(m.get("example.com", "email"), Some("user@test.com".to_string()));
    }

    #[test]
    fn test_remove() {
        let m = FormFillManager::new(&temp_paths());
        m.save("example.com", "email", "user@test.com");
        m.remove("example.com", "email");
        assert!(m.get("example.com", "email").is_none());
    }

    #[test]
    fn test_clear_domain() {
        let m = FormFillManager::new(&temp_paths());
        m.save("a.com", "email", "a@a.com");
        m.save("b.com", "email", "b@b.com");
        m.clear_domain("a.com");
        assert!(m.get("a.com", "email").is_none());
        assert!(m.get("b.com", "email").is_some());
    }

    #[test]
    fn test_autofill() {
        let m = FormFillManager::new(&temp_paths());
        m.save("example.com", "email", "user@test.com");
        m.save("example.com", "name", "John");
        let mut fields = vec![
            FormField { name: "email".to_string(), field_type: "email".to_string(), value: String::new() },
            FormField { name: "name".to_string(), field_type: "text".to_string(), value: String::new() },
        ];
        m.autofill("example.com", &mut fields);
        assert_eq!(fields[0].value, "user@test.com");
        assert_eq!(fields[1].value, "John");
    }

    #[test]
    fn test_autofill_preserves_existing() {
        let m = FormFillManager::new(&temp_paths());
        m.save("example.com", "email", "user@test.com");
        let mut fields = vec![
            FormField { name: "email".to_string(), field_type: "email".to_string(), value: "different@x.com".to_string() },
        ];
        m.autofill("example.com", &mut fields);
        // Should NOT overwrite existing value
        assert_eq!(fields[0].value, "different@x.com");
    }

    #[test]
    fn test_for_domain() {
        let m = FormFillManager::new(&temp_paths());
        m.save("a.com", "email", "a@a.com");
        m.save("a.com", "name", "A");
        m.save("b.com", "email", "b@b.com");
        let a_fields = m.for_domain("a.com");
        assert_eq!(a_fields.len(), 2);
    }

    #[test]
    fn test_save_load() {
        let paths = temp_paths();
        let m1 = FormFillManager::new(&paths);
        m1.save("example.com", "email", "user@test.com");
        m1.save_to_disk().unwrap();

        let m2 = FormFillManager::new(&paths);
        m2.load().unwrap();
        assert_eq!(m2.get("example.com", "email"), Some("user@test.com".to_string()));
    }

    #[test]
    fn test_use_count() {
        let m = FormFillManager::new(&temp_paths());
        m.save("example.com", "email", "user@test.com");
        m.save("example.com", "email", "user@test.com");
        let fields = m.for_domain("example.com");
        assert_eq!(fields[0].use_count, 2);
    }
}
