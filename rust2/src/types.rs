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
        MalForm::Bool(*self)
    }
}

impl ToMalForm for f64 {
    fn to_mal_form(&self) -> MalForm {
        MalForm::Number(*self)
    }
}

impl ToMalForm for String {
    fn to_mal_form(&self) -> MalForm {
        MalForm::Key(MalKey::String(self.clone()))
    }
}

impl ToMalForm for () {
    fn to_mal_form(&self) -> MalForm {
        MalForm::Nil
    }
}

impl ToMalForm for MalKey {
    fn to_mal_form(&self) -> MalForm {
        MalForm::Key(self.clone())
    }
}
