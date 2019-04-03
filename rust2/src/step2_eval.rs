use std::collections::HashMap;
use std::rc::Rc;
use std::clone::Clone;

mod readline;
mod types;
mod reader;
mod utils;

use rustyline::error::ReadlineError;
use types::{MalForm,MalError,MalAtom,MalNativeFn};

const PROMPT: &str = "user> ";
const HISTORY_FILE: &str = "mal_history.txt";

fn binary_fn(f: fn(f32, f32) -> f32) -> Rc<dyn Fn(Vec<MalForm>) -> MalForm> {
    Rc::new(move |vec: Vec<MalForm>| {
        match vec.as_slice() {
            [MalForm::Atom(MalAtom::Number(ref a)), MalForm::Atom(MalAtom::Number(ref b))] => MalForm::Atom(MalAtom::Number(f(*a, *b))),
            _ => MalForm::Error(MalError::EvalError("Wrong argument".to_string())),
        }
    })
}

type Env = HashMap<String, MalForm>;

fn main() {
    let mut editor = readline::Reader::new(HISTORY_FILE);

    let mut repl_env = Env::new();
    repl_env.insert("+".to_string(),
                    MalForm::NativeFn("+".to_string(), MalNativeFn(binary_fn(|a,b| a + b))));
    repl_env.insert("-".to_string(),
                    MalForm::NativeFn("-".to_string(), MalNativeFn(binary_fn(|a,b| a - b))));
    repl_env.insert("*".to_string(),
                    MalForm::NativeFn("*".to_string(), MalNativeFn(binary_fn(|a,b| a * b))));
    repl_env.insert("/".to_string(),
                    MalForm::NativeFn("/".to_string(), MalNativeFn(binary_fn(|a,b| a / b))));

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

fn read(str: &String) -> Result<MalForm, MalError> {
    reader::read_str(str)
}

fn eval_ast(ast: &MalForm, env: &mut Env) -> MalForm {
    match ast {
        MalForm::Atom(MalAtom::Symbol(ref sym)) => match env.get(sym) {
            Some(val) => val.clone(),
            None => MalForm::Error(MalError::EvalError(format!("'{}' not found", sym))),
        },
        MalForm::List(ref list) =>
            MalForm::List(list.into_iter().map(|x| eval(x, env)).collect()),
        MalForm::Vector(ref list) =>
            MalForm::Vector(list.into_iter().map(|x| eval(x, env)).collect()),
        MalForm::HashMap(ref hash) =>
            MalForm::HashMap(hash.into_iter().map(|(k, v)| (k.clone(), eval(v, env))).collect()),
        x => x.clone(),
    }
}

fn eval(ast: &MalForm, env: &mut Env) -> MalForm {
    if let MalForm::List(xs) = ast {
        if xs.is_empty() {
            ast.clone()
        } else if let MalForm::List(v) = eval_ast(ast, env) {
            let f_ast = &v.as_slice()[0];
            let args = &v.as_slice()[1 ..];
            match f_ast {
                MalForm::NativeFn(_, MalNativeFn(f)) => f(args.to_vec()),
                _ => MalForm::Error(MalError::EvalError(format!("'{}' is not a function", f_ast))),
            }
        } else {
            unreachable!();
        }
    } else {
        eval_ast(ast, env)
    }
}

fn print(form: MalForm) -> String {
    format!("{}", form)
}

fn rep(str: &String, env: &mut Env) -> Result<String, MalError> {
    Ok(print(eval(&read(str)?, env)))
}
