//! WebAssembly Table - Tabla de referencias a funciones
//!
//! Equivalente a WebAssembly.Table en JS.

use std::sync::Mutex;

pub struct WasmTable {
    pub entries: Mutex<Vec<Option<u32>>>,
    pub initial: u32,
    pub max: Option<u32>,
}

impl WasmTable {
    pub fn new(initial: u32, max: Option<u32>) -> Self {
        Self {
            entries: Mutex::new(vec![None; initial as usize]),
            initial,
            max,
        }
    }

    /// Crece la tabla
    pub fn grow(&self, delta: u32) -> Result<i32, String> {
        let current_size = self.size();
        if let Some(max) = self.max {
            if current_size + delta > max {
                return Err("Table grow would exceed max".to_string());
            }
        }
        let old_len = current_size as i32;
        let mut entries = self.entries.lock().unwrap();
        let current = entries.len();
        entries.resize(current + delta as usize, None);
        Ok(old_len)
    }

    /// Obtiene elemento
    pub fn get(&self, idx: u32) -> Option<u32> {
        let entries = self.entries.lock().unwrap();
        entries.get(idx as usize).copied().flatten()
    }

    /// Establece elemento
    pub fn set(&self, idx: u32, value: Option<u32>) {
        let mut entries = self.entries.lock().unwrap();
        if let Some(slot) = entries.get_mut(idx as usize) {
            *slot = value;
        }
    }

    pub fn size(&self) -> u32 {
        self.entries.lock().unwrap().len() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_creation() {
        let t = WasmTable::new(5, None);
        assert_eq!(t.size(), 5);
    }

    #[test]
    fn test_table_grow() {
        let t = WasmTable::new(2, None);
        let old = t.grow(3).unwrap();
        assert_eq!(old, 2);
        assert_eq!(t.size(), 5);
    }

    #[test]
    fn test_table_get_set() {
        let t = WasmTable::new(3, None);
        t.set(1, Some(42));
        assert_eq!(t.get(1), Some(42));
        assert_eq!(t.get(0), None);
    }

    #[test]
    fn test_table_grow_max() {
        let t = WasmTable::new(1, Some(3));
        assert!(t.grow(1).is_ok());
        assert!(t.grow(2).is_err());
    }
}
