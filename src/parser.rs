use crate::ast::{Expr, Pattern};
use crate::error::NailError;

#[derive(Clone)]
struct Token {
    lexeme: String,
    line: usize,
    col: usize,
}

impl Token {
    fn at(&self) -> String {
        format!("line {}, col {}", self.line, self.col)
    }
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut line = 1;
    let mut col = 1;
    let mut current = String::new();
    let mut start_col = 1;

    for ch in input.chars() {
        match ch {
            '(' | ')' => {
                if !current.is_empty() {
                    tokens.push(Token {
                        lexeme: current.clone(),
                        line,
                        col: start_col,
                    });
                    current.clear();
                }
                tokens.push(Token {
                    lexeme: ch.to_string(),
                    line,
                    col,
                });
                col += 1;
            }
            '\n' => {
                if !current.is_empty() {
                    tokens.push(Token {
                        lexeme: current.clone(),
                        line,
                        col: start_col,
                    });
                    current.clear();
                }
                line += 1;
                col = 1;
            }
            c if c.is_whitespace() => {
                if !current.is_empty() {
                    tokens.push(Token {
                        lexeme: current.clone(),
                        line,
                        col: start_col,
                    });
                    current.clear();
                }
                col += 1;
            }
            c => {
                if current.is_empty() {
                    start_col = col;
                }
                current.push(c);
                col += 1;
            }
        }
    }

    if !current.is_empty() {
        tokens.push(Token {
            lexeme: current,
            line,
            col: start_col,
        });
    }

    tokens
}

pub(crate) fn parse_all(input: &str) -> Result<Vec<Expr>, NailError> {
    let tokens = tokenize(input);
    let mut pos = 0;
    let mut exprs = Vec::new();
    while pos < tokens.len() {
        exprs.push(parse_expr(&tokens, &mut pos)?);
    }
    Ok(exprs)
}

fn parse_expr(tokens: &[Token], pos: &mut usize) -> Result<Expr, NailError> {
    let token = tokens
        .get(*pos)
        .ok_or_else(|| NailError::parse("unexpected end of input"))?;
    *pos += 1;
    match token.lexeme.as_str() {
        "(" => {
            let mut list = Vec::new();
            while *pos < tokens.len() && tokens[*pos].lexeme != ")" {
                list.push(parse_expr(tokens, pos)?);
            }
            if *pos >= tokens.len() {
                return Err(NailError::parse(format!(
                    "missing ')' started at {}",
                    token.at()
                )));
            }
            *pos += 1;
            Ok(Expr::List(list))
        }
        ")" => Err(NailError::parse(format!(
            "unexpected ')' at {}",
            token.at()
        ))),
        "true" => Ok(Expr::Bool(true)),
        "false" => Ok(Expr::Bool(false)),
        "nil" => Ok(Expr::Nil),
        _ if token.lexeme.starts_with(':') => {
            Ok(Expr::Atom(token.lexeme.trim_start_matches(':').to_string()))
        }
        _ => {
            if let Ok(n) = token.lexeme.parse::<i64>() {
                Ok(Expr::Number(n))
            } else {
                Ok(Expr::Symbol(token.lexeme.clone()))
            }
        }
    }
}

pub(crate) fn pattern_from_expr(expr: &Expr) -> Result<Pattern, NailError> {
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
