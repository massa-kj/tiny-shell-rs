pub mod token;

use std::fmt;
use self::token::{Token, TokenKind};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LexError {
    UnexpectedChar(char, usize),
    UnterminatedQuote(char),
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexError::UnexpectedChar(c, pos) => write!(f, "Unexpected character '{}' at position {}", c, pos),
            LexError::UnterminatedQuote(q) => write!(f, "Unterminated quote '{}'", q),
        }
    }
}

pub struct Lexer;

impl Lexer {
    pub fn tokenize(line: &str) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let mut pos = 0;
        let mut buf = String::new();
        let mut token_start = 0;

        while pos < chars.len() {
            let ch = chars[pos];
            match ch {
                ' ' | '\t' | '\n' => {
                    if !buf.is_empty() {
                        tokens.push(Token {
                            kind: TokenKind::Word,
                            lexeme: buf.clone(),
                            span: (token_start, pos),
                        });
                        buf.clear();
                    }
                    pos += 1;
                }
                '|' => {
                    if !buf.is_empty() {
                        tokens.push(Token {
                            kind: TokenKind::Word,
                            lexeme: buf.clone(),
                            span: (token_start, pos),
                        });
                        buf.clear();
                    }
                    let start = pos;
                    pos += 1;
                    if pos < chars.len() && chars[pos] == '|' {
                        pos += 1;
                        tokens.push(Token {
                            kind: TokenKind::Or,
                            lexeme: "||".to_string(),
                            span: (start, pos),
                        });
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::Pipe,
                            lexeme: "|".to_string(),
                            span: (start, pos),
                        });
                    }
                }
                '&' => {
                    if !buf.is_empty() {
                        tokens.push(Token {
                            kind: TokenKind::Word,
                            lexeme: buf.clone(),
                            span: (token_start, pos),
                        });
                        buf.clear();
                    }
                    let start = pos;
                    pos += 1;
                    if pos < chars.len() && chars[pos] == '&' {
                        pos += 1;
                        tokens.push(Token {
                            kind: TokenKind::And,
                            lexeme: "&&".to_string(),
                            span: (start, pos),
                        });
                    }
                }
                '>' => {
                    if !buf.is_empty() {
                        tokens.push(Token {
                            kind: TokenKind::Word,
                            lexeme: buf.clone(),
                            span: (token_start, pos),
                        });
                        buf.clear();
                    }
                    let start = pos;
                    pos += 1;
                    tokens.push(Token {
                        kind: TokenKind::RedirectOut,
                        lexeme: ">".to_string(),
                        span: (start, pos),
                    });
                }
                '<' => {
                    if !buf.is_empty() {
                        tokens.push(Token {
                            kind: TokenKind::Word,
                            lexeme: buf.clone(),
                            span: (token_start, pos),
                        });
                        buf.clear();
                    }
                    let start = pos;
                    pos += 1;
                    tokens.push(Token {
                        kind: TokenKind::RedirectIn,
                        lexeme: "<".to_string(),
                        span: (start, pos),
                    });
                }
                ';' => {
                    if !buf.is_empty() {
                        tokens.push(Token {
                            kind: TokenKind::Word,
                            lexeme: buf.clone(),
                            span: (token_start, pos),
                        });
                        buf.clear();
                    }
                    let start = pos;
                    pos += 1;
                    tokens.push(Token {
                        kind: TokenKind::Semicolon,
                        lexeme: ";".to_string(),
                        span: (start, pos),
                    });
                }
                '(' => {
                    if !buf.is_empty() {
                        tokens.push(Token {
                            kind: TokenKind::Word,
                            lexeme: buf.clone(),
                            span: (token_start, pos),
                        });
                        buf.clear();
                    }
                    let start = pos;
                    pos += 1;
                    tokens.push(Token {
                        kind: TokenKind::LParen,
                        lexeme: "(".to_string(),
                        span: (start, pos),
                    });
                }
                ')' => {
                    if !buf.is_empty() {
                        tokens.push(Token {
                            kind: TokenKind::Word,
                            lexeme: buf.clone(),
                            span: (token_start, pos),
                        });
                        buf.clear();
                    }
                    let start = pos;
                    pos += 1;
                    tokens.push(Token {
                        kind: TokenKind::RParen,
                        lexeme: ")".to_string(),
                        span: (start, pos),
                    });
                }
                '"' | '\'' => {
                    let quote = ch;
                    let start = pos;
                    pos += 1;
                    let mut quoted = String::new();
                    while pos < chars.len() {
                        let nc = chars[pos];
                        if nc == quote {
                            pos += 1;
                            break;
                        } else {
                            quoted.push(nc);
                            pos += 1;
                        }
                    }
                    // TODO: Added support for cases where quotes are not closed
                    // TODO: escape char
                    tokens.push(Token {
                        kind: if quote == '"' { TokenKind::DoubleQuote } else { TokenKind::SingleQuote },
                        lexeme: quoted,
                        span: (start, pos),
                    });
                }
                _ => {
                    if buf.is_empty() {
                        token_start = pos;
                    }
                    buf.push(ch);
                    pos += 1;
                }
            }
        }

        if !buf.is_empty() {
            tokens.push(Token {
                kind: TokenKind::Word,
                lexeme: buf,
                span: (token_start, pos),
            });
        }

        tokens.push(Token {
            kind: TokenKind::Eof,
            lexeme: "".to_string(),
            span: (pos, pos),
        });

        Ok(tokens)
    }
}

#[test]
fn test_tokenize_basic() {
    let tokens = Lexer::tokenize("echo hello | grep world").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Word);
    assert_eq!(tokens[0].lexeme, "echo");
    assert_eq!(tokens[2].kind, TokenKind::Pipe);
    assert_eq!(tokens[2].lexeme, "|");
}

