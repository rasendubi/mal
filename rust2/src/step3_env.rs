use std::collections::HashMap;
use std::rc::Rc;
use std::clone::Clone;

mod readline;
mod types;
mod reader;
mod utils;
mod env;

use rustyline::error::ReadlineError;
use types::{MalForm,MalError,MalAtom,MalNativeFn,MalResult};
use env::Env;

const PROMPT: &str = "user> ";
const HISTORY_FILE: &str = "mal_history.txt";

fn binary_fn(name: &'static str, f: fn(f32, f32) -> f32) -> MalForm {
    MalForm::NativeFn(name.to_string(), MalNativeFn(Rc::new(move |vec: Vec<MalForm>| {
        match vec.as_slice() {
            [MalForm::Atom(MalAtom::Number(ref a)), MalForm::Atom(MalAtom::Number(ref b))] => Ok(MalForm::Atom(MalAtom::Number(f(*a, *b)))),
            _ => Err(MalError::EvalError(format!("'{}': wrong arguments", name))),
        }
    })))
}

fn main() {
    let mut editor = readline::Reader::new(HISTORY_FILE);

    let mut repl_env = Env::new(None);
    repl_env.set("+".to_string(), binary_fn("+", |a,b| a + b));
    repl_env.set("-".to_string(), binary_fn("-", |a,b| a - b));
    repl_env.set("*".to_string(), binary_fn("*", |a,b| a * b));
    repl_env.set("/".to_string(), binary_fn("/", |a,b| a / b));

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
        MalForm::Atom(MalAtom::Symbol(ref sym)) => env.get(&sym)?.clone(),
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

fn process_bindings(bindings_ast: &MalForm, env: &mut Env) -> MalResult<()> {
    let vec = match bindings_ast {
        MalForm::List(v) => v,
        MalForm::Vector(v) => v,
        _ => return Err(MalError::EvalError(format!("'let*': bindings list must be either a list or vector, {} was given", bindings_ast))),
    };

    let mut b = vec.into_iter();

    while let Some(key_ast) = b.next() {
        if let MalForm::Atom(MalAtom::Symbol(ref key)) = key_ast {
            let val_ast = b.next().ok_or(MalError::EvalError(format!("'let*': mising value for {}", key)))?;
            let val = eval(val_ast, env)?;

            env.set(key.clone(), val.clone());
        } else {
            return Err(MalError::EvalError(format!("'let*': binding name must be a symbol, {} was given", key_ast)));
        }
    }

    Ok(())
}

fn eval_def_(args: &[MalForm], env: &mut Env) -> MalResult<MalForm> {
    match args {
        [MalForm::Atom(MalAtom::Symbol(name)), val_ast] => {
            let val = eval(val_ast, env)?;
            env.set(name.clone(), val.clone());
            Ok(val)
        },
        [_, _] => Err(MalError::EvalError("'def!': first argument must be a symbol".to_string())),
        _ => Err(MalError::EvalError("'def!' requires at least 2 arguments".to_string())),
    }
}

fn eval_let_(args: &[MalForm], env: &mut Env) -> MalResult<MalForm> {
    match args {
        [bindings_ast, value_ast] => {
            let mut new_env = Env::new(Some(env));
            process_bindings(bindings_ast, &mut new_env)?;
            eval(value_ast, &mut new_env)
        },
        _ => Err(MalError::EvalError("'let*' requires at least 2 arguments".to_string())),
    }
}

fn eval_fn(ast: &MalForm, env: &mut Env) -> MalResult<MalForm> {
    if let MalForm::List(xs) = eval_ast(ast, env)? {
        match &xs[0] {
            MalForm::NativeFn(_, MalNativeFn(f)) => {
                let args = &xs[1 ..];
                f(args.to_vec())
            },
            head => return Err(MalError::EvalError(format!("'{}' is not a function", head))),
        }
    } else {
        unreachable!();
    }
}

fn eval(ast: &MalForm, env: &mut Env) -> MalResult<MalForm> {
    Ok(if let MalForm::List(xs) = ast {
        if xs.is_empty() {
            ast.clone()
        } else {
            let s = xs.as_slice();
            match &s[0] {
                MalForm::Atom(MalAtom::Symbol(sym)) if sym == "def!" => eval_def_(&s[1..], env)?,
                MalForm::Atom(MalAtom::Symbol(sym)) if sym == "let*" => eval_let_(&s[1..], env)?,
                _ => eval_fn(ast, env)?,
            }
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
