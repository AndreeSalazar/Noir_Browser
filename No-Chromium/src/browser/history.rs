// HistoryStore: Almacena el historial de navegación en memoria (sin disco).
// Stub implementado para resolver error E0432.

use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct HistoryEntry {
    pub url: String,
    pub title: String,
    pub timestamp: DateTime<Utc>,
    pub tab_id: u64,
}

pub struct HistoryStore {
    entries: Vec<HistoryEntry>,
    max_entries: usize,
}

impl HistoryStore {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::with_capacity(max_entries),
            max_entries,
        }
    }

    pub fn add_entry(&mut self, url: String, title: String, tab_id: u64) {
        let entry = HistoryEntry {
            url,
            title,
            timestamp: Utc::now(),
            tab_id,
        };
        
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }

    pub fn get_recent(&self, limit: usize) -> &[HistoryEntry] {
        let start = self.entries.len().saturating_sub(limit);
        &self.entries[start..]
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
