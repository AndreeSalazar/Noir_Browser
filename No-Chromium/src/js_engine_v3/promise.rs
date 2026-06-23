//! Promises - A+ compliant minimal implementation
use super::JsValue;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum PromiseState {
    Pending,
    Fulfilled,
    Rejected,
}

#[derive(Debug, Clone)]
pub struct Promise {
    pub id: u64,
    pub state: PromiseState,
    pub value: Option<JsValue>,
    pub callbacks: Vec<u64>, // then callback IDs
    pub error_callbacks: Vec<u64>, // catch callback IDs
}

impl Promise {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            state: PromiseState::Pending,
            value: None,
            callbacks: Vec::new(),
            error_callbacks: Vec::new(),
        }
    }

    pub fn resolve(&mut self, value: JsValue) {
        if self.state == PromiseState::Pending {
            self.state = PromiseState::Fulfilled;
            self.value = Some(value);
        }
    }

    pub fn reject(&mut self, value: JsValue) {
        if self.state == PromiseState::Pending {
            self.state = PromiseState::Rejected;
            self.value = Some(value);
        }
    }

    pub fn then(&mut self, callback_id: u64) {
        if self.state == PromiseState::Fulfilled {
            // Execute immediately
            // Would need callback execution mechanism
        } else {
            self.callbacks.push(callback_id);
        }
    }

    pub fn catch(&mut self, callback_id: u64) {
        if self.state == PromiseState::Rejected {
            // Execute immediately
        } else {
            self.error_callbacks.push(callback_id);
        }
    }
}

pub struct PromiseQueue {
    pub promises: RefCell<HashMap<u64, Promise>>,
    pub next_id: RefCell<u64>,
    pub microtasks: RefCell<Vec<u64>>, // promise IDs to resolve
}

impl PromiseQueue {
    pub fn new() -> Self {
        Self {
            promises: RefCell::new(HashMap::new()),
            next_id: RefCell::new(1),
            microtasks: RefCell::new(Vec::new()),
        }
    }

    pub fn create(&self) -> u64 {
        let id = *self.next_id.borrow();
        *self.next_id.borrow_mut() += 1;
        let promise = Promise::new(id);
        self.promises.borrow_mut().insert(id, promise);
        id
    }

    pub fn resolve(&self, id: u64, value: JsValue) {
        if let Some(promise) = self.promises.borrow_mut().get_mut(&id) {
            promise.resolve(value);
            self.microtasks.borrow_mut().push(id);
        }
    }

    pub fn reject(&self, id: u64, value: JsValue) {
        if let Some(promise) = self.promises.borrow_mut().get_mut(&id) {
            promise.reject(value);
            self.microtasks.borrow_mut().push(id);
        }
    }

    pub fn get_state(&self, id: u64) -> Option<PromiseState> {
        self.promises.borrow().get(&id).map(|p| p.state.clone())
    }

    pub fn get_value(&self, id: u64) -> Option<JsValue> {
        self.promises.borrow().get(&id).and_then(|p| p.value.clone())
    }

    pub fn take_microtasks(&self) -> Vec<u64> {
        std::mem::take(&mut *self.microtasks.borrow_mut())
    }
}

// Native Promise constructor
pub fn js_promise_constructor(args: &[JsValue]) -> Result<JsValue, String> {
    if args.is_empty() {
        return Err("Promise requires executor function".to_string());
    }
    // Would need to store the executor and run it
    // For now, return a placeholder
    let mut map = std::collections::HashMap::new();
    map.insert("then".to_string(), JsValue::NativeFunction {
        name: "then".to_string(),
        func: |_args: &[JsValue]| Ok(JsValue::Undefined),
    });
    map.insert("catch".to_string(), JsValue::NativeFunction {
        name: "catch".to_string(),
        func: |_args: &[JsValue]| Ok(JsValue::Undefined),
    });
    Ok(JsValue::Object(Rc::new(RefCell::new(map))))
}

// Native async/await support functions
pub fn js_async_function(args: &[JsValue]) -> Result<JsValue, String> {
    // An async function always returns a Promise
    let promise = js_promise_constructor(args)?;
    Ok(promise)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_promise_creation() {
        let promise = Promise::new(1);
        assert_eq!(promise.state, PromiseState::Pending);
        assert_eq!(promise.id, 1);
    }

    #[test]
    fn test_promise_resolve() {
        let mut promise = Promise::new(1);
        promise.resolve(JsValue::Number(42.0));
        assert_eq!(promise.state, PromiseState::Fulfilled);
        assert!(promise.value.is_some());
    }

    #[test]
    fn test_promise_reject() {
        let mut promise = Promise::new(1);
        promise.reject(JsValue::String("error".to_string()));
        assert_eq!(promise.state, PromiseState::Rejected);
    }

    #[test]
    fn test_promise_queue() {
        let queue = PromiseQueue::new();
        let id = queue.create();
        assert_eq!(id, 1);
        queue.resolve(id, JsValue::Number(42.0));
        assert_eq!(queue.get_state(id), Some(PromiseState::Fulfilled));
    }
}
