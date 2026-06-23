//! JsValue - Valores del JS engine
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use super::ast::*;
use super::dom::DomNode;

#[derive(Debug, Clone)]
pub enum JsValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Object(Rc<RefCell<HashMap<String, JsValue>>>),
    Array(Rc<RefCell<Vec<JsValue>>>),
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<super::env::Env>>,
    },
    NativeFunction {
        name: String,
        func: fn(&[JsValue]) -> Result<JsValue, String>,
    },
    DomElement(Rc<RefCell<DomNode>>),
}

impl PartialEq for JsValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (JsValue::Undefined, JsValue::Undefined) => true,
            (JsValue::Null, JsValue::Null) => true,
            (JsValue::Boolean(a), JsValue::Boolean(b)) => a == b,
            (JsValue::Number(a), JsValue::Number(b)) => a == b,
            (JsValue::String(a), JsValue::String(b)) => a == b,
            _ => false,
        }
    }
}

impl JsValue {
    pub fn type_of(&self) -> &'static str {
        match self {
            JsValue::Undefined => "undefined",
            JsValue::Null => "object",
            JsValue::Boolean(_) => "boolean",
            JsValue::Number(_) => "number",
            JsValue::String(_) => "string",
            JsValue::Object(_) | JsValue::Array(_) => "object",
            JsValue::Function { .. } | JsValue::NativeFunction { .. } => "function",
            JsValue::DomElement(_) => "object",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            JsValue::Boolean(b) => *b,
            JsValue::Number(n) => *n != 0.0 && !n.is_nan(),
            JsValue::String(s) => !s.is_empty(),
            JsValue::Undefined | JsValue::Null => false,
            _ => true,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            JsValue::Undefined => "undefined".to_string(),
            JsValue::Null => "null".to_string(),
            JsValue::Boolean(b) => b.to_string(),
            JsValue::Number(n) => {
                if n.is_nan() { "NaN".to_string() }
                else if n.is_infinite() { if *n > 0.0 { "Infinity".to_string() } else { "-Infinity".to_string() } }
                else { n.to_string() }
            }
            JsValue::String(s) => s.clone(),
            JsValue::Object(_) => "[object Object]".to_string(),
            JsValue::Array(arr) => {
                let elements = arr.borrow();
                let strs: Vec<String> = elements.iter().map(|e| e.to_string()).collect();
                strs.join(",")
            }
            _ => "[function]".to_string(),
        }
    }

    pub fn to_number(&self) -> f64 {
        match self {
            JsValue::Number(n) => *n,
            JsValue::Boolean(true) => 1.0,
            JsValue::Boolean(false) => 0.0,
            JsValue::String(s) => s.parse().unwrap_or(f64::NAN),
            JsValue::Null => 0.0,
            JsValue::Undefined => f64::NAN,
            _ => f64::NAN,
        }
    }
}
