use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Clone)]
pub enum Value {
    Number(i64),
    Bool(bool),
    Atom(String),
    List(Vec<Value>),
    Func(UserFunc),
    Builtin(fn(Vec<Value>) -> Result<Value, String>),
    Nil,
}

fn value_structural_eq(a: &Value, b: &Value) -> bool {
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

fn is_nil_like(value: &Value) -> bool {
    matches!(value, Value::Nil) || matches!(value, Value::List(items) if items.is_empty())
}

#[derive(Clone)]
pub struct UserFunc {
    clauses: Vec<Clause>,
    env: Env,
}

#[derive(Clone)]
struct Clause {
    pattern: Pattern,
    body: Expr,
}

#[derive(Clone)]
enum Pattern {
    Wildcard,
    Var(String),
    Number(i64),
    Bool(bool),
    Atom(String),
    List(Vec<Pattern>),
}

#[derive(Clone, Debug)]
enum Expr {
    Symbol(String),
    Number(i64),
    Bool(bool),
    Atom(String),
    Nil,
    List(Vec<Expr>),
}

#[derive(Clone)]
pub struct Env {
    parent: Option<Rc<Env>>,
    values: Rc<RefCell<HashMap<String, Value>>>,
}

impl Env {
    fn new() -> Self {
        Self {
            parent: None,
            values: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    fn child(&self) -> Self {
        Self {
            parent: Some(Rc::new(self.clone())),
            values: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    fn set(&self, key: &str, value: Value) {
        self.values.borrow_mut().insert(key.to_string(), value);
    }

    fn get(&self, key: &str) -> Option<Value> {
        if let Some(v) = self.values.borrow().get(key) {
            return Some(v.clone());
        }
        self.parent.as_ref().and_then(|p| p.get(key))
    }
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

fn tokenize(input: &str) -> Vec<String> {
    input
        .replace('(', " ( ")
        .replace(')', " ) ")
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

fn parse_all(input: &str) -> Result<Vec<Expr>, String> {
    let tokens = tokenize(input);
    let mut pos = 0;
    let mut exprs = Vec::new();
    while pos < tokens.len() {
        exprs.push(parse_expr(&tokens, &mut pos)?);
    }
    Ok(exprs)
}

fn parse_expr(tokens: &[String], pos: &mut usize) -> Result<Expr, String> {
    let token = tokens
        .get(*pos)
        .ok_or_else(|| "unexpected end of input".to_string())?;
    *pos += 1;
    match token.as_str() {
        "(" => {
            let mut list = Vec::new();
            while *pos < tokens.len() && tokens[*pos] != ")" {
                list.push(parse_expr(tokens, pos)?);
            }
            if *pos >= tokens.len() {
                return Err("missing ')'".to_string());
            }
            *pos += 1;
            Ok(Expr::List(list))
        }
        ")" => Err("unexpected ')'".to_string()),
        "true" => Ok(Expr::Bool(true)),
        "false" => Ok(Expr::Bool(false)),
        "nil" => Ok(Expr::Nil),
        _ if token.starts_with(':') => Ok(Expr::Atom(token.trim_start_matches(':').to_string())),
        _ => {
            if let Ok(n) = token.parse::<i64>() {
                Ok(Expr::Number(n))
            } else {
                Ok(Expr::Symbol(token.clone()))
            }
        }
    }
}

fn pattern_from_expr(expr: &Expr) -> Result<Pattern, String> {
    match expr {
        Expr::Symbol(s) if s == "_" => Ok(Pattern::Wildcard),
        Expr::Symbol(s) => Ok(Pattern::Var(s.clone())),
        Expr::Number(n) => Ok(Pattern::Number(*n)),
        Expr::Bool(b) => Ok(Pattern::Bool(*b)),
        Expr::Atom(a) => Ok(Pattern::Atom(a.clone())),
        Expr::Nil => Ok(Pattern::List(vec![])),
        Expr::List(items) => {
            let mut out = Vec::new();
            for i in items {
                out.push(pattern_from_expr(i)?);
            }
            Ok(Pattern::List(out))
        }
    }
}

fn match_pattern(pattern: &Pattern, value: &Value, bindings: &mut HashMap<String, Value>) -> bool {
    match pattern {
        Pattern::Wildcard => true,
        Pattern::Var(name) => {
            if let Some(existing) = bindings.get(name) {
                value_structural_eq(existing, value)
            } else {
                bindings.insert(name.clone(), value.clone());
                true
            }
        }
        Pattern::Number(n) => matches!(value, Value::Number(v) if v == n),
        Pattern::Bool(b) => matches!(value, Value::Bool(v) if v == b),
        Pattern::Atom(a) => matches!(value, Value::Atom(v) if v == a),
        Pattern::List(items) => match value {
            Value::List(vals) => {
                if items.len() != vals.len() {
                    return false;
                }
                for (p, v) in items.iter().zip(vals.iter()) {
                    if !match_pattern(p, v, bindings) {
                        return false;
                    }
                }
                true
            }
            Value::Nil if items.is_empty() => true,
            _ => false,
        },
    }
}

enum EvalOutcome {
    Value(Value),
    TailCall(Value, Vec<Value>),
}

fn eval(expr: &Expr, env: &Env) -> Result<Value, String> {
    match eval_with_tail(expr, env, false)? {
        EvalOutcome::Value(v) => Ok(v),
        EvalOutcome::TailCall(callee, args) => apply(callee, args),
    }
}

fn eval_with_tail(expr: &Expr, env: &Env, tail_position: bool) -> Result<EvalOutcome, String> {
    match expr {
        Expr::Number(n) => Ok(EvalOutcome::Value(Value::Number(*n))),
        Expr::Bool(b) => Ok(EvalOutcome::Value(Value::Bool(*b))),
        Expr::Atom(a) => Ok(EvalOutcome::Value(Value::Atom(a.clone()))),
        Expr::Nil => Ok(EvalOutcome::Value(Value::Nil)),
        Expr::Symbol(s) => env
            .get(s)
            .map(EvalOutcome::Value)
            .ok_or_else(|| format!("undefined symbol: {}", s)),
        Expr::List(items) => {
            if items.is_empty() {
                return Ok(EvalOutcome::Value(Value::Nil));
            }

            if let Expr::Symbol(op) = &items[0] {
                match op.as_str() {
                    "def" => {
                        if items.len() != 3 {
                            return Err("def expects (def name expr)".to_string());
                        }
                        let name = match &items[1] {
                            Expr::Symbol(s) => s.clone(),
                            _ => return Err("def name must be symbol".to_string()),
                        };
                        let value = eval(&items[2], env)?;
                        env.set(&name, value.clone());
                        return Ok(EvalOutcome::Value(value));
                    }
                    "if" => {
                        if items.len() != 4 {
                            return Err("if expects (if cond then else)".to_string());
                        }
                        let cond = eval(&items[1], env)?;
                        let branch = if matches!(cond, Value::Bool(true)) {
                            &items[2]
                        } else {
                            &items[3]
                        };
                        return eval_with_tail(branch, env, tail_position);
                    }
                    "let" => {
                        if items.len() != 4 {
                            return Err("let expects (let name value body)".to_string());
                        }
                        let name = match &items[1] {
                            Expr::Symbol(s) => s.clone(),
                            _ => return Err("let name must be symbol".to_string()),
                        };
                        let value = eval(&items[2], env)?;
                        let local = env.child();
                        local.set(&name, value);
                        return eval_with_tail(&items[3], &local, tail_position);
                    }
                    "fn" => {
                        if items.len() < 2 {
                            return Err("fn expects at least one clause".to_string());
                        }
                        let mut clauses = Vec::new();
                        for clause_expr in &items[1..] {
                            let clause_items = match clause_expr {
                                Expr::List(v) => v,
                                _ => return Err("fn clause must be list".to_string()),
                            };
                            if clause_items.len() != 2 {
                                return Err("fn clause expects (pattern body)".to_string());
                            }
                            let pattern = pattern_from_expr(&clause_items[0])?;
                            let body = clause_items[1].clone();
                            clauses.push(Clause { pattern, body });
                        }
                        return Ok(EvalOutcome::Value(Value::Func(UserFunc {
                            clauses,
                            env: env.clone(),
                        })));
                    }
                    "|>" => {
                        if items.len() < 3 {
                            return Err("|> expects at least (|> value step)".to_string());
                        }
                        let mut acc = eval(&items[1], env)?;
                        for (idx, step) in items[2..].iter().enumerate() {
                            let is_last = idx == items.len() - 3;
                            let (callee, mut args) = match step {
                                Expr::List(parts) if !parts.is_empty() => {
                                    let callee = eval(&parts[0], env)?;
                                    let mut args = Vec::new();
                                    for arg in &parts[1..] {
                                        args.push(eval(arg, env)?);
                                    }
                                    (callee, args)
                                }
                                _ => {
                                    let callee = eval(step, env)?;
                                    (callee, Vec::new())
                                }
                            };
                            args.insert(0, acc);

                            if tail_position && is_last {
                                return Ok(EvalOutcome::TailCall(callee, args));
                            }
                            acc = apply(callee, args)?;
                        }
                        return Ok(EvalOutcome::Value(acc));
                    }
                    _ => {}
                }
            }

            let callee = eval(&items[0], env)?;
            let mut args = Vec::new();
            for arg in &items[1..] {
                args.push(eval(arg, env)?);
            }
            if tail_position {
                Ok(EvalOutcome::TailCall(callee, args))
            } else {
                Ok(EvalOutcome::Value(apply(callee, args)?))
            }
        }
    }
}

fn apply(callee: Value, args: Vec<Value>) -> Result<Value, String> {
    let mut callee = callee;
    let mut args = args;

    loop {
        match callee.clone() {
            Value::Builtin(f) => return f(args),
            Value::Func(func) => {
                if args.len() != 1 {
                    return Err("user function expects exactly one argument".to_string());
                }
                let arg = &args[0];
                let mut matched = false;
                for clause in &func.clauses {
                    let mut bindings = HashMap::new();
                    if match_pattern(&clause.pattern, arg, &mut bindings) {
                        let local = func.env.child();
                        for (k, v) in bindings {
                            local.set(&k, v);
                        }
                        matched = true;
                        match eval_with_tail(&clause.body, &local, true)? {
                            EvalOutcome::Value(v) => return Ok(v),
                            EvalOutcome::TailCall(next_callee, next_args) => {
                                callee = next_callee;
                                args = next_args;
                            }
                        }
                        break;
                    }
                }
                if !matched {
                    return Err("no function clause matched".to_string());
                }
            }
            _ => return Err("attempted to call non-function".to_string()),
        }
    }
}

fn register_builtins(env: &Env) {
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

pub fn default_env() -> Env {
    let env = Env::new();
    register_builtins(&env);
    env
}

pub fn run_program(code: &str, env: &Env) -> Result<Value, String> {
    let mut last = Value::Nil;
    for expr in parse_all(code)? {
        last = eval(&expr, env)?;
    }
    Ok(last)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Env {
        default_env()
    }

    #[test]
    fn pattern_matching_works() {
        let env = setup();
        let code = r#"
(def f (fn ((:ok x) (+ x 10)) (_ 0)))
(f (list :ok 2))
"#;
        let value = run_program(code, &env).unwrap();
        assert!(matches!(value, Value::Number(12)));
    }

    #[test]
    fn pipeline_works() {
        let env = setup();
        let code = "(|> 5 (+ 3) (* 2))";
        let value = run_program(code, &env).unwrap();
        assert!(matches!(value, Value::Number(16)));
    }

    #[test]
    fn deep_tail_recursion_works() {
        let env = setup();
        let code = r#"
(def count-down
  (fn
    (0 0)
    (n (count-down (- n 1)))))

(count-down 50000)
"#;
        let value = run_program(code, &env).unwrap();
        assert!(matches!(value, Value::Number(0)));
    }

    #[test]
    fn pipeline_can_pass_list_values() {
        let env = setup();
        let code = "(|> (list 1 2 3) (tail) (head))";
        let value = run_program(code, &env).unwrap();
        assert!(matches!(value, Value::Number(2)));
    }

    #[test]
    fn duplicate_pattern_variables_must_match_same_value() {
        let env = setup();
        let code = r#"
(def same-pair
  (fn
    ((x x) :ok)
    (_ :ng)))

(same-pair (list 1 2))
"#;
        let value = run_program(code, &env).unwrap();
        assert!(matches!(value, Value::Atom(ref a) if a == "ng"));
    }

    #[test]
    fn unary_minus_works() {
        let env = setup();
        let code = "(- 5)";
        let value = run_program(code, &env).unwrap();
        assert!(matches!(value, Value::Number(-5)));
    }

    #[test]
    fn nil_literal_works() {
        let env = setup();
        let code = "nil";
        let value = run_program(code, &env).unwrap();
        assert!(matches!(value, Value::Nil));
    }

    #[test]
    fn nil_pattern_matches_nil_literal_and_empty_list() {
        let env = setup();
        let code = r#"
(def is-empty
  (fn
    (nil :ok)
    (_ :ng)))

(list (is-empty nil) (is-empty (list)))
"#;

        let value = run_program(code, &env).unwrap();
        assert!(matches!(
            value,
            Value::List(ref values)
                if values.len() == 2
                    && matches!(values[0], Value::Atom(ref atom) if atom == "ok")
                    && matches!(values[1], Value::Atom(ref atom) if atom == "ok")
        ));
    }
}
