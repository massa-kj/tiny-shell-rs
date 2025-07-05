use std::fmt;
use std::io;
use crate::lexer::LexError;
use crate::parser::ParseError;

#[derive(Debug)]
pub enum ExecError {
    CommandNotFound(String),
    Io(io::Error),
    PermissionDenied(String),
    InvalidArgument(String),
    PipelineError(String),
    RedirectError(String),
    SubshellError(String),
    NoSuchBuiltin(String),
    Lex(LexError),
    Parse(ParseError),
    Custom(String),
}

impl fmt::Display for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecError::CommandNotFound(cmd) => write!(f, "Command not found: {}", cmd),
            ExecError::Io(e) => write!(f, "IO error: {}", e),
            ExecError::PermissionDenied(cmd) => write!(f, "Permission denied: {}", cmd),
            ExecError::InvalidArgument(arg) => write!(f, "Invalid argument: {}", arg),
            ExecError::PipelineError(msg) => write!(f, "Pipeline error: {}", msg),
            ExecError::RedirectError(msg) => write!(f, "Redirect error: {}", msg),
            ExecError::SubshellError(msg) => write!(f, "Subshell error: {}", msg),
            ExecError::NoSuchBuiltin(name) => write!(f, "No such builtin command: {}", name),
            ExecError::Lex(e) => write!(f, "Lexing error: {}", e),
            ExecError::Parse(e) => write!(f, "Parsing error: {}", e),
            ExecError::Custom(msg) => write!(f, "Execution error: {}", msg),
        }
    }
}

impl std::error::Error for ExecError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ExecError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ExecError {
    fn from(e: std::io::Error) -> Self {
        ExecError::Io(e)
    }
}

