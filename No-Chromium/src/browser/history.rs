use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const HISTORY_LIMIT: usize = 512;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub url: String,
    pub visit_count: u32,
    pub last_visited_unix: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct HistoryStore {
    pub entries: Vec<HistoryEntry>,
}

impl HistoryStore {
    pub fn load() -> Self {
        let path = history_path();
        let Ok(contents) = fs::read_to_string(path) else {
            return Self::default();
        };

        serde_json::from_str(&contents).unwrap_or_default()
    }

    pub fn record_visit(&mut self, url: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default();

        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.url == url) {
            entry.visit_count = entry.visit_count.saturating_add(1);
            entry.last_visited_unix = now;
        } else {
            self.entries.push(HistoryEntry {
                url: url.to_string(),
                visit_count: 1,
                last_visited_unix: now,
            });
        }

        self.entries
            .sort_by(|a, b| b.last_visited_unix.cmp(&a.last_visited_unix));
        self.entries.truncate(HISTORY_LIMIT);
        self.save();
    }

    fn save(&self) {
        let path = history_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }
}

fn history_path() -> PathBuf {
    PathBuf::from("profile").join("history.json")
}
