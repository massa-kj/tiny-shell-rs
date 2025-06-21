use crate::ast::{Token, TokenKind};

pub fn tokenize(line: &str) -> Vec<Token> {
    line.split_whitespace()
        .map(|s| Token {
            kind: TokenKind::Word,
            text: s.to_string(),
        })
        .collect()
}

