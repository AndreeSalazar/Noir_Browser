//! WebAssembly Memory - Memoria lineal
//!
//! Equivalente a WebAssembly.Memory en JS.

use std::sync::Mutex;

pub struct WasmMemory {
    pub data: Mutex<Vec<u8>>,
    pub initial_pages: u32,
    pub max_pages: Option<u32>,
}

const PAGE_SIZE: usize = 65536; // 64KB

impl WasmMemory {
    /// Crea una nueva memoria con N páginas iniciales
    pub fn new(initial_pages: u32) -> Result<Self, String> {
        if initial_pages == 0 {
            return Err("Memory must have at least 1 page".to_string());
        }
        let size = (initial_pages as usize) * PAGE_SIZE;
        Ok(Self {
            data: Mutex::new(vec![0u8; size]),
            initial_pages,
            max_pages: None,
        })
    }

    /// Crea con max_pages
    pub fn with_max(initial_pages: u32, max_pages: u32) -> Result<Self, String> {
        if initial_pages > max_pages {
            return Err("Initial pages > max pages".to_string());
        }
        let mut mem = Self::new(initial_pages)?;
        mem.max_pages = Some(max_pages);
        Ok(mem)
    }

    /// Crece la memoria N páginas
    pub fn grow(&self, pages: u32) -> Result<i32, String> {
        let current_pages = self.size_in_pages();

        if let Some(max) = self.max_pages {
            if current_pages + pages > max {
                return Err("Memory grow would exceed max".to_string());
            }
        }

        let additional = (pages as usize) * PAGE_SIZE;
        let mut data = self.data.lock().unwrap();
        let old = (data.len() / PAGE_SIZE) as i32;
        let new_size = data.len() + additional;
        data.resize(new_size, 0);
        Ok(old)
    }

    /// Tamaño actual en bytes
    pub fn size(&self) -> usize {
        self.data.lock().unwrap().len()
    }

    /// Tamaño en páginas
    pub fn size_in_pages(&self) -> u32 {
        (self.size() / PAGE_SIZE) as u32
    }

    /// Lee bytes
    pub fn read(&self, offset: usize, len: usize) -> Option<Vec<u8>> {
        let data = self.data.lock().unwrap();
        if offset + len <= data.len() {
            Some(data[offset..offset+len].to_vec())
        } else {
            None
        }
    }

    /// Escribe bytes
    pub fn write(&self, offset: usize, bytes: &[u8]) -> Result<(), String> {
        let mut data = self.data.lock().unwrap();
        if offset + bytes.len() <= data.len() {
            data[offset..offset+bytes.len()].copy_from_slice(bytes);
            Ok(())
        } else {
            Err("Write out of bounds".to_string())
        }
    }

    /// Lee un i32 little-endian
    pub fn read_i32(&self, offset: usize) -> Option<i32> {
        self.read(offset, 4).map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// Escribe un i32 little-endian
    pub fn write_i32(&self, offset: usize, value: i32) -> Result<(), String> {
        self.write(offset, &value.to_le_bytes())
    }

    /// Lee string UTF-8 hasta null terminator
    pub fn read_string(&self, offset: usize) -> Option<String> {
        let data = self.data.lock().unwrap();
        let mut end = offset;
        while end < data.len() && data[end] != 0 {
            end += 1;
        }
        if end < data.len() {
            String::from_utf8(data[offset..end].to_vec()).ok()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation() {
        let mem = WasmMemory::new(1).unwrap();
        assert_eq!(mem.size(), 65536);
    }

    #[test]
    fn test_memory_zero_pages() {
        assert!(WasmMemory::new(0).is_err());
    }

    #[test]
    fn test_memory_write_read() {
        let mem = WasmMemory::new(1).unwrap();
        mem.write(0, b"hello").unwrap();
        let data = mem.read(0, 5).unwrap();
        assert_eq!(data, b"hello");
    }

    #[test]
    fn test_memory_i32() {
        let mem = WasmMemory::new(1).unwrap();
        mem.write_i32(0, 0x12345678).unwrap();
        assert_eq!(mem.read_i32(0), Some(0x12345678));
    }

    #[test]
    fn test_memory_grow() {
        let mem = WasmMemory::new(1).unwrap();
        let old = mem.grow(2).unwrap();
        assert_eq!(old, 1);
        assert_eq!(mem.size(), 3 * 65536);
    }

    #[test]
    fn test_memory_grow_with_max() {
        let mem = WasmMemory::with_max(1, 2).unwrap();
        assert!(mem.grow(1).is_ok());
        assert!(mem.grow(1).is_err());
    }

    #[test]
    fn test_memory_read_string() {
        let mem = WasmMemory::new(1).unwrap();
        mem.write(0, b"hello\x00world").unwrap();
        assert_eq!(mem.read_string(0), Some("hello".to_string()));
    }

    #[test]
    fn test_memory_out_of_bounds() {
        let mem = WasmMemory::new(1).unwrap();
        let result = mem.read(65530, 100);
        assert!(result.is_none());
    }
}
