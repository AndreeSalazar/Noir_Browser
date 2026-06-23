//! Live DOM (FASE B1)
//!
//! DOM real (no solo parser result) que el JS puede mutar.
//! Implementa:
//! - Node, Element, Document
//! - querySelector, querySelectorAll
//! - getElementById, getElementsByTagName, getElementsByClassName
//! - createElement, createTextNode
//! - appendChild, removeChild, insertBefore
//! - textContent, innerHTML
//! - setAttribute, getAttribute, removeAttribute
//!
//! Inspirado en el WHATWG DOM standard, subset de Web IDL.

use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

/// Tipos de nodos del DOM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    Document,
    Element,
    Text,
    Comment,
    DocumentFragment,
}

impl NodeType {
    pub fn name(&self) -> &'static str {
        match self {
            NodeType::Document => "#document",
            NodeType::Element => "ELEMENT_NODE",
            NodeType::Text => "#text",
            NodeType::Comment => "#comment",
            NodeType::DocumentFragment => "#document-fragment",
        }
    }
}

/// Un nodo del DOM (live, mutable, con parent/child pointers)
#[derive(Debug)]
pub struct DomNode {
    pub node_type: NodeType,
    pub tag_name: String,
    pub node_value: String,  // para text/comment
    pub attributes: HashMap<String, String>,
    pub children: Vec<Rc<RefCell<DomNode>>>,
    pub parent_id: Option<u64>,  // id del parent node
    pub id: u64,
}

impl DomNode {
    /// Crear un nodo elemento
    pub fn new_element(tag: &str) -> Self {
        Self {
            node_type: NodeType::Element,
            tag_name: tag.to_lowercase(),
            node_value: String::new(),
            attributes: HashMap::new(),
            children: Vec::new(),
            parent_id: None,
            id: next_id(),
        }
    }

    /// Crear un nodo texto
    pub fn new_text(text: &str) -> Self {
        Self {
            node_type: NodeType::Text,
            tag_name: String::new(),
            node_value: text.to_string(),
            attributes: HashMap::new(),
            children: Vec::new(),
            parent_id: None,
            id: next_id(),
        }
    }

    /// Crear un nodo documento
    pub fn new_document() -> Self {
        Self {
            node_type: NodeType::Document,
            tag_name: String::new(),
            node_value: String::new(),
            attributes: HashMap::new(),
            children: Vec::new(),
            parent_id: None,
            id: next_id(),
        }
    }

    /// Tag name en MAYUSCULAS (HTML convention) o minusculas
    pub fn local_name(&self) -> &str {
        &self.tag_name
    }

    /// Es un element?
    pub fn is_element(&self) -> bool {
        self.node_type == NodeType::Element
    }

    /// Es un text?
    pub fn is_text(&self) -> bool {
        self.node_type == NodeType::Text
    }

    /// Set attribute
    pub fn set_attribute(&mut self, name: &str, value: &str) {
        self.attributes.insert(name.to_string(), value.to_string());
    }

    /// Get attribute
    pub fn get_attribute(&self, name: &str) -> Option<String> {
        self.attributes.get(name).cloned()
    }

    /// Remove attribute
    pub fn remove_attribute(&mut self, name: &str) {
        self.attributes.remove(name);
    }

    /// Get id attribute
    pub fn id(&self) -> Option<String> {
        self.get_attribute("id")
    }

    /// className (class attribute) split en lista
    pub fn class_list(&self) -> Vec<String> {
        if let Some(c) = self.get_attribute("class") {
            c.split_whitespace().map(String::from).collect()
        } else {
            Vec::new()
        }
    }

    /// textContent - concatena todos los text nodes descendientes
    pub fn text_content(&self) -> String {
        let mut out = String::new();
        self.collect_text(&mut out);
        out
    }

    fn collect_text(&self, out: &mut String) {
        if self.is_text() {
            out.push_str(&self.node_value);
        }
        for child in &self.children {
            child.borrow().collect_text(out);
        }
    }

    /// innerHTML - serializa hijos a HTML
    pub fn inner_html(&self) -> String {
        let mut out = String::new();
        for child in &self.children {
            out.push_str(&serialize_node(&child.borrow()));
        }
        out
    }

    /// Numero de hijos
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// Primer hijo
    pub fn first_child(&self) -> Option<Rc<RefCell<DomNode>>> {
        self.children.first().cloned()
    }

    /// Ultimo hijo
    pub fn last_child(&self) -> Option<Rc<RefCell<DomNode>>> {
        self.children.last().cloned()
    }

    /// Siguiente hermano
    pub fn next_sibling(&self) -> Option<Rc<RefCell<DomNode>>> {
        // Requiere acceso al padre - simplificado: no soportado
        None
    }

    /// Hermano anterior
    pub fn previous_sibling(&self) -> Option<Rc<RefCell<DomNode>>> {
        None
    }

    /// Append child
    pub fn append_child(&mut self, child: Rc<RefCell<DomNode>>) {
        // Quitar del padre anterior si lo tiene
        child.borrow_mut().parent_id = Some(self.id);
        self.children.push(child);
    }

    /// Remove child
    pub fn remove_child(&mut self, child: &Rc<RefCell<DomNode>>) -> Option<Rc<RefCell<DomNode>>> {
        let pos = self.children.iter().position(|c| c.borrow().id == child.borrow().id)?;
        let removed = self.children.remove(pos);
        removed.borrow_mut().parent_id = None;
        Some(removed)
    }

    /// Insert before
    pub fn insert_before(
        &mut self,
        new_child: Rc<RefCell<DomNode>>,
        reference: &Rc<RefCell<DomNode>>,
    ) -> Option<Rc<RefCell<DomNode>>> {
        let pos = self.children.iter().position(|c| c.borrow().id == reference.borrow().id)?;
        new_child.borrow_mut().parent_id = Some(self.id);
        self.children.insert(pos, new_child);
        Some(reference.clone())
    }
}

/// querySelector - encuentra el primer descendiente que matchea un selector CSS simple
/// Soporta: tag, .class, #id, tag.class, tag#id
pub fn query_selector(node: &Rc<RefCell<DomNode>>, selector: &str) -> Option<Rc<RefCell<DomNode>>> {
    let parsed = parse_selector(selector)?;
    query_selector_recursive(node, &parsed)
}

fn query_selector_recursive(
    node: &Rc<RefCell<DomNode>>,
    sel: &SimpleSelector,
) -> Option<Rc<RefCell<DomNode>>> {
    if matches_selector(&node.borrow(), sel) {
        return Some(node.clone());
    }
    let children = node.borrow().children.clone();
    for child in &children {
        if let Some(found) = query_selector_recursive(child, sel) {
            return Some(found);
        }
    }
    None
}

/// querySelectorAll - todos los descendientes que matchean
pub fn query_selector_all(node: &Rc<RefCell<DomNode>>, selector: &str) -> Vec<Rc<RefCell<DomNode>>> {
    if let Some(parsed) = parse_selector(selector) {
        let mut out = Vec::new();
        query_selector_all_recursive(node, &parsed, &mut out);
        out
    } else {
        Vec::new()
    }
}

fn query_selector_all_recursive(
    node: &Rc<RefCell<DomNode>>,
    sel: &SimpleSelector,
    out: &mut Vec<Rc<RefCell<DomNode>>>,
) {
    if matches_selector(&node.borrow(), sel) {
        out.push(node.clone());
    }
    let children = node.borrow().children.clone();
    for child in &children {
        query_selector_all_recursive(child, sel, out);
    }
}

fn matches_selector(node: &DomNode, sel: &SimpleSelector) -> bool {
    if node.node_type != NodeType::Element {
        return false;
    }
    if let Some(tag) = &sel.tag {
        if node.tag_name != *tag {
            return false;
        }
    }
    if let Some(id) = &sel.id {
        if node.get_attribute("id").as_deref() != Some(id.as_str()) {
            return false;
        }
    }
    for class in &sel.classes {
        if !node.class_list().iter().any(|c| c == class) {
            return false;
        }
    }
    true
}

#[derive(Debug, Clone, Default)]
struct SimpleSelector {
    tag: Option<String>,
    id: Option<String>,
    classes: Vec<String>,
}

fn parse_selector(sel: &str) -> Option<SimpleSelector> {
    let s: Vec<char> = sel.trim().chars().collect();
    if s.is_empty() { return None; }
    let mut out = SimpleSelector::default();
    let mut i = 0;
    let mut in_brackets = false;
    let mut tag_consumed = false;
    while i < s.len() {
        let c = s[i];
        i += 1;
        if in_brackets {
            if c == ']' { in_brackets = false; }
            continue;
        }
        if c == '#' {
            let start = i;
            while i < s.len() && (s[i].is_alphanumeric() || s[i] == '-' || s[i] == '_') {
                i += 1;
            }
            if i == start { return None; }
            out.id = Some(s[start..i].iter().collect());
        } else if c == '.' {
            let start = i;
            while i < s.len() && (s[i].is_alphanumeric() || s[i] == '-' || s[i] == '_') {
                i += 1;
            }
            if i == start { return None; }
            out.classes.push(s[start..i].iter().collect());
        } else if c == '[' {
            in_brackets = true;
        } else if c == ' ' || c == '>' || c == '+' || c == '~' {
            break;
        } else if !tag_consumed && c.is_ascii_alphabetic() {
            let start = i - 1;
            while i < s.len() && (s[i].is_ascii_alphabetic() || s[i].is_ascii_digit()) {
                i += 1;
            }
            let tag: String = s[start..i].iter().collect();
            out.tag = Some(tag.to_lowercase());
            tag_consumed = true;
        }
    }
    Some(out)
}

/// getElementById
pub fn get_element_by_id(node: &Rc<RefCell<DomNode>>, id: &str) -> Option<Rc<RefCell<DomNode>>> {
    if let Some(my_id) = node.borrow().id() {
        if my_id == id {
            return Some(node.clone());
        }
    }
    let children = node.borrow().children.clone();
    for child in &children {
        if let Some(found) = get_element_by_id(child, id) {
            return Some(found);
        }
    }
    None
}

/// getElementsByTagName
pub fn get_elements_by_tag_name(node: &Rc<RefCell<DomNode>>, tag: &str) -> Vec<Rc<RefCell<DomNode>>> {
    let mut out = Vec::new();
    let tag_lower = tag.to_lowercase();
    get_elements_by_tag_recursive(node, &tag_lower, &mut out);
    out
}

fn get_elements_by_tag_recursive(
    node: &Rc<RefCell<DomNode>>,
    tag: &str,
    out: &mut Vec<Rc<RefCell<DomNode>>>,
) {
    let n = node.borrow();
    if (tag == "*" || n.tag_name == tag) && n.is_element() {
        out.push(node.clone());
    }
    drop(n);
    let children = node.borrow().children.clone();
    for child in &children {
        get_elements_by_tag_recursive(child, tag, out);
    }
}

/// getElementsByClassName
pub fn get_elements_by_class_name(node: &Rc<RefCell<DomNode>>, class: &str) -> Vec<Rc<RefCell<DomNode>>> {
    let mut out = Vec::new();
    get_elements_by_class_recursive(node, class, &mut out);
    out
}

fn get_elements_by_class_recursive(
    node: &Rc<RefCell<DomNode>>,
    class: &str,
    out: &mut Vec<Rc<RefCell<DomNode>>>,
) {
    let n = node.borrow();
    if n.is_element() && n.class_list().iter().any(|c| c == class) {
        out.push(node.clone());
    }
    drop(n);
    let children = node.borrow().children.clone();
    for child in &children {
        get_elements_by_class_recursive(child, class, out);
    }
}

fn serialize_node(node: &DomNode) -> String {
    match node.node_type {
        NodeType::Text => escape_html(&node.node_value),
        NodeType::Comment => format!("<!--{}-->", node.node_value),
        NodeType::Element => {
            let mut s = format!("<{}", node.tag_name);
            for (k, v) in &node.attributes {
                s.push_str(&format!(" {}=\"{}\"", k, escape_html(v)));
            }
            s.push('>');
            for child in &node.children {
                s.push_str(&serialize_node(&child.borrow()));
            }
            s.push_str(&format!("</{}>", node.tag_name));
            s
        }
        _ => String::new(),
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn next_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_creation() {
        let n = DomNode::new_element("div");
        assert_eq!(n.tag_name, "div");
        assert!(n.is_element());
    }

    #[test]
    fn test_text_node() {
        let n = DomNode::new_text("hello");
        assert!(n.is_text());
        assert_eq!(n.node_value, "hello");
    }

    #[test]
    fn test_attributes() {
        let mut n = DomNode::new_element("a");
        n.set_attribute("href", "https://x.com");
        assert_eq!(n.get_attribute("href"), Some("https://x.com".to_string()));
        n.remove_attribute("href");
        assert_eq!(n.get_attribute("href"), None);
    }

    #[test]
    fn test_class_list() {
        let mut n = DomNode::new_element("div");
        n.set_attribute("class", "btn primary active");
        assert_eq!(n.class_list(), vec!["btn", "primary", "active"]);
    }

    #[test]
    fn test_text_content() {
        let mut parent = DomNode::new_element("div");
        let text1 = Rc::new(RefCell::new(DomNode::new_text("Hello ")));
        let text2 = Rc::new(RefCell::new(DomNode::new_text("world")));
        parent.append_child(text1);
        parent.append_child(text2);
        assert_eq!(parent.text_content(), "Hello world");
    }

    #[test]
    fn test_query_selector_tag() {
        let doc = Rc::new(RefCell::new(DomNode::new_document()));
        let div = Rc::new(RefCell::new(DomNode::new_element("div")));
        doc.borrow_mut().append_child(div);
        let result = query_selector(&doc, "div");
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_selector_tag() {
        let s = parse_selector("div").unwrap();
        assert_eq!(s.tag, Some("div".to_string()));
    }

    #[test]
    fn test_parse_selector_id() {
        let s = parse_selector("#main").unwrap();
        assert_eq!(s.id, Some("main".to_string()));
    }

    #[test]
    fn test_parse_selector_class() {
        let s = parse_selector(".btn").unwrap();
        assert_eq!(s.classes, vec!["btn"]);
    }

    #[test]
    fn test_parse_selector_compound() {
        let s = parse_selector("div.btn#main").unwrap();
        assert_eq!(s.tag, Some("div".to_string()));
        assert_eq!(s.id, Some("main".to_string()));
        assert_eq!(s.classes, vec!["btn"]);
    }

    #[test]
    fn test_matches_selector() {
        let mut n = DomNode::new_element("div");
        n.set_attribute("id", "main");
        n.set_attribute("class", "container wide");
        let sel = parse_selector("div#main.wide").unwrap();
        assert!(matches_selector(&n, &sel));
    }

    #[test]
    fn test_get_elements_by_tag_name() {
        let doc = Rc::new(RefCell::new(DomNode::new_document()));
        let a = Rc::new(RefCell::new(DomNode::new_element("a")));
        let b = Rc::new(RefCell::new(DomNode::new_element("a")));
        let c = Rc::new(RefCell::new(DomNode::new_element("p")));
        doc.borrow_mut().append_child(a);
        doc.borrow_mut().append_child(b);
        doc.borrow_mut().append_child(c);
        let results = get_elements_by_tag_name(&doc, "a");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<a>"), "&lt;a&gt;");
        assert_eq!(escape_html("\"hello\""), "&quot;hello&quot;");
    }

    #[test]
    fn test_serialize_node() {
        let n = DomNode::new_text("Hello");
        assert_eq!(serialize_node(&n), "Hello");
    }
}
