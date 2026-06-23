//! Rule Tree (Firefox CSS engine-inspired)
//!
//! Basado en el algoritmo de Firefox:
//! - Una linked-list de reglas CSS ordenadas por especificidad
//! - Multiple nodos DOM pueden compartir el mismo path en el rule tree
//! - En restyle, si las reglas del padre no cambian, los hijos pueden
//!   simplemente seguir el puntero y subir el arbol para obtener todas
//!   las reglas aplicables
//!
//! Ahorra trabajo de re-matching en restyles.

use std::collections::HashMap;
use std::sync::Arc;

/// Una regla CSS con su especificidad
#[derive(Debug, Clone)]
pub struct CssRule {
    pub selector: String,
    pub declarations: Vec<(String, String)>,
    pub specificity: u32,
    pub origin: RuleOrigin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleOrigin {
    UserAgent,
    User,
    Author,
}

impl CssRule {
    pub fn new(selector: &str, specificity: u32, origin: RuleOrigin) -> Self {
        Self {
            selector: selector.to_string(),
            declarations: Vec::new(),
            specificity,
            origin,
        }
    }
}

/// Nodo del rule tree
#[derive(Debug, Clone)]
pub struct RuleNode {
    pub id: u64,
    pub rule: Option<Arc<CssRule>>,
    /// Indice al parent node (None solo para root)
    pub parent: Option<u64>,
    /// Hijos directos
    pub children: Vec<u64>,
    /// Ref count para garbage collection
    pub ref_count: u32,
}

/// Rule tree (Firefox CSS engine)
pub struct RuleTree {
    nodes: HashMap<u64, RuleNode>,
    root: u64,
    next_id: u64,
    /// LRU cache: key = (parent_node_id, rule_id) -> child_node_id
    cache: HashMap<(u64, u64), u64>,
}

impl RuleTree {
    pub fn new() -> Self {
        let mut tree = Self {
            nodes: HashMap::new(),
            root: 0,
            next_id: 1,
            cache: HashMap::new(),
        };
        tree.root = tree.create_node(None, None);
        tree
    }

    fn create_node(&mut self, rule: Option<Arc<CssRule>>, parent: Option<u64>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.insert(id, RuleNode {
            id,
            rule,
            parent,
            children: Vec::new(),
            ref_count: 0,
        });
        id
    }

    /// Encuentra o crea un path en el rule tree para una lista ordenada de reglas.
    /// Retorna el id del nodo final (mas especifico) del path.
    pub fn ensure_path(&mut self, parent_id: u64, rules: &[Arc<CssRule>]) -> u64 {
        if rules.is_empty() {
            return parent_id;
        }
        let mut current = parent_id;
        for rule in rules {
            let key = (current, rule_selector_id(rule));
            if let Some(&cached) = self.cache.get(&key) {
                current = cached;
                if let Some(node) = self.nodes.get_mut(&current) {
                    node.ref_count += 1;
                }
                continue;
            }
            // Crear nuevo nodo hijo de current
            let new_id = self.create_node(Some(rule.clone()), Some(current));
            if let Some(parent_node) = self.nodes.get_mut(&current) {
                parent_node.children.push(new_id);
            }
            self.cache.insert(key, new_id);
            current = new_id;
        }
        current
    }

    /// Obtiene todas las reglas aplicables a un nodo (siguiendo el path)
    pub fn get_applicable_rules(&self, leaf_id: u64) -> Vec<Arc<CssRule>> {
        // Recolectamos todos los nodos del path (del leaf al root)
        let mut path = Vec::new();
        let mut current = Some(leaf_id);
        let mut visited = 0;
        while let Some(id) = current {
            if visited > 10000 { break; }
            visited += 1;
            if let Some(node) = self.nodes.get(&id) {
                let parent = node.parent;
                path.push(node);
                current = parent;
            } else {
                break;
            }
        }
        // path esta en orden: leaf -> root (mas especifico a menos especifico)
        // Filtramos solo las reglas (algunos nodos son solo structure)
        path.iter().filter_map(|n| n.rule.clone()).collect()
    }

    /// Tamano del tree
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Estadisticas: numero de nodos y profundidad
    pub fn stats(&self) -> (usize, usize) {
        let total = self.nodes.len();
        let mut max_depth = 0;
        for &id in self.nodes.keys() {
            let mut depth = 0;
            let mut current = Some(id);
            let mut visited = 0;
            while let Some(nid) = current {
                if visited > 10000 { break; }
                visited += 1;
                depth += 1;
                current = self.nodes.get(&nid).and_then(|n| n.parent);
            }
            if depth > max_depth {
                max_depth = depth;
            }
        }
        (total, max_depth)
    }
}

fn rule_selector_id(rule: &CssRule) -> u64 {
    // Hash simple del selector para usar como key
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    rule.selector.hash(&mut hasher);
    rule.specificity.hash(&mut hasher);
    hasher.finish()
}

impl Default for RuleTree {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_path() {
        let mut tree = RuleTree::new();
        let path = tree.ensure_path(tree.root, &[]);
        assert_eq!(path, tree.root);
    }

    #[test]
    fn test_single_rule_path() {
        let mut tree = RuleTree::new();
        let rule = Arc::new(CssRule::new("p", 1, RuleOrigin::Author));
        let path = tree.ensure_path(tree.root, &[rule.clone()]);
        assert_ne!(path, tree.root);
        let rules = tree.get_applicable_rules(path);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].selector, "p");
    }

    #[test]
    fn test_multiple_rules_path() {
        let mut tree = RuleTree::new();
        let r1 = Arc::new(CssRule::new("p", 1, RuleOrigin::Author));
        let r2 = Arc::new(CssRule::new(".clase", 10, RuleOrigin::Author));
        let r3 = Arc::new(CssRule::new("#id", 100, RuleOrigin::Author));
        let path = tree.ensure_path(tree.root, &[r1, r2, r3]);
        let rules = tree.get_applicable_rules(path);
        assert_eq!(rules.len(), 3);
        // Debe estar de mas especifico a menos especifico
        assert_eq!(rules[0].selector, "#id");
    }

    #[test]
    fn test_path_sharing() {
        let mut tree = RuleTree::new();
        let r1 = Arc::new(CssRule::new("p", 1, RuleOrigin::Author));
        let r2 = Arc::new(CssRule::new(".test", 10, RuleOrigin::Author));
        let path1 = tree.ensure_path(tree.root, &[r1.clone(), r2.clone()]);
        let initial_len = tree.len();
        // Mismo path -> debe reusar nodos
        let path2 = tree.ensure_path(tree.root, &[r1, r2]);
        assert_eq!(path1, path2);
        // Tree no debe crecer
        assert_eq!(tree.len(), initial_len);
    }

    #[test]
    fn test_stats() {
        let tree = RuleTree::new();
        let (total, max_depth) = tree.stats();
        assert!(total >= 1);
        assert!(max_depth >= 1);
    }

    #[test]
    fn test_different_paths() {
        let mut tree = RuleTree::new();
        let r1 = Arc::new(CssRule::new("p", 1, RuleOrigin::Author));
        let r2 = Arc::new(CssRule::new("div", 1, RuleOrigin::Author));
        let path1 = tree.ensure_path(tree.root, &[r1]);
        let path2 = tree.ensure_path(tree.root, &[r2]);
        assert_ne!(path1, path2);
    }
}
