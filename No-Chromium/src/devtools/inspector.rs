//! Inspector - DOM tree inspector
//!
//! Permite ver y explorar el DOM de la página actual.

use crate::parsers::dom_tree::DomNode;

#[derive(Debug, Clone)]
pub struct DomNodeInfo {
    pub depth: u32,
    pub tag: String,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub text_preview: String,
    pub has_children: bool,
}

pub struct Inspector;

impl Inspector {
    /// Genera un árbol de strings del DOM
    pub fn tree_as_strings(nodes: &[DomNode]) -> Vec<String> {
        let mut output = Vec::new();
        for node in nodes {
            Self::render_node(node, 0, &mut output);
        }
        output
    }

    /// Genera info resumida de un nodo
    pub fn node_info(node: &DomNode, depth: u32) -> DomNodeInfo {
        match node {
            DomNode::Element { tag, attributes, children } => {
                let id = attributes.get("id").cloned();
                let classes: Vec<String> = attributes.get("class")
                    .map(|c| c.split_whitespace().map(String::from).collect())
                    .unwrap_or_default();
                let text_preview = Self::first_text(children);
                DomNodeInfo {
                    depth,
                    tag: format!("{:?}", tag),
                    id,
                    classes,
                    text_preview,
                    has_children: !children.is_empty(),
                }
            }
            DomNode::Text(text) => DomNodeInfo {
                depth,
                tag: "Text".to_string(),
                id: None,
                classes: vec![],
                text_preview: text.chars().take(50).collect(),
                has_children: false,
            },
        }
    }

    /// Cuenta total de nodos
    pub fn count_nodes(nodes: &[DomNode]) -> usize {
        let mut count = 0;
        for node in nodes {
            count += Self::count_recursive(node);
        }
        count
    }

    /// Cuenta elementos por tag
    pub fn count_by_tag(nodes: &[DomNode]) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        for node in nodes {
            Self::count_tags_recursive(node, &mut counts);
        }
        counts
    }

    /// Encuentra elementos por ID
    pub fn find_by_id<'a>(nodes: &'a [DomNode], id: &str) -> Option<&'a DomNode> {
        for node in nodes {
            if let Some(found) = Self::find_by_id_recursive(node, id) {
                return Some(found);
            }
        }
        None
    }

    /// Encuentra elementos por clase
    pub fn find_by_class<'a>(nodes: &'a [DomNode], class: &str) -> Vec<&'a DomNode> {
        let mut results = Vec::new();
        for node in nodes {
            Self::find_by_class_recursive(node, class, &mut results);
        }
        results
    }

    /// Encuentra elementos por tag
    pub fn find_by_tag<'a>(nodes: &'a [DomNode], tag_name: &str) -> Vec<&'a DomNode> {
        let mut results = Vec::new();
        for node in nodes {
            Self::find_by_tag_recursive(node, tag_name, &mut results);
        }
        results
    }

    fn render_node(node: &DomNode, depth: u32, output: &mut Vec<String>) {
        let indent = "  ".repeat(depth as usize);
        match node {
            DomNode::Element { tag, attributes, children } => {
                let id_part = attributes.get("id")
                    .map(|i| format!(" id=\"{}\"", i))
                    .unwrap_or_default();
                let class_part = attributes.get("class")
                    .map(|c| format!(" class=\"{}\"", c))
                    .unwrap_or_default();
                output.push(format!("{}<{:?}{}{}>", indent, tag, id_part, class_part));
                for child in children {
                    Self::render_node(child, depth + 1, output);
                }
            }
            DomNode::Text(text) => {
                let preview: String = text.chars().take(40).collect();
                output.push(format!("{}\"{}\"", indent, preview));
            }
        }
    }

    fn first_text(nodes: &[DomNode]) -> String {
        for node in nodes {
            if let DomNode::Text(text) = node {
                return text.chars().take(50).collect();
            }
            if let DomNode::Element { children, .. } = node {
                let inner = Self::first_text(children);
                if !inner.is_empty() {
                    return inner;
                }
            }
        }
        String::new()
    }

    fn count_recursive(node: &DomNode) -> usize {
        match node {
            DomNode::Element { children, .. } => {
                1 + children.iter().map(Self::count_recursive).sum::<usize>()
            }
            DomNode::Text(_) => 1,
        }
    }

    fn count_tags_recursive(node: &DomNode, counts: &mut std::collections::HashMap<String, usize>) {
        match node {
            DomNode::Element { tag, children, .. } => {
                *counts.entry(format!("{:?}", tag)).or_insert(0) += 1;
                for child in children {
                    Self::count_tags_recursive(child, counts);
                }
            }
            DomNode::Text(_) => {
                *counts.entry("Text".to_string()).or_insert(0) += 1;
            }
        }
    }

    fn find_by_id_recursive<'a>(node: &'a DomNode, id: &str) -> Option<&'a DomNode> {
        if let DomNode::Element { attributes, children, .. } = node {
            if attributes.get("id").map(|i| i == id).unwrap_or(false) {
                return Some(node);
            }
            for child in children {
                if let Some(found) = Self::find_by_id_recursive(child, id) {
                    return Some(found);
                }
            }
        }
        None
    }

    fn find_by_class_recursive<'a>(node: &'a DomNode, class: &str, results: &mut Vec<&'a DomNode>) {
        if let DomNode::Element { attributes, children, .. } = node {
            if let Some(c) = attributes.get("class") {
                if c.split_whitespace().any(|x| x == class) {
                    results.push(node);
                }
            }
            for child in children {
                Self::find_by_class_recursive(child, class, results);
            }
        }
    }

    fn find_by_tag_recursive<'a>(node: &'a DomNode, tag_name: &str, results: &mut Vec<&'a DomNode>) {
        if let DomNode::Element { tag, children, .. } = node {
            let tag_str = format!("{:?}", tag);
            if tag_str == tag_name || tag_str.to_lowercase() == tag_name.to_lowercase() {
                results.push(node);
            }
            for child in children {
                Self::find_by_tag_recursive(child, tag_name, results);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::dom_tree::parse_html;

    #[test]
    fn test_inspector_tree() {
        let html = "<html><body><h1>Title</h1><p>Text</p></body></html>";
        let nodes = parse_html(html);
        let tree = Inspector::tree_as_strings(&nodes);
        assert!(!tree.is_empty());
        assert!(tree.iter().any(|l| l.contains("H1")));
    }

    #[test]
    fn test_count_nodes() {
        let html = "<html><body><h1>Title</h1><p>Text</p></body></html>";
        let nodes = parse_html(html);
        let count = Inspector::count_nodes(&nodes);
        assert!(count >= 5);
    }

    #[test]
    fn test_count_by_tag() {
        let html = "<html><body><h1>1</h1><h1>2</h1><p>Text</p></body></html>";
        let nodes = parse_html(html);
        let counts = Inspector::count_by_tag(&nodes);
        assert!(counts.values().sum::<usize>() >= 3);
    }

    #[test]
    fn test_find_by_id() {
        let html = r#"<html><body><div id="main">Content</div></body></html>"#;
        let nodes = parse_html(html);
        let found = Inspector::find_by_id(&nodes, "main");
        assert!(found.is_some());
    }

    #[test]
    fn test_find_by_class() {
        let html = r#"<html><body><div class="card a">A</div><div class="card b">B</div></body></html>"#;
        let nodes = parse_html(html);
        let cards = Inspector::find_by_class(&nodes, "card");
        assert_eq!(cards.len(), 2);
    }

    #[test]
    fn test_find_by_tag() {
        let html = "<html><body><h1>1</h1><h1>2</h1><p>3</p></body></html>";
        let nodes = parse_html(html);
        let h1s = Inspector::find_by_tag(&nodes, "H1");
        assert_eq!(h1s.len(), 2);
    }

    #[test]
    fn test_node_info_text() {
        let html = "<p>Hello world</p>";
        let nodes = parse_html(html);
        if let Some(node) = nodes.first() {
            let info = Inspector::node_info(node, 0);
            assert!(!info.text_preview.is_empty() || info.tag == "Text");
        }
    }

    #[test]
    fn test_inspector_empty() {
        let nodes = vec![];
        assert_eq!(Inspector::count_nodes(&nodes), 0);
        assert!(Inspector::tree_as_strings(&nodes).is_empty());
    }
}
