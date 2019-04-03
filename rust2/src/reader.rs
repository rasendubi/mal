use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub reader);

use crate::types::{MalForm, MalError};

pub fn read_str(str: &str) -> Result<MalForm, MalError> {
    let mut errors = Vec::new();

    match reader::FormParser::new().parse(&mut errors, str) {
        Ok(res) => {
            if errors.is_empty() {
                Ok(res)
            } else {
                Err(MalError::ParseError(errors[0].clone()))
            }
        }
        Err(err) => {
            Err(MalError::ParseError(err.map_token(|t| (t.0, t.1))))
        },
    }
}
