use std::collections::VecDeque;

pub struct HistoryStore { 
    entries: VecDeque<String>,
    max_entries: usize,
}

impl HistoryStore {
    pub fn new() -> Self { 
        Self { 
            entries: VecDeque::new(),
            max_entries: 1000,
        } 
    }
    
    /// Load history from disk (stub - returns empty for Phase 0)
    pub fn load() -> Self {
        Self::new()
    }
    
    pub fn push(&mut self, url: &str) { 
        // Avoid duplicates at the front
        if self.entries.front().map(|e| e.as_str()) != Some(url) {
            self.entries.push_front(url.to_string());
            // Trim to max size
            while self.entries.len() > self.max_entries {
                self.entries.pop_back();
            }
        }
    }
    
    /// Record a visit (alias for push for API compatibility)
    pub fn record_visit(&mut self, url: &str) {
        self.push(url);
    }
    
    pub fn get_recent(&self, limit: usize) -> Vec<&str> {
        self.entries.iter().take(limit).map(|s| s.as_str()).collect()
    }
    
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for HistoryStore {
    fn default() -> Self {
        Self::new()
    }
}
