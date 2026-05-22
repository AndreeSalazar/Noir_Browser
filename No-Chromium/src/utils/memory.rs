use std::collections::HashMap;
use zeroize::Zeroize;

/// Cache entry with zeroizable data field
#[derive(Clone)]
pub struct CacheEntry {
    pub data: Vec<u8>,
    pub timestamp: std::time::Instant,
}

// Manual Zeroize implementation - only zeroize the data field
// std::time::Instant cannot be zeroized and doesn't need to be for privacy
impl Zeroize for CacheEntry {
    fn zeroize(&mut self) {
        self.data.zeroize();
        // timestamp is metadata, not sensitive data - leave as is
    }
}

pub struct EphemeralCache {
    map: HashMap<String, CacheEntry>,
    max_bytes: usize,
    current_bytes: usize,
}

impl EphemeralCache {
    pub fn new(max_bytes: usize) -> Self { 
        Self { 
            map: HashMap::new(), 
            max_bytes,
            current_bytes: 0,
        } 
    }

    pub fn write(&mut self, key: &str, data: Vec<u8>) {
        let data_len = data.len();
        
        // Evict old entries if we exceed max_bytes
        while self.current_bytes + data_len > self.max_bytes {
            if let Some((oldest_key, _)) = self.map.iter().next() {
                let oldest_key = oldest_key.clone();
                if let Some(entry) = self.map.remove(&oldest_key) {
                    self.current_bytes = self.current_bytes.saturating_sub(entry.data.len());
                }
            } else {
                break;
            }
        }
        
        // Insert new entry
        if let Some(old_entry) = self.map.insert(key.to_string(), CacheEntry {
            data: data.clone(),
            timestamp: std::time::Instant::now(),
        }) {
            self.current_bytes = self.current_bytes
                .saturating_sub(old_entry.data.len())
                .saturating_add(data_len);
        } else {
            self.current_bytes = self.current_bytes.saturating_add(data_len);
        }
    }

    pub fn get(&self, key: &str) -> Option<CacheEntry> {
        self.map.get(key).cloned()
    }
    
    pub fn clear(&mut self) {
        for entry in self.map.values_mut() {
            entry.zeroize();
        }
        self.map.clear();
        self.current_bytes = 0;
    }
}

impl Drop for EphemeralCache {
    fn drop(&mut self) {
        self.clear();
    }
}
