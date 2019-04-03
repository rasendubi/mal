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

impl fmt::Debug for MalNativeFn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<native-fn>")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MalKey {
    String(String),
    Keyword(String),
}

#[derive(Debug, Clone)]
pub enum MalAtom {
    Key(MalKey),
    Number(f32),
    Symbol(String),
}

#[derive(Debug, Clone)]
pub enum MalError {
    ParseError(lalrpop_util::ParseError<usize, (usize, String), &'static str>),
    EvalError(String),
}

pub type MalResult<T> = Result<T, MalError>;

impl fmt::Display for MalForm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MalForm::NativeFn(name, _) => write!(f, "{}", name),
            MalForm::Atom(x) => write!(f, "{}", x),
            MalForm::List(xs) => {
                write!(f, "(")?;
                let mut it = xs.into_iter();

                if let Some(x) = it.next() {
                    write!(f, "{}", x)?;

                    for x in it {
                        write!(f, " {}", x)?;
                    }
                }

                write!(f, ")")
            },
            MalForm::Vector(xs) => {
                write!(f, "[")?;
                let mut it = xs.into_iter();

                if let Some(x) = it.next() {
                    write!(f, "{}", x)?;

                    for x in it {
                        write!(f, " {}", x)?;
                    }
                }

                write!(f, "]")
            },
            MalForm::HashMap(xs) => {
                write!(f, "{{")?;
                let mut it = xs.into_iter();

                if let Some((k, v)) = it.next() {
                    write!(f, "{} {}", k, v)?;

                    for (k, v) in it {
                        write!(f, " {} {}", k, v)?;
                    }
                }

                write!(f, "}}")
            },
        }
    }
}

impl fmt::Display for MalAtom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MalAtom::Key(s) => write!(f, "{}", s),
            MalAtom::Number(n) => write!(f, "{}", n),
            MalAtom::Symbol(s) => write!(f, "{}", s),
        }
    }
}

impl fmt::Display for MalKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MalKey::String(s) => write!(f, "{:?}", s),
            MalKey::Keyword(s) => write!(f, ":{}", s),
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
