use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NailError {
    Parse(String),
    Eval(String),
    Builtin(String),
}

impl NailError {
    pub(crate) fn parse(message: impl Into<String>) -> Self {
        Self::Parse(message.into())
    }

    pub(crate) fn eval(message: impl Into<String>) -> Self {
        Self::Eval(message.into())
    }

    pub(crate) fn builtin(message: impl Into<String>) -> Self {
        Self::Builtin(message.into())
    }
}

impl fmt::Display for NailError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NailError::Parse(message) => write!(f, "parse error: {}", message),
            NailError::Eval(message) => write!(f, "eval error: {}", message),
            NailError::Builtin(message) => write!(f, "builtin error: {}", message),
        }
    }
}

impl std::error::Error for NailError {}
