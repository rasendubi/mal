use std::fmt;

#[derive(Debug, Clone)]
pub enum MalError {
    ParseError(lalrpop_util::ParseError<usize, (usize, String), &'static str>),
    EvalError(String),
}

pub type MalResult<T> = Result<T, MalError>;

impl fmt::Display for MalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MalError::ParseError(err) =>
                write!(f, "{}", err.clone().map_token(|(_size,s)| s)),
            MalError::EvalError(msg) => write!(f, "Evaluation Error: {}", msg),
        }
    }
}
