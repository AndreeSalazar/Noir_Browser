//! Favorites - Marcadores con tags, folders, búsqueda
//!
//! Bookmarks profesionales con organización.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct Favorite {
    pub id: u32,
    pub title: String,
    pub url: String,
    pub folder: String,
    pub tags: Vec<String>,
    pub created_at: u64,
    pub visit_count: u32,
    pub favicon: Option<String>,
}

impl Favorite {
    pub fn new(id: u32, title: &str, url: &str) -> Self {
        Self {
            id,
            title: title.to_string(),
            url: url.to_string(),
            folder: "Bookmarks".to_string(),
            tags: Vec::new(),
            created_at: current_time(),
            visit_count: 0,
            favicon: None,
        }
    }

    pub fn with_folder(mut self, folder: &str) -> Self {
        self.folder = folder.to_string();
        self
    }

    pub fn add_tag(&mut self, tag: &str) {
        if !self.tags.iter().any(|t| t == tag) {
            self.tags.push(tag.to_string());
        }
    }

    pub fn matches(&self, query: &str) -> bool {
        if query.is_empty() { return true; }
        let q = query.to_lowercase();
        self.title.to_lowercase().contains(&q)
            || self.url.to_lowercase().contains(&q)
            || self.folder.to_lowercase().contains(&q)
            || self.tags.iter().any(|t| t.to_lowercase().contains(&q))
    }

    pub fn increment_visit(&mut self) {
        self.visit_count += 1;
    }
}

pub struct FavoritesManager {
    favorites: HashMap<u32, Favorite>,
    folders: HashMap<String, Vec<u32>>,
    next_id: u32,
}

impl FavoritesManager {
    pub fn new() -> Self {
        let mut m = Self {
            favorites: HashMap::new(),
            folders: HashMap::new(),
            next_id: 1,
        };
        m.folders.insert("Bookmarks".to_string(), Vec::new());
        m
    }

    pub fn add(&mut self, title: &str, url: &str) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let fav = Favorite::new(id, title, url);
        let folder = fav.folder.clone();
        self.favorites.insert(id, fav);
        self.folders.entry(folder).or_default().push(id);
        id
    }

    pub fn add_to_folder(&mut self, title: &str, url: &str, folder: &str) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let fav = Favorite::new(id, title, url).with_folder(folder);
        let folder_name = fav.folder.clone();
        self.favorites.insert(id, fav);
        self.folders.entry(folder_name).or_default().push(id);
        id
    }

    pub fn remove(&mut self, id: u32) {
        if let Some(fav) = self.favorites.remove(&id) {
            if let Some(ids) = self.folders.get_mut(&fav.folder) {
                ids.retain(|&i| i != id);
            }
        }
    }

    pub fn get(&self, id: u32) -> Option<&Favorite> {
        self.favorites.get(&id)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut Favorite> {
        self.favorites.get_mut(&id)
    }

    pub fn all(&self) -> Vec<&Favorite> {
        self.favorites.values().collect()
    }

    pub fn in_folder(&self, folder: &str) -> Vec<&Favorite> {
        self.folders.get(folder)
            .map(|ids| ids.iter().filter_map(|id| self.favorites.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn search(&self, query: &str) -> Vec<&Favorite> {
        self.favorites.values().filter(|f| f.matches(query)).collect()
    }

    pub fn folders(&self) -> Vec<&String> {
        self.folders.keys().collect()
    }

    pub fn count(&self) -> usize {
        self.favorites.len()
    }

    pub fn create_folder(&mut self, name: &str) {
        self.folders.entry(name.to_string()).or_default();
    }

    pub fn move_to_folder(&mut self, id: u32, new_folder: &str) {
        if let Some(fav) = self.favorites.get_mut(&id) {
            let old_folder = fav.folder.clone();
            fav.folder = new_folder.to_string();
            if let Some(ids) = self.folders.get_mut(&old_folder) {
                ids.retain(|&i| i != id);
            }
            self.folders.entry(new_folder.to_string()).or_default().push(id);
        }
    }
}

impl Default for FavoritesManager {
    fn default() -> Self { Self::new() }
}

fn current_time() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_favorite_new() {
        let f = Favorite::new(1, "GitHub", "https://github.com");
        assert_eq!(f.id, 1);
        assert_eq!(f.title, "GitHub");
        assert_eq!(f.folder, "Bookmarks");
    }

    #[test]
    fn test_favorite_with_folder() {
        let f = Favorite::new(1, "X", "https://x.com").with_folder("Work");
        assert_eq!(f.folder, "Work");
    }

    #[test]
    fn test_favorite_add_tag() {
        let mut f = Favorite::new(1, "X", "u");
        f.add_tag("rust");
        f.add_tag("rust"); // duplicate
        assert_eq!(f.tags.len(), 1);
    }

    #[test]
    fn test_favorite_matches() {
        let mut f = Favorite::new(1, "GitHub", "https://github.com");
        f.add_tag("code");
        assert!(f.matches("git"));
        assert!(f.matches("GIT"));
        assert!(f.matches("code"));
        assert!(f.matches("https"));
        assert!(f.matches(""));
        assert!(!f.matches("xyz"));
    }

    #[test]
    fn test_favorite_visit() {
        let mut f = Favorite::new(1, "X", "u");
        f.increment_visit();
        f.increment_visit();
        assert_eq!(f.visit_count, 2);
    }

    #[test]
    fn test_manager_add() {
        let mut m = FavoritesManager::new();
        let id = m.add("X", "https://x.com");
        assert!(m.get(id).is_some());
    }

    #[test]
    fn test_manager_add_to_folder() {
        let mut m = FavoritesManager::new();
        let id = m.add_to_folder("X", "u", "Work");
        assert_eq!(m.get(id).unwrap().folder, "Work");
    }

    #[test]
    fn test_manager_remove() {
        let mut m = FavoritesManager::new();
        let id = m.add("X", "u");
        m.remove(id);
        assert!(m.get(id).is_none());
    }

    #[test]
    fn test_manager_in_folder() {
        let mut m = FavoritesManager::new();
        m.add_to_folder("A", "u1", "Work");
        m.add_to_folder("B", "u2", "Work");
        m.add_to_folder("C", "u3", "Personal");
        assert_eq!(m.in_folder("Work").len(), 2);
        assert_eq!(m.in_folder("Personal").len(), 1);
    }

    #[test]
    fn test_manager_search() {
        let mut m = FavoritesManager::new();
        m.add("GitHub", "https://github.com");
        m.add("GitLab", "https://gitlab.com");
        m.add("Google", "https://google.com");
        let r = m.search("git");
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn test_manager_folders() {
        let mut m = FavoritesManager::new();
        m.add_to_folder("A", "u1", "Work");
        m.add_to_folder("B", "u2", "Personal");
        let f = m.folders();
        assert!(f.len() >= 2);
    }

    #[test]
    fn test_manager_move_folder() {
        let mut m = FavoritesManager::new();
        let id = m.add_to_folder("A", "u1", "Work");
        m.move_to_folder(id, "Personal");
        assert_eq!(m.get(id).unwrap().folder, "Personal");
        assert_eq!(m.in_folder("Work").len(), 0);
        assert_eq!(m.in_folder("Personal").len(), 1);
    }

    #[test]
    fn test_manager_create_folder() {
        let mut m = FavoritesManager::new();
        m.create_folder("Shopping");
        assert!(m.folders().iter().any(|f| f.as_str() == "Shopping"));
    }

    #[test]
    fn test_manager_count() {
        let mut m = FavoritesManager::new();
        m.add("A", "u1");
        m.add("B", "u2");
        m.add("C", "u3");
        assert_eq!(m.count(), 3);
    }
}
