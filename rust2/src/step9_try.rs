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
use types::{MalForm,MalKey,MalError,MalNativeFn,MalFn,MalResult,ToMalForm};
use env::Env;

const PROMPT: &str = "user> ";
const HISTORY_FILE: &str = "mal_history.txt";

fn main() {
    let mut editor = readline::Reader::new(HISTORY_FILE);

    let repl_env = Rc::new(RefCell::new(Env::new(None)));

    for (name, val) in core::get_namespace() {
        repl_env.borrow_mut().set(name.to_string(), val.clone());
    }

    {
        let repl_env_clone = repl_env.clone(); // to be moved into eval
        repl_env.borrow_mut().set("eval".to_string(), core::native_fn("eval", move |args, _| {
            let ast = args.get(0).ok_or(MalError::EvalError(format!("'eval': argument required")))?;
            eval(&ast, &repl_env_clone)
        }));
    }
    {
        let repl_env_clone = repl_env.clone(); // to be moved into eval
        repl_env.borrow_mut().set("swap!".to_string(), core::native_fn("swap!", move |args, _| {
            if args.len() < 2 {
                return Err(MalError::EvalError(format!("'swap!': at least 2 argument required")));
            }

            if let MalForm::Atom(ref atom) = args[0] {
                let cur = { atom.borrow().clone() };

                let mut f_args = vec![args[1].clone(), cur];
                for arg in &args[2 ..] {
                    f_args.push(arg.clone());
                }

                // Not sure if that should be repl env or calling env
                let res = eval(&MalForm::List(f_args), &repl_env_clone)?;

                *atom.borrow_mut() = res.clone();

                Ok(res)
            } else {
                return Err(MalError::EvalError(format!("'swap!': first argument must be an atom")));
            }
        }));
    }

    repl_env.borrow_mut().set(
        "*ARGV*".to_string(),
        MalForm::List(std::env::args().skip(2).map(|x| x.to_mal_form()).collect::<Vec<MalForm>>()));

    let _ = rep(r#"(def! not (fn* (a) (if a false true)))"#, &repl_env);
    let _ = rep(r#"(def! load-file (fn* (f) (eval (read-string (str "(do " (slurp f) ")")))))"#, &repl_env);
    let _ = rep(r#"(defmacro! cond (fn* (& xs) (if (> (count xs) 0) (list 'if (first xs) (if (> (count xs) 1) (nth xs 1) (throw "odd number of forms to cond")) (cons 'cond (rest (rest xs)))))))"#, &repl_env);
    let _ = rep(r#"(defmacro! or (fn* (& xs) (if (empty? xs) nil (if (= 1 (count xs)) (first xs) `(let* (or_FIXME ~(first xs)) (if or_FIXME or_FIXME (or ~@(rest xs))))))))"#, &repl_env);

    if let Some(file) = std::env::args().nth(1) {
        let _ = rep(&format!("(load-file {:?})", file), &repl_env);
        return;
    }

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

fn eval_defmacro_(args: &[MalForm], env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    match args {
        [MalForm::Symbol(name), val_ast] => {
            if let MalForm::MalFn(ref f) = eval(val_ast, env)? {
                let mut m: MalFn = (**f).clone();
                m.is_macro = true;

                let val = MalForm::MalFn(Rc::new(m));
                env.borrow_mut().set(name.clone(), val.clone());
                Ok(val)
            } else {
                Err(MalError::EvalError(format!("'defmacro!': argument must be fn*")))
            }
        },
        [_, _] => Err(MalError::EvalError("'defmacro!': first argument must be a symbol".to_string())),
        _ => Err(MalError::EvalError("'defmacro!' requires 2 arguments".to_string())),
    }
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

    Ok(MalForm::MalFn(Rc::new(MalFn::new(outer, bindings, body, eval))))
}

fn is_pair(ast: &MalForm) -> bool {
    match ast {
        MalForm::List(xs) => !xs.is_empty(),
        MalForm::Vector(xs) => !xs.is_empty(),
        _ => false,
    }
}

fn quasiquote(ast: &MalForm, env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    if !is_pair(ast) {
        return Ok(MalForm::List(vec![MalForm::Symbol("quote".to_string()), ast.clone()]));
    }

    // xs contains at least 1 element
    let xs = ast.coerce_list().unwrap();
    let x = &xs[0];

    if *x == MalForm::Symbol("unquote".to_string()) {
        return xs.get(1).map(|x| x.clone()).ok_or(MalError::EvalError(format!("'unquote': argument required")));
    }

    if is_pair(x) {
        let ys = x.coerce_list().unwrap();
        let y = &ys[0];
        if *y == MalForm::Symbol("splice-unquote".to_string()) {
            let res = vec![
                MalForm::Symbol("concat".to_string()),
                ys.get(1)
                    .map(|x| x.clone())
                    .ok_or(MalError::EvalError(format!("'splice-unquote': at least one argument expected")))?,
                quasiquote(&MalForm::List(xs.into_iter().skip(1).map(|x| x.clone()).collect()), env)?,
            ];
            return Ok(MalForm::List(res));
        }
    }

    let res = vec![
        MalForm::Symbol("cons".to_string()),
        quasiquote(&x, env)?,
        quasiquote(&MalForm::List(xs.into_iter().skip(1).map(|x| x.clone()).collect()), env)?,
    ];
    Ok(MalForm::List(res))
}

fn is_macro_call(ast: &MalForm, env: &Rc<RefCell<Env>>) -> MalResult<bool> {
    match ast {
        MalForm::List(xs) => {
            match xs.get(0) {
                Some(MalForm::Symbol(ref x)) => {
                    match env.borrow().get(x) {
                        Ok(MalForm::MalFn(f)) => Ok(f.is_macro),
                        _ => Ok(false),
                    }
                },
                _ => Ok(false),
            }
        },
        _ => Ok(false),
    }
}

fn macroexpand(ast: &MalForm, env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    let mut ast = ast.clone();

    while is_macro_call(&ast, env)? {
        let xs = ast.coerce_list().unwrap(); // should be here because is_macro_call is true
        if let MalForm::Symbol(ref f_name) = xs[0] {
            let f_form = env.borrow().get(f_name).unwrap();

            match &f_form {
                MalForm::MalFn(f) => {
                    let f_env = Rc::new(RefCell::new(Env::new_fn_closure(
                        Some(f.env.clone()), &f.params, &xs[1 ..])?));
                    ast = eval(&f.ast, &f_env)?;
                    // continue expansion
                },
                head => return Err(MalError::EvalError(format!("'{}' is not a function", head))),
            }
        }
    }

    Ok(ast)
}

fn eval(ast: &MalForm, env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    let mut ast = ast.clone();
    let mut env = env.clone();

    loop {
        // println!("Evaluating {}", ast);

        if let MalForm::List(xs) = &ast {
            if xs.is_empty() {
                return Ok(ast)
            }
        } else {
            return eval_ast(&ast, &env);
        }

        ast = macroexpand(&ast, &env)?;

        if let MalForm::List(xs) = &ast {
            if xs.is_empty() {
                return Ok(ast)
            }

            let s = xs.as_slice();
            match &s[0] {
                MalForm::Symbol(sym) if sym == "def!" => return eval_def_(&s[1..], &env),
                MalForm::Symbol(sym) if sym == "defmacro!" => return eval_defmacro_(&s[1..], &env),
                MalForm::Symbol(sym) if sym == "let*" =>
                    match &s[1..] {
                        [bindings_ast, value_ast] => {
                            let new_env = Rc::new(RefCell::new(Env::new(Some(env.clone()))));
                            process_bindings(bindings_ast, &new_env)?;

                            ast = value_ast.clone();
                            env = new_env.clone();
                            // tco
                        },
                        _ => return Err(MalError::EvalError("'let*' requires at least 2 arguments".to_string())),
                    },
                MalForm::Symbol(sym) if sym == "do" => {
                    for arg in &s[1 .. s.len()-1] {
                        let _ = eval(&arg, &env)?;
                    }

                    ast = s[s.len() - 1].clone();
                    // tco
                },
                MalForm::Symbol(sym) if sym == "if" => {
                    let cond_ast = s.get(1).ok_or(MalError::EvalError(format!("Missing condition for 'if'")))?;
                    let cond = eval(cond_ast, &env)?;

                    let i = match cond {
                        MalForm::Bool(false) | MalForm::Nil => 3,
                        _ => 2,
                    };

                    ast = s.get(i).unwrap_or(&MalForm::Nil).clone();
                    // tco
                },
                MalForm::Symbol(sym) if sym == "quote" => {
                    match s.get(1) {
                        Some(x) => return Ok(x.clone()),
                        _ => return Err(MalError::EvalError(format!("'quote': must have an argument"))),
                    }
                },
                MalForm::Symbol(sym) if sym == "quasiquote" => {
                    match s.get(1) {
                        Some(x) => {
                            ast = quasiquote(&x, &env)?;
                            // tco
                        },
                        _ => return Err(MalError::EvalError(format!("'quasiquote': argument required"))),
                    }
                },
                MalForm::Symbol(sym) if sym == "macroexpand" => {
                    return macroexpand(
                        s.get(1).ok_or(MalError::EvalError(format!("'macroexpand': argument required")))?,
                        &env);
                },
                MalForm::Symbol(sym) if sym == "fn*" => return eval_fn_(&s[1..], &env),
                MalForm::Symbol(sym) if sym == "try*" => {
                    let body = s.get(1).ok_or(MalError::EvalError(format!("'try*': body required")))?;

                    let val = eval(body, &env);

                    match val {
                        x@Ok(_) => return x,
                        Err(err) => {
                            let catch_clause = s.get(2)
                                .and_then(|x| x.coerce_list())
                                .ok_or(err.clone())?;

                            if Some(&MalForm::Symbol("catch*".to_string())) != catch_clause.get(0) {
                                return Err(err);
                            }

                            let catch_symbol = if let Some(MalForm::Symbol(catch_symbol)) = catch_clause.get(1) {
                                catch_symbol
                            } else {
                                return Err(MalError::EvalError(format!("'catch*': exception symbol is required")));
                            };

                            let catch_body = if let Some(body) = catch_clause.get(2) {
                                body
                            } else {
                                return Err(MalError::EvalError(format!("'catch*': body is required")));
                            };

                            let mal_error = match err {
                                MalError::EvalError(err) => MalForm::Key(MalKey::String(err)),
                                MalError::ParseError(_) => MalForm::Key(MalKey::String(format!("parse error"))),
                                MalError::MalException(x) => x,
                            };

                            let catch_env = Rc::new(RefCell::new(
                                Env::new_fn_closure(
                                    Some(env.clone()),
                                    &[catch_symbol.clone()],
                                    &[mal_error])?));
                            return eval(catch_body, &catch_env);
                        },
                    }
                },
                _ => if let MalForm::List(xs) = eval_ast(&ast, &env)? {
                    match &xs[0] {
                        MalForm::NativeFn(_, MalNativeFn(f)) => {
                            let args = &xs[1 ..];
                            return f(args.to_vec(), &env);
                        },
                        MalForm::MalFn(f) => {
                            env = Rc::new(RefCell::new(Env::new_fn_closure(
                                Some(f.env.clone()), &f.params, &xs[1 ..])?));
                            ast = f.ast.clone();
                            // tco
                        },
                        head => return Err(MalError::EvalError(format!("'{}' is not a function", head))),
                    }
                } else {
                    unreachable!();
                }
            }
        } else {
            return eval_ast(&ast, &env);
        }
    }
}

fn print(form: MalForm) -> String {
    format!("{}", form)
}

fn rep(str: &str, env: &Rc<RefCell<Env>>) -> Result<String, MalError> {
    Ok(print(eval(&read(str)?, env)?))
}
