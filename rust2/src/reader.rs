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
                println!("{:?}", errors);
                Err(MalError::ParseError)
            }
        }
        Err(err) => {
            println!("{}", err);
            Err(MalError::ParseError)
        },
    }
}
