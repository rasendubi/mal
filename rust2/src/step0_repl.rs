extern crate rustyline;

mod readline;

use rustyline::error::ReadlineError;

const PROMPT: &str = "user> ";

fn main() {
    loop {
        match rep() {
            Ok(_) => (),
            Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => return,
            Err(error) => {
                println!("Error: {}", error);
            },
        }
    }
}

fn read() -> rustyline::Result<String> {
    readline::read(PROMPT)
}

fn eval(str: &String) -> &String {
    str
}

fn print(str: &String) {
    println!("{}", str);
}

fn rep() -> rustyline::Result<()> {
    Ok(print(eval(&read()?)))
}
