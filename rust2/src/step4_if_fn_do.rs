use std::collections::HashMap;
use std::rc::Rc;
use std::clone::Clone;
use std::cell::RefCell;

mod readline;
mod types;
mod reader;
mod utils;
mod env;
mod core;
mod printer;

use rustyline::error::ReadlineError;
use types::{MalForm,MalError,MalNativeFn,MalResult};
use env::Env;

const PROMPT: &str = "user> ";
const HISTORY_FILE: &str = "mal_history.txt";

fn main() {
    let mut editor = readline::Reader::new(HISTORY_FILE);

    let mut repl_env = Env::new(None);
    for (name, val) in core::get_namespace() {
        repl_env.set(name.to_string(), val.clone());
    }
    let repl_env = Rc::new(RefCell::new(repl_env));

    let _ = rep("(def! not (fn* (a) (if a false true)))", &repl_env);

    loop {
        match editor.readline(PROMPT) {
            Ok(line) => {
                match rep(&line, &repl_env) {
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

fn read(str: &str) -> MalResult<MalForm> {
    reader::read_str(str)
}

fn eval_ast(ast: &MalForm, env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    Ok(match ast {
        MalForm::Symbol(ref sym) => env.borrow().get(&sym)?.clone(),
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

fn process_bindings(bindings_ast: &MalForm, env: &Rc<RefCell<Env>>) -> MalResult<()> {
    let vec = match bindings_ast {
        MalForm::List(v) => v,
        MalForm::Vector(v) => v,
        _ => return Err(MalError::EvalError(format!("'let*': bindings list must be either a list or vector, {} was given", bindings_ast))),
    };

    let mut b = vec.into_iter();

    while let Some(key_ast) = b.next() {
        if let MalForm::Symbol(ref key) = key_ast {
            let val_ast = b.next().ok_or(MalError::EvalError(format!("'let*': mising value for {}", key)))?;
            let val = eval(val_ast, env)?;

            env.borrow_mut().set(key.clone(), val.clone());
        } else {
            return Err(MalError::EvalError(format!("'let*': binding name must be a symbol, {} was given", key_ast)));
        }
    }

    Ok(())
}

fn eval_def_(args: &[MalForm], env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    match args {
        [MalForm::Symbol(name), val_ast] => {
            let val = eval(val_ast, env)?;
            env.borrow_mut().set(name.clone(), val.clone());
            Ok(val)
        },
        [_, _] => Err(MalError::EvalError("'def!': first argument must be a symbol".to_string())),
        _ => Err(MalError::EvalError("'def!' requires at least 2 arguments".to_string())),
    }
}

fn eval_let_(args: &[MalForm], env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    match args {
        [bindings_ast, value_ast] => {
            let new_env = Rc::new(RefCell::new(Env::new(Some(env.clone()))));
            process_bindings(bindings_ast, &new_env)?;
            eval(value_ast, &new_env)
        },
        _ => Err(MalError::EvalError("'let*' requires at least 2 arguments".to_string())),
    }
}

fn eval_fn(ast: &MalForm, env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    if let MalForm::List(xs) = eval_ast(ast, env)? {
        match &xs[0] {
            MalForm::NativeFn(_, MalNativeFn(f)) => {
                let args = &xs[1 ..];
                f(args.to_vec(), env)
            },
            head => return Err(MalError::EvalError(format!("'{}' is not a function", head))),
        }
    } else {
        unreachable!();
    }
}

fn eval_do(args: &[MalForm], env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    let mut result = MalForm::Nil;

    for arg in args {
        result = eval(&arg, env)?;
    }

    Ok(result)
}

fn eval_if(args: &[MalForm], env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    let cond_ast = args.get(0).ok_or(MalError::EvalError(format!("Missing condition for 'if'")))?;
    let cond = eval(cond_ast, env)?;

    let i = match cond {
        MalForm::Bool(false) | MalForm::Nil => 2,
        _ => 1,
    };

    let arg = args.get(i).unwrap_or(&MalForm::Nil);
    eval(arg, env)
}

fn get_binds(form: &MalForm) -> MalResult<Vec<String>> {
    let v = match form {
        MalForm::List(x) => x,
        MalForm::Vector(x) => x,
        _ => return Err(MalError::EvalError(format!("'fn*' bindings list must be a list or vector, {} given", form))),
    };

    let res: MalResult<Vec<_>> = v.iter().map(|x| match x {
        MalForm::Symbol(name) => Ok(name.clone()),
        _ => Err(MalError::EvalError(format!("'fn*' bindings must be symbols, {} given", x))),
    }).collect();

    res
}

fn eval_fn_(args: &[MalForm], env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    let outer = env.clone();

    let bindings = get_binds(&args[0])?;
    let body = args[1].clone();

    Ok(MalForm::NativeFn("fn*".to_string(), MalNativeFn(Rc::new(move |params, _env| {
        let env = Rc::new(RefCell::new(Env::new_fn_closure(Some(outer.clone()), &bindings, &params)?));

        eval(&body, &env)
    }))))
}

fn eval(ast: &MalForm, env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    Ok(if let MalForm::List(xs) = ast {
        if xs.is_empty() {
            ast.clone()
        } else {
            let s = xs.as_slice();
            match &s[0] {
                MalForm::Symbol(sym) if sym == "def!" => eval_def_(&s[1..], env)?,
                MalForm::Symbol(sym) if sym == "let*" => eval_let_(&s[1..], env)?,
                MalForm::Symbol(sym) if sym == "do" => eval_do(&s[1..], env)?,
                MalForm::Symbol(sym) if sym == "if" => eval_if(&s[1..], env)?,
                MalForm::Symbol(sym) if sym == "fn*" => eval_fn_(&s[1..], env)?,
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

fn rep(str: &str, env: &Rc<RefCell<Env>>) -> Result<String, MalError> {
    Ok(print(eval(&read(str)?, env)?))
}
