//! DOM - Document Object Model
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use super::value::JsValue;

#[derive(Debug, Clone)]
pub struct DomNode {
    pub id: u64,
    pub tag_name: String,
    pub attributes: HashMap<String, String>,
    pub text_content: String,
    pub inner_html: String,
    pub children: Vec<Rc<RefCell<DomNode>>>,
    pub parent: Option<Rc<RefCell<DomNode>>>,
    pub style: HashMap<String, String>,
    pub class_list: Vec<String>,
}

impl DomNode {
    pub fn new(tag_name: &str) -> Self {
        Self {
            id: Dom::next_id(),
            tag_name: tag_name.to_uppercase(),
            attributes: HashMap::new(),
            text_content: String::new(),
            inner_html: String::new(),
            children: Vec::new(),
            parent: None,
            style: HashMap::new(),
            class_list: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DomEvent {
    pub event_type: String,
    pub target_id: u64,
    pub bubbles: bool,
    pub cancelable: bool,
}

pub struct Dom {
    pub document: Rc<RefCell<DomNode>>,
    pub root: Rc<RefCell<DomNode>>,
    pub body: Rc<RefCell<DomNode>>,
    next_node_id: u64,
}

impl Dom {
    pub fn new() -> Self {
        let document = Rc::new(RefCell::new(DomNode::new("#document")));
        let html = Rc::new(RefCell::new(DomNode::new("HTML")));
        let head = Rc::new(RefCell::new(DomNode::new("HEAD")));
        let body = Rc::new(RefCell::new(DomNode::new("BODY")));

        html.borrow_mut().children.push(Rc::clone(&head));
        html.borrow_mut().children.push(Rc::clone(&body));
        head.borrow_mut().parent = Some(Rc::clone(&html));
        body.borrow_mut().parent = Some(Rc::clone(&html));
        document.borrow_mut().children.push(Rc::clone(&html));
        html.borrow_mut().parent = Some(Rc::clone(&document));

        let next_id = Rc::new(RefCell::new(0));
        Self {
            document,
            root: html,
            body,
            next_node_id: 1,
        }
    }

    pub fn next_id() -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    /// Create new element
    pub fn create_element(tag: &str) -> JsValue {
        let node = DomNode::new(tag);
        JsValue::DomElement(Rc::new(RefCell::new(node)))
    }

    /// Get element by ID
    pub fn get_element_by_id(&self, id: &str) -> Option<JsValue> {
        Self::find_by_id(&self.body, id).map(|n| JsValue::DomElement(n))
    }

    fn find_by_id(node: &Rc<RefCell<DomNode>>, id: &str) -> Option<Rc<RefCell<DomNode>>> {
        if node.borrow().attributes.get("id").map(|v| v == id).unwrap_or(false) {
            return Some(Rc::clone(node));
        }
        for child in &node.borrow().children {
            if let Some(found) = Self::find_by_id(child, id) {
                return Some(found);
            }
        }
        None
    }

    /// Query selector (simplified CSS)
    pub fn query_selector(&self, selector: &str) -> Option<JsValue> {
        Self::query(&self.body, selector).map(|n| JsValue::DomElement(n))
    }

    fn query(node: &Rc<RefCell<DomNode>>, selector: &str) -> Option<Rc<RefCell<DomNode>>> {
        let s = selector.trim();
        if let Some(rest) = s.strip_prefix('#') {
            return Self::find_by_id(node, rest);
        }
        if let Some(rest) = s.strip_prefix('.') {
            for child in &node.borrow().children {
                if child.borrow().class_list.contains(&rest.to_string()) {
                    return Some(Rc::clone(child));
                }
                if let Some(found) = Self::query(child, s) {
                    return Some(found);
                }
            }
            return None;
        }
        if node.borrow().tag_name.to_lowercase() == s.to_lowercase() {
            return Some(Rc::clone(node));
        }
        for child in &node.borrow().children {
            if let Some(found) = Self::query(child, s) {
                return Some(found);
            }
        }
        None
    }

    /// Query all (returns Array)
    pub fn query_selector_all(&self, selector: &str) -> JsValue {
        let mut results = Vec::new();
        Self::query_all(&self.body, selector, &mut results);
        JsValue::Array(Rc::new(RefCell::new(results)))
    }

    fn query_all(node: &Rc<RefCell<DomNode>>, selector: &str, results: &mut Vec<JsValue>) {
        let s = selector.trim();
        if let Some(class) = s.strip_prefix('.') {
            for child in &node.borrow().children {
                if child.borrow().class_list.contains(&class.to_string()) {
                    results.push(JsValue::DomElement(Rc::clone(child)));
                }
                Self::query_all(child, s, results);
            }
            return;
        }
        if node.borrow().tag_name.to_lowercase() == s.to_lowercase() {
            results.push(JsValue::DomElement(Rc::clone(node)));
        }
        for child in &node.borrow().children {
            Self::query_all(child, s, results);
        }
    }

    /// Append child to element
    pub fn append_child(&self, parent: &Rc<RefCell<DomNode>>, child: Rc<RefCell<DomNode>>) {
        child.borrow_mut().parent = Some(Rc::clone(parent));
        parent.borrow_mut().children.push(child);
    }

    /// Remove child from element
    pub fn remove_child(&self, parent: &Rc<RefCell<DomNode>>, child: &Rc<RefCell<DomNode>>) {
        let child_id = child.borrow().id;
        parent.borrow_mut().children.retain(|c| c.borrow().id != child_id);
    }

    /// Get property from element
    pub fn get_property(&self, tag: &str, property: &str) -> JsValue {
        match property {
            "tagName" | "nodeName" => JsValue::String(tag.to_uppercase()),
            "nodeType" => JsValue::Number(1.0),
            "innerHTML" => JsValue::String(String::new()),
            "textContent" => JsValue::String(String::new()),
            "style" => JsValue::Object(Rc::new(RefCell::new(HashMap::new()))),
            "children" => JsValue::Array(Rc::new(RefCell::new(Vec::new()))),
            "parentNode" | "parentElement" => JsValue::Null,
            "firstChild" | "lastChild" | "nextSibling" | "previousSibling" => JsValue::Null,
            "id" => JsValue::String(String::new()),
            "className" => JsValue::String(String::new()),
            _ => JsValue::Undefined,
        }
    }

    /// Set property on element
    pub fn set_property(&self, _tag: &str, property: &str, value: JsValue) {
        // Would store in a property map
        let _ = (property, value);
    }
}
