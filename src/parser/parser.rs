use std::fmt;
use crate::ast::{AstNode};

pub trait Parser {
    fn parse(&mut self) -> Result<AstNode, ParseError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    UnexpectedEof,
    UnexpectedToken {
        found: String,
        expected: Vec<String>,
        pos: usize,
    },
    UnmatchedParen {
        pos: usize,
    },
    UnclosedQuote {
        pos: usize,
        quote: char,
    },
    EmptyInput,
}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedEof => write!(f, "Unexpected end of input"),
            ParseError::UnexpectedToken { found, expected, pos } => {
                write!(f, "Unexpected token '{}' at position {}. Expected: {:?}", found, pos, expected)
            }
            ParseError::UnmatchedParen { pos } => write!(f, "Unmatched parenthesis at position {}", pos),
            ParseError::UnclosedQuote { pos, quote } => write!(f, "Unclosed quote '{}' at position {}", quote, pos),
            ParseError::EmptyInput => write!(f, "Input is empty"),
        }
    }
}

