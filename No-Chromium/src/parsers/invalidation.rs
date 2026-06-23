//! Style Invalidation (Blink-inspired)
//!
//! Basado en el sistema de Blink para invalidar styles:
//! - Tracking de que selectors dependen de que (class, id, attribute, etc)
//! - Cuando un class cambia, solo se invalidan los nodos afectados
//! - Reduce drasticamente el re-style trabajo
//!
//! Tipos de invalidation en Blink:
//! - Class change: invalidar nodos que dependen de class selector
//! - Id change: invalidar nodos que dependen de id selector
//! - Attribute change: invalidar nodos que dependen de [attr] selectors
//! - Hover/focus: invalidar affected node

use std::collections::{HashMap, HashSet};

/// Tipo de cambio que invalida styles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InvalidationType {
    ClassAdded,
    ClassRemoved,
    IdChanged,
    AttributeChanged,
    HoverState,
    FocusState,
    ChildStructure,
}

/// Conjunto de selectores que dependen de una clase (Blink-style)
#[derive(Debug, Default)]
pub struct SelectorDependencyIndex {
    /// class -> selectors que la usan
    class_deps: HashMap<String, HashSet<String>>,
    /// id -> selectors que la usan
    id_deps: HashMap<String, HashSet<String>>,
    /// attribute -> selectors que la usan
    attr_deps: HashMap<String, HashSet<String>>,
}

impl SelectorDependencyIndex {
    pub fn new() -> Self { Self::default() }

    /// Registrar un selector con sus dependencias
    pub fn register(&mut self, selector: &str) {
        // Parse simple: buscar .class, #id, [attr]
        let mut chars = selector.chars().peekable();
        let mut current = String::new();
        while let Some(c) = chars.next() {
            if c == '.' {
                // Es una clase
                current.clear();
                while let Some(&nc) = chars.peek() {
                    if nc.is_alphanumeric() || nc == '-' || nc == '_' {
                        current.push(nc);
                        chars.next();
                    } else { break; }
                }
                if !current.is_empty() {
                    self.class_deps.entry(current.clone()).or_default().insert(selector.to_string());
                }
            } else if c == '#' {
                current.clear();
                while let Some(&nc) = chars.peek() {
                    if nc.is_alphanumeric() || nc == '-' || nc == '_' {
                        current.push(nc);
                        chars.next();
                    } else { break; }
                }
                if !current.is_empty() {
                    self.id_deps.entry(current.clone()).or_default().insert(selector.to_string());
                }
            } else if c == '[' {
                current.clear();
                while let Some(&nc) = chars.peek() {
                    if nc == ']' { break; }
                    current.push(nc);
                    chars.next();
                }
                if !current.is_empty() {
                    let attr = current.split('=').next().unwrap_or(&current).trim().to_string();
                    self.attr_deps.entry(attr).or_default().insert(selector.to_string());
                }
            }
        }
    }

    /// Que selectores se invalidan si una clase cambia
    pub fn invalidators_for_class(&self, class: &str) -> Option<HashSet<String>> {
        self.class_deps.get(class).cloned()
    }

    pub fn invalidators_for_id(&self, id: &str) -> Option<HashSet<String>> {
        self.id_deps.get(id).cloned()
    }

    pub fn invalidators_for_attr(&self, attr: &str) -> Option<HashSet<String>> {
        self.attr_deps.get(attr).cloned()
    }

    /// Total de dependencias registradas
    pub fn total_deps(&self) -> usize {
        self.class_deps.len() + self.id_deps.len() + self.attr_deps.len()
    }
}

/// Set de nodos invalidados (Blink-style)
#[derive(Debug, Default)]
pub struct InvalidationSet {
    /// Nodos invalidados por class
    pub class_invalidations: HashMap<String, HashSet<u64>>,
    /// Nodos invalidados por id
    pub id_invalidations: HashMap<String, HashSet<u64>>,
    /// Nodos invalidados globalmente
    pub full_invalidations: HashSet<u64>,
}

impl InvalidationSet {
    pub fn new() -> Self { Self::default() }

    /// Invalidar todos los nodos con una clase especifica
    pub fn invalidate_class(&mut self, class: &str, node_id: u64) {
        self.class_invalidations.entry(class.to_string()).or_default().insert(node_id);
    }

    pub fn invalidate_id(&mut self, id: &str, node_id: u64) {
        self.id_invalidations.entry(id.to_string()).or_default().insert(node_id);
    }

    /// Invalidar todo (cuando un ancestor cambia estructura)
    pub fn invalidate_full(&mut self, node_id: u64) {
        self.full_invalidations.insert(node_id);
    }

    /// Procesa las invalidaciones con un index de dependencias
    pub fn collect_selectors_to_invalidate(&self, deps: &SelectorDependencyIndex) -> HashSet<String> {
        let mut out = HashSet::new();
        for class in self.class_invalidations.keys() {
            if let Some(sels) = deps.invalidators_for_class(class) {
                out.extend(sels);
            }
        }
        for id in self.id_invalidations.keys() {
            if let Some(sels) = deps.invalidators_for_id(id) {
                out.extend(sels);
            }
        }
        out
    }

    /// Total de invalidaciones pendientes
    pub fn total_pending(&self) -> usize {
        let c: usize = self.class_invalidations.values().map(|s| s.len()).sum();
        let i: usize = self.id_invalidations.values().map(|s| s.len()).sum();
        c + i + self.full_invalidations.len()
    }

    /// Clear all
    pub fn clear(&mut self) {
        self.class_invalidations.clear();
        self.id_invalidations.clear();
        self.full_invalidations.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_class_selector() {
        let mut idx = SelectorDependencyIndex::new();
        idx.register(".btn");
        let sels = idx.invalidators_for_class("btn").unwrap();
        assert!(sels.contains(".btn"));
    }

    #[test]
    fn test_register_id_selector() {
        let mut idx = SelectorDependencyIndex::new();
        idx.register("#header");
        let sels = idx.invalidators_for_id("header").unwrap();
        assert!(sels.contains("#header"));
    }

    #[test]
    fn test_register_attribute_selector() {
        let mut idx = SelectorDependencyIndex::new();
        idx.register("[data-id]");
        let sels = idx.invalidators_for_attr("data-id").unwrap();
        assert!(sels.contains("[data-id]"));
    }

    #[test]
    fn test_compound_selector() {
        let mut idx = SelectorDependencyIndex::new();
        idx.register("div.btn#main[data-foo]");
        assert!(idx.invalidators_for_class("btn").is_some());
        assert!(idx.invalidators_for_id("main").is_some());
        assert!(idx.invalidators_for_attr("data-foo").is_some());
    }

    #[test]
    fn test_invalidate_class() {
        let mut set = InvalidationSet::new();
        set.invalidate_class("btn", 42);
        set.invalidate_class("btn", 43);
        assert_eq!(set.total_pending(), 2);
    }

    #[test]
    fn test_invalidate_id() {
        let mut set = InvalidationSet::new();
        set.invalidate_id("main", 1);
        assert_eq!(set.total_pending(), 1);
    }

    #[test]
    fn test_invalidate_full() {
        let mut set = InvalidationSet::new();
        set.invalidate_full(5);
        assert_eq!(set.total_pending(), 1);
    }

    #[test]
    fn test_collect_with_deps() {
        let mut idx = SelectorDependencyIndex::new();
        idx.register(".btn");
        idx.register(".btn.primary");
        let mut set = InvalidationSet::new();
        set.invalidate_class("btn", 1);
        let sels = set.collect_selectors_to_invalidate(&idx);
        assert_eq!(sels.len(), 2);
    }

    #[test]
    fn test_clear() {
        let mut set = InvalidationSet::new();
        set.invalidate_class("a", 1);
        set.invalidate_id("b", 2);
        set.invalidate_full(3);
        set.clear();
        assert_eq!(set.total_pending(), 0);
    }

    #[test]
    fn test_total_deps() {
        let mut idx = SelectorDependencyIndex::new();
        idx.register(".a");
        idx.register("#b");
        idx.register("[c]");
        assert_eq!(idx.total_deps(), 3);
    }
}
