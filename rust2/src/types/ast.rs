use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use super::{MalError, MalResult};

#[derive(Debug, Clone)]
pub enum MalForm {
    NativeFn(String, MalNativeFn),
    MalFn(Rc<MalFn>),
    List(Vec<MalForm>),
    Vector(Vec<MalForm>),
    Atom(MalAtom),
    HashMap(HashMap<MalKey, MalForm>),
}

#[derive(Clone)]
pub struct MalNativeFn(pub Rc<Fn(Vec<MalForm>) -> MalResult<MalForm>>);

#[derive(Debug, Clone)]
pub struct MalFn {
    pub ast: MalForm,
    pub params: Vec<String>,
    pub env: Rc<RefCell<Env>>,
    fn_: MalForm, // MalNativeFn
}

impl MalFn {
    pub fn new(outer: Rc<RefCell<Env>>, bindings: Vec<String>, body: MalForm, eval: fn(&MalForm, &Rc<RefCell<Env>>) -> MalResult<MalForm>) -> MalFn {
        MalFn {
            ast: body.clone(),
            params: bindings.clone(),
            env: outer.clone(),
            fn_: MalForm::NativeFn("fn*".to_string(), MalNativeFn(Rc::new(move |params| {
                let env = Rc::new(RefCell::new(Env::new_fn_closure(Some(outer.clone()), &bindings, &params)?));

                eval(&body, &env)
            }))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MalKey {
    String(String),
    Keyword(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MalAtom {
    Key(MalKey),
    Number(f64),
    Symbol(String),
    True,
    False,
    Nil,
}

impl MalForm {
    pub fn coerce_list(&self) -> Option<&Vec<MalForm>> {
        match self {
            MalForm::List(v) => Some(v),
            MalForm::Vector(v) => Some(v),
            _ => None,
        }
    }
}

impl PartialEq<MalNativeFn> for MalNativeFn {
    fn eq(&self, _other: &MalNativeFn) -> bool {
        false
    }
}

impl fmt::Debug for MalNativeFn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#<function>")
    }
}

impl PartialEq<MalForm> for MalForm {
    fn eq(&self, other: &MalForm) -> bool {
        if let ml1@Some(_) = self.coerce_list() {
            return ml1 == other.coerce_list();
        }

        match (self, other) {
            (MalForm::NativeFn(_, f1), MalForm::NativeFn(_, f2)) => f1 == f2,
            (MalForm::Atom(a1), MalForm::Atom(a2)) => a1 == a2,
            (MalForm::HashMap(h1), MalForm::HashMap(h2)) => h1 == h2,
            _ => false,
        }
    }
}

// it has circular dependency on ast
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

    pub fn new_fn_closure(outer: Option<Rc<RefCell<Env>>>, binds: &[String], exprs: &[MalForm]) -> MalResult<Env> {
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
