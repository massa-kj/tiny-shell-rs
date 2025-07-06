use std::fmt;
use crate::lexer::LexError;
use crate::parser::ParseError;
use crate::executor::ExecError;

#[derive(Debug)]
pub enum ShellError {
    Io(std::io::Error),
    Lex(LexError),
    Parse(ParseError),
    Exec(ExecError),
}

impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShellError::Io(e) => write!(f, "IO error: {}", e),
            ShellError::Lex(e) => write!(f, "Lexing error: {}", e),
            ShellError::Parse(e) => write!(f, "Parsing error: {}", e),
            ShellError::Exec(e) => write!(f, "Execution error: {}", e),
        }
    }
}

impl std::error::Error for ShellError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ShellError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ShellError {
    fn from(e: std::io::Error) -> Self {
        ShellError::Io(e)
    }
}

