use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Clone)]
enum Value {
    Number(i64),
    Bool(bool),
    Atom(String),
    List(Vec<Value>),
    Func(UserFunc),
    Builtin(fn(Vec<Value>) -> Result<Value, String>),
    Nil,
}

#[derive(Clone)]
struct UserFunc {
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
    List(Vec<Expr>),
}

#[derive(Clone)]
struct Env {
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
    match (pattern, value) {
        (Pattern::Wildcard, _) => true,
        (Pattern::Var(name), v) => {
            bindings.insert(name.clone(), v.clone());
            true
        }
        (Pattern::Number(a), Value::Number(b)) => a == b,
        (Pattern::Bool(a), Value::Bool(b)) => a == b,
        (Pattern::Atom(a), Value::Atom(b)) => a == b,
        (Pattern::List(pa), Value::List(vs)) => {
            if pa.len() != vs.len() {
                return false;
            }
            for (p, v) in pa.iter().zip(vs.iter()) {
                if !match_pattern(p, v, bindings) {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}

fn eval(expr: &Expr, env: &Env) -> Result<Value, String> {
    match eval_with_tail(expr, env, true)? {
        EvalOutcome::Value(v) => Ok(v),
        EvalOutcome::TailCall(callee, args) => apply(callee, args),
    }
}

enum EvalOutcome {
    Value(Value),
    TailCall(Value, Vec<Value>),
}

fn eval_with_tail(expr: &Expr, env: &Env, tail_position: bool) -> Result<EvalOutcome, String> {
    match expr {
        Expr::Number(n) => Ok(EvalOutcome::Value(Value::Number(*n))),
        Expr::Bool(b) => Ok(EvalOutcome::Value(Value::Bool(*b))),
        Expr::Atom(a) => Ok(EvalOutcome::Value(Value::Atom(a.clone()))),
        Expr::Symbol(s) => Ok(EvalOutcome::Value(
            env.get(s)
                .ok_or_else(|| format!("undefined symbol: {}", s))?,
        )),
        Expr::List(items) => {
            if items.is_empty() {
                return Ok(EvalOutcome::Value(Value::Nil));
            }
            match &items[0] {
                Expr::Symbol(op) if op == "def" => {
                    if items.len() != 3 {
                        return Err("(def name expr)".to_string());
                    }
                    let name = match &items[1] {
                        Expr::Symbol(s) => s,
                        _ => return Err("def name must be symbol".to_string()),
                    };
                    let value = eval(&items[2], env)?;
                    env.set(name, value.clone());
                    Ok(EvalOutcome::Value(value))
                }
                Expr::Symbol(op) if op == "let" => {
                    if items.len() != 4 {
                        return Err("(let name expr body)".to_string());
                    }
                    let name = match &items[1] {
                        Expr::Symbol(s) => s,
                        _ => return Err("let name must be symbol".to_string()),
                    };
                    let value = eval(&items[2], env)?;
                    let child = env.child();
                    child.set(name, value);
                    eval_with_tail(&items[3], &child, tail_position)
                }
                Expr::Symbol(op) if op == "if" => {
                    if items.len() != 4 {
                        return Err("(if cond then else)".to_string());
                    }
                    let cond = eval(&items[1], env)?;
                    if matches!(cond, Value::Bool(true)) {
                        eval_with_tail(&items[2], env, tail_position)
                    } else {
                        eval_with_tail(&items[3], env, tail_position)
                    }
                }
                Expr::Symbol(op) if op == "fn" => {
                    let mut clauses = Vec::new();
                    for clause in &items[1..] {
                        match clause {
                            Expr::List(parts) if parts.len() == 2 => {
                                clauses.push(Clause {
                                    pattern: pattern_from_expr(&parts[0])?,
                                    body: parts[1].clone(),
                                });
                            }
                            _ => return Err("fn clause must be (pattern body)".to_string()),
                        }
                    }
                    Ok(EvalOutcome::Value(Value::Func(UserFunc {
                        clauses,
                        env: env.clone(),
                    })))
                }
                Expr::Symbol(op) if op == "|>" => {
                    if items.len() < 3 {
                        return Err("(|> value (f ...) ... )".to_string());
                    }
                    let mut current = eval(&items[1], env)?;
                    for stage in &items[2..] {
                        match stage {
                            Expr::List(call) if !call.is_empty() => {
                                let mut full = vec![call[0].clone()];
                                full.push(expr_from_value(current.clone())?);
                                full.extend_from_slice(&call[1..]);
                                current = eval(&Expr::List(full), env)?;
                            }
                            _ => return Err("pipeline stage must be non-empty list".to_string()),
                        }
                    }
                    Ok(EvalOutcome::Value(current))
                }
                _ => {
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
    }
}

fn expr_from_value(value: Value) -> Result<Expr, String> {
    match value {
        Value::Number(n) => Ok(Expr::Number(n)),
        Value::Bool(b) => Ok(Expr::Bool(b)),
        Value::Atom(a) => Ok(Expr::Atom(a)),
        Value::List(items) => {
            let mut exprs = Vec::new();
            for i in items {
                exprs.push(expr_from_value(i)?);
            }
            Ok(Expr::List(exprs))
        }
        _ => Err("pipeline can only forward literal values".to_string()),
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
        Value::Builtin(|args| numeric_fold(args, 0, |a, b| a - b)),
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

fn run_program(code: &str, env: &Env) -> Result<Value, String> {
    let mut last = Value::Nil;
    for expr in parse_all(code)? {
        last = eval(&expr, env)?;
    }
    Ok(last)
}

fn main() {
    let env = Env::new();
    register_builtins(&env);

    let demo = r#"
(def classify
  (fn
    ((:ok x) (+ x 1))
    ((:error _) 0)
    (_ -1)))

(def total
  (|> 1
      (+ 40)
      (* 2)))

(print (classify (list :ok 5)))
(print (classify (list :error 9)))
(print total)
"#;

    match run_program(demo, &env) {
        Ok(v) => println!("=> {}", v),
        Err(e) => eprintln!("error: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Env {
        let env = Env::new();
        register_builtins(&env);
        env
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
}
