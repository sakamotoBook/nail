mod ast;
mod builtins;
mod env;
mod error;
mod eval;
mod parser;

pub use ast::Value;
pub use env::Env;
pub use error::NailError;

use builtins::register_builtins;
use eval::eval;
use parser::parse_all;

pub fn default_env() -> Env {
    let env = Env::new();
    register_builtins(&env);
    env
}

pub fn run_program(code: &str, env: &Env) -> Result<Value, NailError> {
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
    fn empty_list_literal_and_list_builtin_are_nil() {
        let env = setup();
        let code = "(list () (list))";
        let value = run_program(code, &env).unwrap();
        assert!(matches!(
            value,
            Value::List(ref values)
                if values.len() == 2
                    && matches!(values[0], Value::Nil)
                    && matches!(values[1], Value::Nil)
        ));
    }

    #[test]
    fn tail_of_singleton_list_is_nil() {
        let env = setup();
        let code = "(tail (list 1))";
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

    #[test]
    fn user_function_can_take_multiple_arguments() {
        let env = setup();
        let code = r#"
(def sum3
  (fn
    ((a b c) (+ a b c))))

(sum3 1 2 3)
"#;

        let value = run_program(code, &env).unwrap();
        assert!(matches!(value, Value::Number(6)));
    }

    #[test]
    fn single_argument_list_pattern_still_works() {
        let env = setup();
        let code = r#"
(def sum-pair
  (fn
    ((a b) (+ a b))))

(sum-pair (list 2 5))
"#;

        let value = run_program(code, &env).unwrap();
        assert!(matches!(value, Value::Number(7)));
    }

    #[test]
    fn parser_error_contains_position() {
        let env = setup();
        let error = match run_program("(+ 1", &env) {
            Ok(value) => panic!("expected parser error, got value: {}", value),
            Err(error) => error,
        };
        assert!(error.to_string().contains("line 1, col 1"));
    }

    #[test]
    fn closure_observes_redefinition_in_captured_frame() {
        let env = setup();
        let code = r#"
(def x 10)
(def get-x (fn (_ x)))
(def x 20)
(get-x nil)
"#;

        let value = run_program(code, &env).unwrap();
        assert!(matches!(value, Value::Number(20)));
    }

    #[test]
    fn let_is_not_letrec_for_local_function_binding() {
        let env = setup();
        let code = r#"
(let f
  (fn
    (0 0)
    (n (f (- n 1))))
  (f 3))
"#;

        let error = match run_program(code, &env) {
            Ok(value) => panic!("expected error, got value: {}", value),
            Err(error) => error,
        };
        assert!(error.to_string().contains("undefined symbol: f"));
    }

    #[test]
    fn builtin_errors_are_typed() {
        let env = setup();
        let err = match run_program("(head nil)", &env) {
            Ok(value) => panic!("expected error, got value: {}", value),
            Err(error) => error,
        };
        assert!(matches!(err, NailError::Builtin(_)));
    }
}
