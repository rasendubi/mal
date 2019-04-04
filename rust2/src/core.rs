use std::rc::Rc;

use crate::types::{MalForm,MalError,MalAtom,MalNativeFn,MalResult,ToMalForm};
use crate::printer::pr_seq;

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
    ]
}

fn native_fn<F: 'static>(name: &'static str, f: F) -> MalForm
    where F: Fn(Vec<MalForm>) -> MalResult<MalForm>
{
    MalForm::NativeFn(name.to_string(), MalNativeFn(Rc::new(f)))
}

fn binary_fn<T>(name: &'static str, f: fn(f32, f32) -> T) -> MalForm
    where T: ToMalForm + 'static
{
    native_fn(name, move |vec: Vec<MalForm>| {
        match vec.as_slice() {
            [MalForm::Atom(MalAtom::Number(ref a)), MalForm::Atom(MalAtom::Number(ref b))] => Ok(f(*a, *b).to_mal_form()),
            _ => Err(MalError::EvalError(format!("'{}': wrong arguments", name))),
        }
    })
}

fn list(args: Vec<MalForm>) -> MalResult<MalForm> {
    Ok(MalForm::List(args))
}

fn list_q(args: Vec<MalForm>) -> MalResult<MalForm> {
    let is_list = match args.get(0) {
        Some(MalForm::List(_)) => true,
        _ => false,
    };

    Ok(is_list.to_mal_form())
}

fn empty_q(args: Vec<MalForm>) -> MalResult<MalForm> {
    let vec = match args.get(0) {
        Some(MalForm::List(v)) => v,
        Some(MalForm::Vector(v)) => v,
        Some(x) => return Err(MalError::EvalError(format!("'empty?' expects a list or a vector, {} was given", x))),
        None => return Err(MalError::EvalError(format!("'empty?' expects a list or a vector, nothing was given"))),
    };

    Ok(vec.is_empty().to_mal_form())
}

fn count(args: Vec<MalForm>) -> MalResult<MalForm> {
    let vec = match args.get(0) {
        Some(MalForm::List(v)) => v,
        Some(MalForm::Vector(v)) => v,
        Some(MalForm::Atom(MalAtom::Nil)) => return Ok(0.0.to_mal_form()),
        Some(x) => return Err(MalError::EvalError(format!("'count' expects a list or a vector, {} was given", x))),
        None => return Err(MalError::EvalError(format!("'count' expects a list or a vector, nothing was given"))),
    };

    Ok(MalForm::Atom(MalAtom::Number(vec.len() as f32)))
}

fn eq(args: Vec<MalForm>) -> MalResult<MalForm> {
    match args.as_slice() {
        [a, b] => Ok((a == b).to_mal_form()),
        _ => Err(MalError::EvalError(format!("'=' expects two arguments"))),
    }
}

fn prn(args: Vec<MalForm>) -> MalResult<MalForm> {
    let res = pr_seq(&args, " ", true);
    println!("{}", res);
    Ok(().to_mal_form())
}

fn println(args: Vec<MalForm>) -> MalResult<MalForm> {
    let res = pr_seq(&args, " ", false);
    println!("{}", res);
    Ok(().to_mal_form())
}

fn pr_str(args: Vec<MalForm>) -> MalResult<MalForm> {
    Ok(pr_seq(&args, " ", true).to_mal_form())
}

fn str(args: Vec<MalForm>) -> MalResult<MalForm> {
    Ok(pr_seq(&args, "", false).to_mal_form())
}
