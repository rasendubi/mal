#![allow(dead_code)]

use std::fmt;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum MalForm {
    NativeFn(String, MalNativeFn),
    List(Vec<MalForm>),
    Vector(Vec<MalForm>),
    Atom(MalAtom),
    HashMap(HashMap<MalKey, MalForm>),
}

#[derive(Clone)]
pub struct MalNativeFn(pub Rc<Fn(Vec<MalForm>) -> MalResult<MalForm>>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MalKey {
    String(String),
    Keyword(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MalAtom {
    Key(MalKey),
    Number(f32),
    Symbol(String),
    True,
    False,
    Nil,
}

#[derive(Debug, Clone)]
pub enum MalError {
    ParseError(lalrpop_util::ParseError<usize, (usize, String), &'static str>),
    EvalError(String),
}

pub type MalResult<T> = Result<T, MalError>;

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

impl fmt::Display for MalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MalError::ParseError(err) =>
                write!(f, "{}", err.clone().map_token(|(_size,s)| s)),
            MalError::EvalError(msg) => write!(f, "Evaluation Error: {}", msg),
        }
    }
}

pub trait ToMalForm {
    fn to_mal_form(&self) -> MalForm;
}

impl ToMalForm for bool {
    fn to_mal_form(&self) -> MalForm {
        MalForm::Atom(if *self {MalAtom::True} else {MalAtom::False})
    }
}

impl ToMalForm for f32 {
    fn to_mal_form(&self) -> MalForm {
        MalForm::Atom(MalAtom::Number(*self))
    }
}

impl ToMalForm for String {
    fn to_mal_form(&self) -> MalForm {
        MalForm::Atom(MalAtom::Key(MalKey::String(self.clone())))
    }
}

impl ToMalForm for () {
    fn to_mal_form(&self) -> MalForm {
        MalForm::Atom(MalAtom::Nil)
    }
}

impl ToMalForm for MalKey {
    fn to_mal_form(&self) -> MalForm {
        MalForm::Atom(MalAtom::Key(self.clone()))
    }
}
