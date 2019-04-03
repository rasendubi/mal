use std::fmt;

#[derive(Debug, Clone)]
pub enum MalForm {
    List(Vec<MalForm>),
    Vector(Vec<MalForm>),
    Atom(MalAtom),
    // shortcut
    HashMap(Vec<MalForm>),
}

#[derive(Debug, Clone)]
pub enum MalAtom {
    Number(f32),
    Symbol(String),
    String(String),
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

                if let Some(x) = it.next() {
                    write!(f, "{}", x)?;

                    for x in it {
                        write!(f, " {}", x)?;
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
            MalAtom::Symbol(s) => write!(f, "{}", s),
            MalAtom::Number(n) => write!(f, "{}", n),
            MalAtom::String(s) => write!(f, "{:?}", s),
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
