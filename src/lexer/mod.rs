use crate::ast::{TokenKind};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LexError {
    UnexpectedChar(char, usize),
    UnterminatedQuote(char),
}

pub struct Lexer;

impl Lexer {
    pub fn tokenize(line: &str) -> Vec<TokenKind> {
        let mut tokens = Vec::new();
        let mut chars = line.chars().peekable();
        let mut buf = String::new();
        // let chars: Vec<char> = input.chars().collect();
        // let mut pos = 0;

        // while pos < chars.len() {
        //     let start = pos;
        //     let ch = chars[pos];
        while let Some(&ch) = chars.peek() {
            match ch {
                ' ' | '\t' | '\n' => {
                    if !buf.is_empty() {
                        tokens.push(TokenKind::Word(buf.clone()));
                        buf.clear();
                    }
                    chars.next();
                }
                '|' => {
                    if !buf.is_empty() {
                        tokens.push(TokenKind::Word(buf.clone()));
                        buf.clear();
                    }
                    chars.next();
                    if chars.peek() == Some(&'|') {
                        chars.next();
                        tokens.push(TokenKind::Or);
                    } else {
                        tokens.push(TokenKind::Pipe);
                    }
                }
                '&' => {
                    if !buf.is_empty() {
                        tokens.push(TokenKind::Word(buf.clone()));
                        buf.clear();
                    }
                    chars.next();
                    if chars.peek() == Some(&'&') {
                        chars.next();
                        tokens.push(TokenKind::And);
                    }
                }
                '>' => {
                    if !buf.is_empty() {
                        tokens.push(TokenKind::Word(buf.clone()));
                        buf.clear();
                    }
                    chars.next();
                    tokens.push(TokenKind::RedirectOut);
                }
                '<' => {
                    if !buf.is_empty() {
                        tokens.push(TokenKind::Word(buf.clone()));
                        buf.clear();
                    }
                    chars.next();
                    tokens.push(TokenKind::RedirectIn);
                }
                ';' => {
                    if !buf.is_empty() {
                        tokens.push(TokenKind::Word(buf.clone()));
                        buf.clear();
                    }
                    chars.next();
                    tokens.push(TokenKind::Semicolon);
                }
                '(' => {
                    if !buf.is_empty() {
                        tokens.push(TokenKind::Word(buf.clone()));
                        buf.clear();
                    }
                    chars.next();
                    tokens.push(TokenKind::LParen);
                }
                ')' => {
                    if !buf.is_empty() {
                        tokens.push(TokenKind::Word(buf.clone()));
                        buf.clear();
                    }
                    chars.next();
                    tokens.push(TokenKind::RParen);
                }
                '"' | '\'' => {
                    let quote = ch;
                    chars.next();
                    while let Some(&nc) = chars.peek() {
                        if nc == quote {
                            chars.next();
                            break;
                        } else {
                            buf.push(nc);
                            chars.next();
                        }
                    }
                    // TODO: Added support for cases where quotes are not closed
                    // TODO: escape char
                }
                _ => {
                    buf.push(ch);
                    chars.next();
                }
            }
        }

        if !buf.is_empty() {
            tokens.push(TokenKind::Word(buf));
        }
        tokens.push(TokenKind::Eof);

        tokens
    }
}

// #[test]
// fn test_tokenize_basic() {
//     let tokens = tokenize("echo hello | grep world").unwrap();
//     assert_eq!(tokens[0].kind, TokenKind::Word);
//     assert_eq!(tokens[0].lexeme, "echo");
//     assert_eq!(tokens[2].kind, TokenKind::Pipe);
//     assert_eq!(tokens[2].lexeme, "|");
// }

