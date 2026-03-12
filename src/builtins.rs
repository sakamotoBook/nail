use crate::ast::Value;
use crate::env::Env;

pub(crate) fn register_builtins(env: &Env) {
    env.set(
        "+",
        Value::Builtin(|args| numeric_fold(args, 0, |a, b| a + b)),
    );
    env.set(
        "-",
        Value::Builtin(|args| match args.as_slice() {
            [] => Ok(Value::Number(0)),
            [Value::Number(n)] => Ok(Value::Number(-n)),
            [..] => numeric_fold(args, 0, |a, b| a - b),
        }),
    );
    env.set(
        "*",
        Value::Builtin(|args| numeric_fold(args, 1, |a, b| a * b)),
    );
    env.set("list", Value::Builtin(|args| Ok(Value::List(args))));
    env.set(
        "head",
        Value::Builtin(|args| match args.as_slice() {
            [Value::List(v)] if !v.is_empty() => Ok(v[0].clone()),
            _ => Err("head expects non-empty list".to_string()),
        }),
    );
    env.set(
        "tail",
        Value::Builtin(|args| match args.as_slice() {
            [Value::List(v)] if !v.is_empty() => Ok(Value::List(v[1..].to_vec())),
            _ => Err("tail expects non-empty list".to_string()),
        }),
    );
    env.set(
        "print",
        Value::Builtin(|args| {
            for a in args {
                println!("{}", a);
            }
            Ok(Value::Nil)
        }),
    );
}

fn numeric_fold(args: Vec<Value>, seed: i64, op: fn(i64, i64) -> i64) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::Number(seed));
    }
    let mut iter = args.into_iter();
    let mut acc = match iter.next() {
        Some(Value::Number(n)) => n,
        _ => return Err("numeric operation requires numbers".to_string()),
    };
    for value in iter {
        let n = match value {
            Value::Number(n) => n,
            _ => return Err("numeric operation requires numbers".to_string()),
        };
        acc = op(acc, n);
    }
    Ok(Value::Number(acc))
}
