mod readline;
mod types;
mod reader;
mod utils;
mod printer;

use rustyline::error::ReadlineError;
use types::{MalForm,MalError};

const PROMPT: &str = "user> ";
const HISTORY_FILE: &str = "mal_history.txt";

fn main() {
    let mut editor = readline::Reader::new(HISTORY_FILE);
    loop {
        match editor.readline(PROMPT) {
            Ok(line) => {
                match rep(&line) {
                    Ok(result) => println!("{}", result),
                    Err(error) => {
                        println!("Error: {}", error);
                    }
                }
            }
            Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => return,
            Err(error) => {
                println!("Error: {}", error);
                return;
            },
        }
    }
}

fn read(str: &String) -> Result<MalForm, MalError> {
    reader::read_str(str)
}

fn eval(str: MalForm) -> MalForm {
    str
}

fn print(form: MalForm) -> String {
    format!("{}", form)
}

fn rep(str: &String) -> Result<String, MalError> {
    Ok(print(eval(read(str)?)))
}
