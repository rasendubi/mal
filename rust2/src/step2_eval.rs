use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::clone::Clone;

mod readline;
mod types;
mod reader;
mod utils;
mod printer;

use rustyline::error::ReadlineError;
use types::{MalForm,MalError,MalNativeFn,MalResult,Env as NewEnv};

const PROMPT: &str = "user> ";
const HISTORY_FILE: &str = "mal_history.txt";

fn binary_fn(name: &'static str, f: fn(f64, f64) -> f64) -> MalForm {
    MalForm::NativeFn(name.to_string(), MalNativeFn(Rc::new(move |vec: Vec<MalForm>, _| {
        match vec.as_slice() {
            [MalForm::Number(ref a), MalForm::Number(ref b)] => Ok(MalForm::Number(f(*a, *b))),
            _ => Err(MalError::EvalError(format!("'{}': wrong arguments", name))),
        }
    })))
}

type Env = HashMap<String, MalForm>;

fn main() {
    let mut editor = readline::Reader::new(HISTORY_FILE);

    let mut repl_env = Env::new();
    repl_env.insert("+".to_string(), binary_fn("+", |a,b| a + b));
    repl_env.insert("-".to_string(), binary_fn("-", |a,b| a - b));
    repl_env.insert("*".to_string(), binary_fn("*", |a,b| a * b));
    repl_env.insert("/".to_string(), binary_fn("/", |a,b| a / b));

    loop {
        match editor.readline(PROMPT) {
            Ok(line) => {
                match rep(&line, &mut repl_env) {
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

fn read(str: &String) -> MalResult<MalForm> {
    reader::read_str(str)
}

fn eval_ast(ast: &MalForm, env: &mut Env) -> MalResult<MalForm> {
    Ok(match ast {
        MalForm::Symbol(ref sym) => match env.get(sym) {
            Some(val) => val.clone(),
            None => return Err(MalError::EvalError(format!("'{}' not found", sym))),
        },
        MalForm::List(ref list) => {
            let res: Result<Vec<_>, _> = list.into_iter().map(|x| eval(x, env)).collect();
            MalForm::List(res?)
        },
        MalForm::Vector(ref list) => {
            let res: Result<Vec<_>, _> = list.into_iter().map(|x| eval(x, env)).collect();
            MalForm::Vector(res?)
        },
        MalForm::HashMap(ref hash) => {
            let res: MalResult<HashMap<_, _>> = hash.into_iter().map(|(k, v)| Ok((k.clone(), eval(v, env)?))).collect();
            MalForm::HashMap(res?)
        },
        x => x.clone(),
    })
}

fn eval(ast: &MalForm, env: &mut Env) -> MalResult<MalForm> {
    Ok(if let MalForm::List(xs) = ast {
        if xs.is_empty() {
            ast.clone()
        } else if let MalForm::List(v) = eval_ast(ast, env)? {
            let f_ast = &v.as_slice()[0];
            let args = &v.as_slice()[1 ..];
            match f_ast {
                MalForm::NativeFn(_, MalNativeFn(f)) => f(args.to_vec(), &Rc::new(RefCell::new(NewEnv::new(None))))?,
                _ => return Err(MalError::EvalError(format!("'{}' is not a function", f_ast))),
            }
        } else {
            unreachable!();
        }
    } else {
        eval_ast(ast, env)?
    })
}

fn print(form: MalForm) -> String {
    format!("{}", form)
}

fn rep(str: &String, env: &mut Env) -> Result<String, MalError> {
    Ok(print(eval(&read(str)?, env)?))
}
