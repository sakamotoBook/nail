use std::fmt;

use crate::env::Env;
use crate::error::NailError;

#[derive(Clone)]
pub enum Value {
    Number(i64),
    Bool(bool),
    Atom(String),
    List(Vec<Value>),
    Func(UserFunc),
    Builtin(fn(Vec<Value>) -> Result<Value, NailError>),
    Nil,
}

#[derive(Clone)]
pub struct UserFunc {
    pub(crate) clauses: Vec<Clause>,
    pub(crate) env: Env,
}

#[derive(Clone)]
pub(crate) struct Clause {
    pub(crate) pattern: Pattern,
    pub(crate) body: Expr,
}

#[derive(Clone)]
pub(crate) enum Pattern {
    Wildcard,
    Var(String),
    Number(i64),
    Bool(bool),
    Atom(String),
    List(Vec<Pattern>),
}

#[derive(Clone, Debug)]
pub(crate) enum Expr {
    Symbol(String),
    Number(i64),
    Bool(bool),
    Atom(String),
    Nil,
    List(Vec<Expr>),
}

pub(crate) fn value_structural_eq(a: &Value, b: &Value) -> bool {
    if is_nil_like(a) && is_nil_like(b) {
        return true;
    }

    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Atom(x), Value::Atom(y)) => x == y,
        (Value::Nil, Value::Nil) => true,
        (Value::List(xs), Value::List(ys)) => {
            xs.len() == ys.len()
                && xs
                    .iter()
                    .zip(ys.iter())
                    .all(|(x, y)| value_structural_eq(x, y))
        }
        _ => false,
    }
}

pub(crate) fn is_nil_like(value: &Value) -> bool {
    matches!(value, Value::Nil) || matches!(value, Value::List(items) if items.is_empty())
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Atom(a) => write!(f, ":{}", a),
            Value::List(items) => {
                write!(f, "(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, ")")
            }
            Value::Func(_) => write!(f, "<fn>"),
            Value::Builtin(_) => write!(f, "<builtin>"),
            Value::Nil => write!(f, "nil"),
        }
    }
}
