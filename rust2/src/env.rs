use std::collections::HashMap;

use crate::types::{MalForm, MalError, MalResult};

#[derive(Debug)]
pub struct Env<'a> {
    outer: Option<&'a Env<'a>>,
    data: HashMap<String, MalForm>,
}

impl<'a> Env<'a> {
    pub fn new(outer: Option<&'a Env<'a>>) -> Env<'a> {
        Env {
            outer,
            data: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, val: MalForm) {
        self.data.insert(key, val);
    }

    pub fn find(&'a self, key: &String) -> Option<&'a MalForm> {
        self.data.get(key).or_else(|| self.outer.and_then(|o| o.find(key)))
    }

    pub fn get(&'a self, key: &String) -> MalResult<&'a MalForm> {
        self.find(key).ok_or(MalError::EvalError(format!("'{}' not found", key)))
    }
}
