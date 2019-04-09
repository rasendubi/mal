#![allow(dead_code)]
use std::fmt;

use crate::types::{MalForm,MalKey,ToMalForm};

impl MalForm {
    pub fn pr_str(&self, print_readably: bool) -> String {
        pr_str(self, print_readably)
    }
}

pub fn escape_string(s: &String) -> String {
    let mut res = String::new();

    let mut it = s.chars();
    while let Some(c) = it.next() {
        match c {
            '\\' => res.push_str("\\\\"),
            '\n' => res.push_str("\\n"),
            '"' => res.push_str("\\\""),
            _ => res.push(c),
        }
    }

    res
}

pub fn pr_str(x: &MalForm, print_readably: bool) -> String {
    match x {
        MalForm::NativeFn(name, _) => format!("#<{}>", name),
        MalForm::MalFn(_) => format!("#<fn*>"),
        MalForm::Key(MalKey::String(s)) =>
            if print_readably { format!("\"{}\"", escape_string(s)) } else { s.clone() },
        MalForm::Key(MalKey::Keyword(s)) => format!(":{}", s),
        MalForm::Number(n) => format!("{}", n),
        MalForm::Symbol(s) => format!("{}", s),
        MalForm::Bool(true) => format!("true"),
        MalForm::Bool(false) => format!("false"),
        MalForm::Nil => format!("nil"),
        MalForm::List(xs) => format!("({})", pr_seq(xs, " ", print_readably)),
        MalForm::Vector(xs) => format!("[{}]", pr_seq(xs, " ", print_readably)),
        MalForm::HashMap(xs) => {
            let v: Vec<MalForm> = xs
                .into_iter()
                .flat_map(|(k, v)| vec![k.to_mal_form(), v.clone()])
                .collect();
            format!("{{{}}}", pr_seq(&v, " ", print_readably))
        },
        MalForm::Atom(x) => format!("(atom {})", pr_str(&x.borrow(), print_readably)),
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

impl fmt::Display for MalKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MalKey::String(s) => write!(f, "{:?}", s),
            MalKey::Keyword(s) => write!(f, ":{}", s),
        }
    }
}
