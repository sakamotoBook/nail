use std::env;
use std::fs;
use std::io::{self, Write};

use nail::{default_env, run_program};

const DEMO: &str = r#"
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

fn run_repl() {
    let env = default_env();
    println!("nail REPL (Ctrl-D で終了)");

    let stdin = io::stdin();
    loop {
        print!("> ");
        let _ = io::stdout().flush();

        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                let src = line.trim();
                if src.is_empty() {
                    continue;
                }
                match run_program(src, &env) {
                    Ok(v) => println!("=> {}", v),
                    Err(e) => eprintln!("error: {}", e),
                }
            }
            Err(e) => {
                eprintln!("error: failed to read input: {}", e);
                break;
            }
        }
    }
}

fn main() {
    let env = default_env();
    let args: Vec<String> = env::args().collect();

    match args.as_slice() {
        [_] => match run_program(DEMO, &env) {
            Ok(v) => println!("=> {}", v),
            Err(e) => eprintln!("error: {}", e),
        },
        [_, flag] if flag == "--repl" => run_repl(),
        [_, path] => match fs::read_to_string(path) {
            Ok(code) => match run_program(&code, &env) {
                Ok(v) => println!("=> {}", v),
                Err(e) => eprintln!("error: {}", e),
            },
            Err(e) => eprintln!("error: failed to read {}: {}", path, e),
        },
        _ => eprintln!("usage: cargo run -- [--repl | path/to/program.nail]"),
    }
}
