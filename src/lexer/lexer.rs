use std::fmt;
use super::token::{Token, TokenKind};

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

pub struct Lexer<'a> {
    input: &'a str,
    chars: std::str::Chars<'a>,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input,
            chars: input.chars(),
            pos: 0,
        }
    }

    pub fn tokenize(input: &str) -> Result<Vec<Token>, LexError> {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();
        while let Some(token) = lexer.next_token()? {
            tokens.push(token);
            if tokens.last().map_or(false, |t| t.kind == TokenKind::Eof) {
                break;
            }
        }
        Ok(tokens)
    }

    pub fn next_token(&mut self) -> Result<Option<Token>, LexError> {
        Ok(None)
    }

    pub fn tokenize_all(line: &str) -> Result<Vec<Token>, LexError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{Token, TokenKind};

    fn token(kind: TokenKind, lexeme: &str, span: (usize, usize)) -> Token {
        Token {
            kind,
            lexeme: lexeme.to_string(),
            span,
        }
    }

    #[test]
    fn test_tokenize_simple_words() {
        let input = "echo hello";
        let tokens = Lexer::tokenize_all(input).unwrap();
        assert_eq!(
            tokens,
            vec![
                token(TokenKind::Word, "echo", (0, 4)),
                token(TokenKind::Word, "hello", (5, 10)),
                token(TokenKind::Eof, "", (10, 10)),
            ]
        );
    }

    #[test]
    fn test_tokenize_operators() {
        let input = "a|b && c || d > e < f ; (g) ";
        let tokens = Lexer::tokenize_all(input).unwrap();
        assert_eq!(
            tokens,
            vec![
                token(TokenKind::Word, "a", (0, 1)),
                token(TokenKind::Pipe, "|", (1, 2)),
                token(TokenKind::Word, "b", (2, 3)),
                token(TokenKind::And, "&&", (4, 6)),
                token(TokenKind::Word, "c", (7, 8)),
                token(TokenKind::Or, "||", (9, 11)),
                token(TokenKind::Word, "d", (12, 13)),
                token(TokenKind::RedirectOut, ">", (14, 15)),
                token(TokenKind::Word, "e", (16, 17)),
                token(TokenKind::RedirectIn, "<", (18, 19)),
                token(TokenKind::Word, "f", (20, 21)),
                token(TokenKind::Semicolon, ";", (22, 23)),
                token(TokenKind::LParen, "(", (24, 25)),
                token(TokenKind::Word, "g", (25, 26)),
                token(TokenKind::RParen, ")", (26, 27)),
                token(TokenKind::Eof, "", (28, 28)),
            ]
        );
    }

    #[test]
    fn test_tokenize_quotes() {
        let input = r#"echo "hello world" 'foo bar'"#;
        let tokens = Lexer::tokenize_all(input).unwrap();
        assert_eq!(
            tokens,
            vec![
                token(TokenKind::Word, "echo", (0, 4)),
                token(TokenKind::DoubleQuote, "hello world", (5, 18)),
                token(TokenKind::SingleQuote, "foo bar", (19, 29)),
                token(TokenKind::Eof, "", (29, 29)),
            ]
        );
    }

    #[test]
    fn test_tokenize_mixed() {
        let input = r#"ls -l | grep "foo bar" && echo done"#;
        let tokens = Lexer::tokenize_all(input).unwrap();
        assert_eq!(
            tokens,
            vec![
                token(TokenKind::Word, "ls", (0, 2)),
                token(TokenKind::Word, "-l", (3, 5)),
                token(TokenKind::Pipe, "|", (6, 7)),
                token(TokenKind::Word, "grep", (8, 12)),
                token(TokenKind::DoubleQuote, "foo bar", (13, 23)),
                token(TokenKind::And, "&&", (24, 26)),
                token(TokenKind::Word, "echo", (27, 31)),
                token(TokenKind::Word, "done", (32, 36)),
                token(TokenKind::Eof, "", (36, 36)),
            ]
        );
    }

    #[test]
    fn test_tokenize_empty() {
        let input = "";
        let tokens = Lexer::tokenize_all(input).unwrap();
        assert_eq!(
            tokens,
            vec![
                token(TokenKind::Eof, "", (0, 0)),
            ]
        );
    }
}

