use std::io;
use std::io::Write;
use std::io::ErrorKind;

const PROMPT: &str = "user> ";

fn main() {
    loop {
        print!("{}", PROMPT);
        io::stdout().flush().unwrap();

        match rep() {
            Ok(_) => (),
            Err(error) => {
                if error.kind() == ErrorKind::UnexpectedEof {
                    return;
                }
                println!("Error: {}", error);
            },
        }
    }
}

fn read() -> Result<String, std::io::Error> {
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(n) => if n == 0 {
            Err(std::io::Error::new(ErrorKind::UnexpectedEof, "End of file"))
        } else {
            Ok(input)
        },
        Err(error) => Err(error),
    }
}

fn eval(str: &String) -> &String {
    str
}

fn print(str: &String) {
    print!("{}", str);
}

fn rep() -> Result<(), std::io::Error> {
    Ok(print(eval(&read()?)))
}
