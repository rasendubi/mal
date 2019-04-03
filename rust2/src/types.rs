use std::fmt;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum MalForm {
    List(Vec<MalForm>),
    Vector(Vec<MalForm>),
    Atom(MalAtom),
    HashMap(HashMap<MalKey, MalForm>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MalKey {
    Symbol(String),
    String(String),
}

#[derive(Debug, Clone)]
pub enum MalAtom {
    Key(MalKey),
    Number(f32),
}

#[derive(Debug)]
pub enum MalError<'input> {
    ParseError(lalrpop_util::ParseError<usize, (usize, &'input str), &'static str>),
}

impl fmt::Display for MalForm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
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
            MalAtom::Number(n) => write!(f, "{}", n),
            MalAtom::Key(s) => write!(f, "{}", s),
        }
    }
}

impl fmt::Display for MalKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MalKey::Symbol(s) => write!(f, "{}", s),
            MalKey::String(s) => write!(f, "{:?}", s),
        }
    }
}

impl<'input> fmt::Display for MalError<'input> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MalError::ParseError(err) =>
                write!(f, "{}", err.clone().map_token(|(size,s)| s.chars().nth(size).unwrap())),
        }
    }
}
