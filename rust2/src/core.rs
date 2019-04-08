use std::rc::Rc;
use std::cell::RefCell;
use std::fs;

use crate::types::{MalForm,MalError,MalKey,MalNativeFn,MalResult,ToMalForm,Env};
use crate::printer::pr_seq;
use crate::reader::read_str;

pub fn get_namespace() -> Vec<(&'static str, MalForm)> {
    vec![
        ("+", binary_fn("+", |a,b| a + b)),
        ("-", binary_fn("-", |a,b| a - b)),
        ("*", binary_fn("*", |a,b| a * b)),
        ("/", binary_fn("/", |a,b| a / b)),
        ("<", binary_fn("<", |a,b| a < b)),
        ("<=", binary_fn("<=", |a,b| a <= b)),
        (">", binary_fn(">", |a,b| a > b)),
        (">=", binary_fn(">=", |a,b| a >= b)),
        ("prn", native_fn("prn", prn)),
        ("list", native_fn("list", list)),
        ("list?", native_fn("list?", list_q)),
        ("empty?", native_fn("empty?", empty_q)),
        ("count", native_fn("count", count)),
        ("=", native_fn("=", eq)),
        ("pr-str", native_fn("pr-str", pr_str)),
        ("str", native_fn("str", str)),
        ("println", native_fn("println", println)),
        ("read-string", native_fn("read-string", read_string)),
        ("slurp", native_fn("slurp", slurp)),
        ("atom", native_fn("atom", atom)),
        ("atom?", native_fn("atom?", atom_q)),
        ("deref", native_fn("deref", deref)),
        ("reset!", native_fn("reset!", reset_)),
        ("cons", native_fn("cons", cons)),
        ("concat", native_fn("concat", concat)),
        ("nth", native_fn("nth", nth)),
        ("first", native_fn("first", first)),
        ("rest", native_fn("rest", rest)),
    ]
}

pub fn native_fn<F: 'static>(name: &'static str, f: F) -> MalForm
    where F: Fn(Vec<MalForm>, &Rc<RefCell<Env>>) -> MalResult<MalForm>
{
    MalForm::NativeFn(name.to_string(), MalNativeFn(Rc::new(f)))
}

fn binary_fn<T>(name: &'static str, f: fn(f64, f64) -> T) -> MalForm
    where T: ToMalForm + 'static
{
    native_fn(name, move |vec: Vec<MalForm>, _| {
        match vec.as_slice() {
            [MalForm::Number(ref a), MalForm::Number(ref b)] => Ok(f(*a, *b).to_mal_form()),
            _ => Err(MalError::EvalError(format!("'{}': wrong arguments", name))),
        }
    })
}

fn list(args: Vec<MalForm>, _env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    Ok(MalForm::List(args))
}

fn list_q(args: Vec<MalForm>, _env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    let is_list = match args.get(0) {
        Some(MalForm::List(_)) => true,
        _ => false,
    };

    Ok(is_list.to_mal_form())
}

fn empty_q(args: Vec<MalForm>, _env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    let vec = match args.get(0) {
        Some(MalForm::List(v)) => v,
        Some(MalForm::Vector(v)) => v,
        Some(x) => return Err(MalError::EvalError(format!("'empty?' expects a list or a vector, {} was given", x))),
        None => return Err(MalError::EvalError(format!("'empty?' expects a list or a vector, nothing was given"))),
    };

    Ok(vec.is_empty().to_mal_form())
}

fn count(args: Vec<MalForm>, _env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    let vec = match args.get(0) {
        Some(MalForm::List(v)) => v,
        Some(MalForm::Vector(v)) => v,
        Some(MalForm::Nil) => return Ok(0.0.to_mal_form()),
        Some(x) => return Err(MalError::EvalError(format!("'count' expects a list or a vector, {} was given", x))),
        None => return Err(MalError::EvalError(format!("'count' expects a list or a vector, nothing was given"))),
    };

    Ok(MalForm::Number(vec.len() as f64))
}

fn eq(args: Vec<MalForm>, _env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    match args.as_slice() {
        [a, b] => Ok((a == b).to_mal_form()),
        _ => Err(MalError::EvalError(format!("'=' expects two arguments"))),
    }
}

fn prn(args: Vec<MalForm>, _env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    let res = pr_seq(&args, " ", true);
    println!("{}", res);
    Ok(().to_mal_form())
}

fn println(args: Vec<MalForm>, _env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    let res = pr_seq(&args, " ", false);
    println!("{}", res);
    Ok(().to_mal_form())
}

fn pr_str(args: Vec<MalForm>, _env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    Ok(pr_seq(&args, " ", true).to_mal_form())
}

fn str(args: Vec<MalForm>, _env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    Ok(pr_seq(&args, "", false).to_mal_form())
}

fn read_string(args: Vec<MalForm>, _env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    match args.get(0) {
        Some(MalForm::Key(MalKey::String(ref s))) => read_str(s),
        Some(x) => Err(MalError::EvalError(format!("'read-string': argument must be a string, {} was given", x))),
        _ => Err(MalError::EvalError(format!("'read-string': argument required"))),
    }
}

fn slurp(args: Vec<MalForm>, _env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    match args.get(0) {
        Some(MalForm::Key(MalKey::String(ref s))) => {
            let contents = fs::read_to_string(s);
            Ok(contents.map(|x| x.to_mal_form()).unwrap_or(().to_mal_form()))
        },
        Some(x) => Err(MalError::EvalError(format!("'slurp': argument must be a string, {} was given", x))),
        _ => Err(MalError::EvalError(format!("'slurp': argument required"))),
    }
}

fn atom(mut args: Vec<MalForm>, _env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    if args.len() < 1 {
        return Err(MalError::EvalError(format!("'atom': argument required")));
    }

    Ok(MalForm::Atom(Rc::new(RefCell::new(args.remove(0)))))
}

fn atom_q(args: Vec<MalForm>, _env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    Ok(if let Some(MalForm::Atom(_)) = args.get(0) {
        true
    } else {
        false
    }.to_mal_form())
}

fn deref(args: Vec<MalForm>, _env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    match args.get(0) {
        Some(MalForm::Atom(a)) => {
            Ok(a.borrow().clone())
        },
        Some(x) => Err(MalError::EvalError(format!("'deref': argument must be an atom, {} was given", x))),
        _ => Err(MalError::EvalError(format!("'deref': argument required"))),
    }
}

fn reset_(args: Vec<MalForm>, _env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    match (args.get(0), args.get(1)) {
        (Some(MalForm::Atom(a)), Some(x)) => {
            *a.borrow_mut() = x.clone();
            Ok(x.clone())
        },
        _ => Err(MalError::EvalError(format!("'reset!': wrong arguments"))),
    }
}

fn cons(args: Vec<MalForm>, _env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    match (args.get(0), args.get(1).and_then(|x| x.coerce_list())) {
        (Some(x), Some(xs)) => {
            let mut res = xs.clone();
            res.insert(0, x.clone());
            Ok(MalForm::List(res))
        },
        _ => Err(MalError::EvalError(format!("'cons': wrong arguments")))
    }
}

fn concat(args: Vec<MalForm>, _env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    let mut result = Vec::new();

    let mut it = args.into_iter();
    while let Some(ref mut x) = it.next() {
        match x {
            MalForm::List(ref mut xs) => {
                result.append(xs);
            },
            MalForm::Vector(ref mut xs) => {
                result.append(xs);
            },
            _ => return Err(MalError::EvalError(format!("'concat': arguments must be lists, {} given", x))),
        }
    }

    Ok(MalForm::List(result))
}

fn nth(args: Vec<MalForm>, _env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    match (args.get(0), args.get(1)) {
        (Some(xs_list), Some(MalForm::Number(i))) => {
            let xs = xs_list.coerce_list().ok_or(MalError::EvalError(format!("'nth': first argument is neither a list nor a vector")))?;
            Ok(xs.get(*i as usize).ok_or(MalError::EvalError(format!("'nth': index out of bound")))?.clone())
        },
        _ => Err(MalError::EvalError(format!("'nth': wrong arguments")))
    }
}

fn first(args: Vec<MalForm>, _env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    if let Some(MalForm::Nil) = args.get(0) {
        return Ok(MalForm::Nil);
    }

    match args.get(0).and_then(|x| x.coerce_list()) {
        Some(xs) => {
            Ok(xs.get(0).unwrap_or(&MalForm::Nil).clone())
        },
        _ => Err(MalError::EvalError(format!("'first': wrong arguments")))
    }
}

fn rest(args: Vec<MalForm>, _env: &Rc<RefCell<Env>>) -> MalResult<MalForm> {
    if let Some(MalForm::Nil) = args.get(0) {
        return Ok(MalForm::List(vec![]));
    }

    match args.get(0).and_then(|x| x.coerce_list()) {
        Some(xs) => {
            Ok(MalForm::List(xs.get(1 ..).unwrap_or(&[]).to_vec().clone()))
        },
        _ => Err(MalError::EvalError(format!("'rest': wrong arguments")))
    }
}
