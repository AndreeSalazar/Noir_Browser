//! Environment - Almacena variables y scope chain
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use super::value::JsValue;

#[derive(Debug, Clone)]
pub struct Env {
    variables: HashMap<String, JsValue>,
    parent: Option<Rc<RefCell<Env>>>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Rc<RefCell<Env>>) -> Self {
        Self {
            variables: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn get(&self, name: &str) -> Option<JsValue> {
        if let Some(v) = self.variables.get(name) {
            return Some(v.clone());
        }
        if let Some(parent) = &self.parent {
            return parent.borrow().get(name);
        }
        None
    }

    pub fn set(&mut self, name: String, value: JsValue) {
        self.variables.insert(name, value);
    }

    pub fn has(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }
}
