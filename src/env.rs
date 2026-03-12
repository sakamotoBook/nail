use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::Value;

#[derive(Clone)]
pub struct Env {
    parent: Option<Rc<Env>>,
    values: Rc<RefCell<HashMap<String, Value>>>,
}

impl Env {
    pub(crate) fn new() -> Self {
        Self {
            parent: None,
            values: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub(crate) fn child(&self) -> Self {
        Self {
            parent: Some(Rc::new(self.clone())),
            values: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub(crate) fn set(&self, key: &str, value: Value) {
        self.values.borrow_mut().insert(key.to_string(), value);
    }

    pub(crate) fn get(&self, key: &str) -> Option<Value> {
        if let Some(v) = self.values.borrow().get(key) {
            return Some(v.clone());
        }
        self.parent.as_ref().and_then(|p| p.get(key))
    }
}
