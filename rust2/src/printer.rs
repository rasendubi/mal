#![allow(dead_code)]
use std::fmt;

use crate::types::{MalForm,MalAtom,MalKey,ToMalForm};

impl MalForm {
    pub fn pr_str(&self, print_readably: bool) -> String {
        pr_str(self, print_readably)
    }
}

pub fn pr_str(x: &MalForm, print_readably: bool) -> String {
    match x {
        MalForm::NativeFn(name, _) => format!("#<{}>", name),
        MalForm::MalFn(_) => format!("#<fn*>"),
        MalForm::Atom(MalAtom::Key(MalKey::String(s))) =>
            if print_readably { format!("{:?}", s) } else { s.clone() },
        MalForm::Atom(MalAtom::Key(MalKey::Keyword(s))) => format!(":{}", s),
        MalForm::Atom(MalAtom::Number(n)) => format!("{}", n),
        MalForm::Atom(MalAtom::Symbol(s)) => format!("{}", s),
        MalForm::Atom(MalAtom::True) => format!("true"),
        MalForm::Atom(MalAtom::False) => format!("false"),
        MalForm::Atom(MalAtom::Nil) => format!("nil"),
        MalForm::List(xs) => format!("({})", pr_seq(xs, " ", print_readably)),
        MalForm::Vector(xs) => format!("[{}]", pr_seq(xs, " ", print_readably)),
        MalForm::HashMap(xs) => {
            let v: Vec<MalForm> = xs
                .into_iter()
                .flat_map(|(k, v)| vec![k.to_mal_form(), v.clone()])
                .collect();
            format!("{{{}}}", pr_seq(&v, " ", print_readably))
        },
    }
}

pub fn pr_seq(xs: &Vec<MalForm>, separator: &str, print_readably: bool) -> String {
    let mut res = String::new();
    let mut it = xs.into_iter();

    if let Some(x) = it.next() {
        res.push_str(&x.pr_str(print_readably));

        for x in it {
            res.push_str(separator);
            res.push_str(&x.pr_str(print_readably));
        }
    }

    res
}

impl fmt::Display for MalForm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", pr_str(self, true))
    }
}

impl fmt::Display for MalAtom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MalAtom::Key(s) => write!(f, "{}", s),
            MalAtom::Number(n) => write!(f, "{}", n),
            MalAtom::Symbol(s) => write!(f, "{}", s),
            MalAtom::True => write!(f, "true"),
            MalAtom::False => write!(f, "false"),
            MalAtom::Nil => write!(f, "nil"),
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
