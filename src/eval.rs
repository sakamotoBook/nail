use std::collections::HashMap;

use crate::ast::{Clause, Expr, Pattern, UserFunc, Value, value_structural_eq};
use crate::env::Env;
use crate::parser::pattern_from_expr;

enum EvalOutcome {
    Value(Value),
    TailCall(Value, Vec<Value>),
}

pub(crate) fn eval(expr: &Expr, env: &Env) -> Result<Value, String> {
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
