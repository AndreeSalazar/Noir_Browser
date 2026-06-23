//! Tab Groups - Agrupar tabs con color/nombre
//!
//! Chrome-like tab groups con colores personalizables.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroupColor {
    Grey,
    Blue,
    Red,
    Yellow,
    Green,
    Pink,
    Purple,
    Cyan,
    Orange,
}

impl GroupColor {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "blue" | "azul" => Self::Blue,
            "red" | "rojo" => Self::Red,
            "yellow" | "amarillo" => Self::Yellow,
            "green" | "verde" => Self::Green,
            "pink" | "rosa" => Self::Pink,
            "purple" | "morado" => Self::Purple,
            "cyan" => Self::Cyan,
            "orange" | "naranja" => Self::Orange,
            _ => Self::Grey,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Grey => "grey",
            Self::Blue => "blue",
            Self::Red => "red",
            Self::Yellow => "yellow",
            Self::Green => "green",
            Self::Pink => "pink",
            Self::Purple => "purple",
            Self::Cyan => "cyan",
            Self::Orange => "orange",
        }
    }

    pub fn rgb(&self) -> (u8, u8, u8) {
        match self {
            Self::Grey => (128, 128, 128),
            Self::Blue => (66, 133, 244),
            Self::Red => (234, 67, 53),
            Self::Yellow => (251, 188, 4),
            Self::Green => (52, 168, 83),
            Self::Pink => (233, 30, 99),
            Self::Purple => (156, 39, 176),
            Self::Cyan => (0, 188, 212),
            Self::Orange => (255, 152, 0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TabGroup {
    pub id: u32,
    pub name: String,
    pub color: GroupColor,
    pub collapsed: bool,
    pub tab_ids: Vec<u32>,
}

impl TabGroup {
    pub fn new(id: u32, name: &str, color: GroupColor) -> Self {
        Self {
            id,
            name: name.to_string(),
            color,
            collapsed: false,
            tab_ids: Vec::new(),
        }
    }

    pub fn add_tab(&mut self, tab_id: u32) {
        if !self.tab_ids.contains(&tab_id) {
            self.tab_ids.push(tab_id);
        }
    }

    pub fn remove_tab(&mut self, tab_id: u32) {
        self.tab_ids.retain(|&id| id != tab_id);
    }

    pub fn tab_count(&self) -> usize {
        self.tab_ids.len()
    }

    pub fn toggle_collapsed(&mut self) {
        self.collapsed = !self.collapsed;
    }
}

pub struct TabGroupManager {
    groups: HashMap<u32, TabGroup>,
    next_id: u32,
}

impl TabGroupManager {
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn create(&mut self, name: &str, color: GroupColor) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let group = TabGroup::new(id, name, color);
        self.groups.insert(id, group);
        id
    }

    pub fn delete(&mut self, id: u32) {
        self.groups.remove(&id);
    }

    pub fn get(&self, id: u32) -> Option<&TabGroup> {
        self.groups.get(&id)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut TabGroup> {
        self.groups.get_mut(&id)
    }

    pub fn all(&self) -> Vec<&TabGroup> {
        let mut v: Vec<&TabGroup> = self.groups.values().collect();
        v.sort_by_key(|g| g.id);
        v
    }

    pub fn count(&self) -> usize {
        self.groups.len()
    }

    pub fn assign_tab(&mut self, group_id: u32, tab_id: u32) {
        if let Some(g) = self.groups.get_mut(&group_id) {
            g.add_tab(tab_id);
        }
    }

    pub fn unassign_tab(&mut self, group_id: u32, tab_id: u32) {
        if let Some(g) = self.groups.get_mut(&group_id) {
            g.remove_tab(tab_id);
        }
    }

    pub fn find_group_for_tab(&self, tab_id: u32) -> Option<&TabGroup> {
        self.groups.values().find(|g| g.tab_ids.contains(&tab_id))
    }
}

impl Default for TabGroupManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_str() {
        assert_eq!(GroupColor::from_str("blue"), GroupColor::Blue);
        assert_eq!(GroupColor::from_str("ROJO"), GroupColor::Red);
        assert_eq!(GroupColor::from_str("unknown"), GroupColor::Grey);
    }

    #[test]
    fn test_color_to_str() {
        assert_eq!(GroupColor::Blue.to_str(), "blue");
    }

    #[test]
    fn test_color_rgb() {
        let (r, g, b) = GroupColor::Red.rgb();
        assert_eq!(r, 234);
        assert_eq!(g, 67);
    }

    #[test]
    fn test_group_new() {
        let g = TabGroup::new(1, "Work", GroupColor::Blue);
        assert_eq!(g.id, 1);
        assert_eq!(g.name, "Work");
        assert_eq!(g.tab_count(), 0);
        assert!(!g.collapsed);
    }

    #[test]
    fn test_group_add_remove_tab() {
        let mut g = TabGroup::new(1, "X", GroupColor::Green);
        g.add_tab(10);
        g.add_tab(20);
        g.add_tab(10); // duplicate
        assert_eq!(g.tab_count(), 2);
        g.remove_tab(10);
        assert_eq!(g.tab_count(), 1);
    }

    #[test]
    fn test_group_collapsed() {
        let mut g = TabGroup::new(1, "X", GroupColor::Grey);
        assert!(!g.collapsed);
        g.toggle_collapsed();
        assert!(g.collapsed);
    }

    #[test]
    fn test_manager_create() {
        let mut m = TabGroupManager::new();
        let id = m.create("Work", GroupColor::Blue);
        assert!(m.get(id).is_some());
    }

    #[test]
    fn test_manager_delete() {
        let mut m = TabGroupManager::new();
        let id = m.create("X", GroupColor::Red);
        m.delete(id);
        assert!(m.get(id).is_none());
    }

    #[test]
    fn test_manager_count() {
        let mut m = TabGroupManager::new();
        m.create("A", GroupColor::Blue);
        m.create("B", GroupColor::Red);
        assert_eq!(m.count(), 2);
    }

    #[test]
    fn test_manager_assign() {
        let mut m = TabGroupManager::new();
        let id = m.create("Work", GroupColor::Blue);
        m.assign_tab(id, 5);
        let g = m.get(id).unwrap();
        assert_eq!(g.tab_count(), 1);
    }

    #[test]
    fn test_manager_unassign() {
        let mut m = TabGroupManager::new();
        let id = m.create("Work", GroupColor::Blue);
        m.assign_tab(id, 5);
        m.unassign_tab(id, 5);
        assert_eq!(m.get(id).unwrap().tab_count(), 0);
    }

    #[test]
    fn test_find_group_for_tab() {
        let mut m = TabGroupManager::new();
        let id1 = m.create("A", GroupColor::Blue);
        let id2 = m.create("B", GroupColor::Red);
        m.assign_tab(id1, 100);
        m.assign_tab(id2, 200);
        assert_eq!(m.find_group_for_tab(100).unwrap().id, id1);
        assert_eq!(m.find_group_for_tab(200).unwrap().id, id2);
        assert!(m.find_group_for_tab(999).is_none());
    }

    #[test]
    fn test_all_sorted() {
        let mut m = TabGroupManager::new();
        m.create("B", GroupColor::Blue);
        m.create("A", GroupColor::Red);
        let all = m.all();
        assert_eq!(all.len(), 2);
    }
}
