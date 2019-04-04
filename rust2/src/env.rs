#![allow(dead_code)]

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use crate::types::{MalForm, MalError, MalResult};

#[derive(Debug)]
pub struct Env {
    outer: Option<Rc<RefCell<Env>>>,
    data: HashMap<String, MalForm>,
}

impl Env {
    pub fn new(outer: Option<Rc<RefCell<Env>>>) -> Env {
        Env {
            outer,
            data: HashMap::new(),
        }
    }

    pub fn new_fn_closure(outer: Option<Rc<RefCell<Env>>>, binds: &Vec<String>, exprs: &Vec<MalForm>) -> MalResult<Env> {
        let mut env = Env {
            outer,
            data: HashMap::new(),
        };

        let mut it = binds.iter().enumerate();
        while let Some((i, key)) = it.next() {
            if key == "&" {
                let (_, next_key) = it.next().ok_or(MalError::EvalError(format!("& requires next argument")))?;
                env.set(next_key.clone(), MalForm::List(Vec::from(&exprs[i ..])));
                break;
            }

            let val = exprs.get(i).ok_or(MalError::EvalError(format!("Missing value for '{}' parameter", key)))?;
            env.set(key.clone(), val.clone());
        }

        Ok(env)
    }

    pub fn set(&mut self, key: String, val: MalForm) {
        self.data.insert(key, val);
    }

    pub fn find(&self, key: &String) -> Option<MalForm> {
        if let Some(val) = self.data.get(key) {
            Some(val.clone())
        } else if let Some(out) = &self.outer {
            out.borrow().find(key)
        } else {
            None
        }
    }

    pub fn get(&self, key: &String) -> MalResult<MalForm> {
        self.find(key).ok_or(MalError::EvalError(format!("'{}' not found", key)))
    }
}
