extern crate rustyline;

mod readline;

use rustyline::error::ReadlineError;

const PROMPT: &str = "user> ";
const HISTORY_FILE: &str = "mal_history.txt";

fn main() {
    let mut editor = readline::Reader::new(HISTORY_FILE);
    loop {
        match editor.readline(PROMPT) {
            Ok(line) => println!("{}", rep(&line)),
            Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => return,
            Err(error) => {
                println!("Error: {}", error);
                return;
            },
        }
    }
}

fn read(str: &String) -> &String {
    str
}

fn eval(str: &String) -> &String {
    str
}

fn print(str: &String) -> &String {
    str
}

fn rep(str: &String) -> &String {
    print(eval(read(str)))
}
