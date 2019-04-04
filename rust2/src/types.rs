#![allow(dead_code)]
pub mod error;
pub mod ast;

pub use error::*;
pub use ast::*;

pub trait ToMalForm {
    fn to_mal_form(&self) -> MalForm;
}

impl ToMalForm for bool {
    fn to_mal_form(&self) -> MalForm {
        MalForm::Atom(if *self {MalAtom::True} else {MalAtom::False})
    }
}

impl ToMalForm for f64 {
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
