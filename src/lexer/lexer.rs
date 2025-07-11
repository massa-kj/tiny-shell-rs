use std::fmt;
use super::token::{Token, TokenKind};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LexError {
    UnexpectedChar(char, usize),
    UnterminatedQuote(char, usize),
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexError::UnexpectedChar(c, pos) => write!(f, "Unexpected character '{}' at position {}", c, pos),
            LexError::UnterminatedQuote(c, q) => write!(f, "Unterminated quote '{}' starting at position {}", c, q),
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

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token()? {
            tokens.push(token);
            if tokens.last().map_or(false, |t| t.kind == TokenKind::Eof) {
                break;
            }
        }
        Ok(tokens)
    }

    pub fn next_token(&mut self) -> Result<Option<Token>, LexError> {
        let chars: Vec<char> = self.input.chars().collect();
        let mut buf = String::new();
        let mut token_start = self.pos;

        while self.pos < chars.len() {
            let ch = chars[self.pos];

            match ch {
                ' ' | '\t' | '\n' => {
                    if !buf.is_empty() {
                        let token = Token {
                            kind: TokenKind::Word,
                            lexeme: buf.clone(),
                            span: (token_start, self.pos),
                        };
                        buf.clear();
                        self.pos += 1;
                        return Ok(Some(token));
                    }
                    self.pos += 1;
                }
                '|' => {
                    if !buf.is_empty() {
                        let token = Token {
                            kind: TokenKind::Word,
                            lexeme: buf.clone(),
                            span: (token_start, self.pos),
                        };
                        buf.clear();
                        // Do not consume '|' (process it in the next loop)
                        return Ok(Some(token));
                    }
                    if self.pos + 1 < chars.len() && chars[self.pos + 1] == '|' {
                        let token = Token {
                            kind: TokenKind::Or,
                            lexeme: "||".to_string(),
                            span: (self.pos, self.pos + 2),
                        };
                        self.pos += 2;
                        return Ok(Some(token));
                    } else {
                        let token = Token {
                            kind: TokenKind::Pipe,
                            lexeme: "|".to_string(),
                            span: (self.pos, self.pos + 1),
                        };
                        self.pos += 1;
                        return Ok(Some(token));
                    }
                }
                '&' => {
                    if !buf.is_empty() {
                        let token = Token {
                            kind: TokenKind::Word,
                            lexeme: buf.clone(),
                            span: (token_start, self.pos),
                        };
                        buf.clear();
                        // Do not consume '&' (process it in the next loop)
                        return Ok(Some(token));
                    }
                    if self.pos + 1 < chars.len() && chars[self.pos + 1] == '&' {
                        let token = Token {
                            kind: TokenKind::And,
                            lexeme: "&&".to_string(),
                            span: (self.pos, self.pos + 2),
                        };
                        self.pos += 2;
                        return Ok(Some(token));
                    } else {
                        let token = Token {
                            kind: TokenKind::NotImplemented,
                            lexeme: "&".to_string(),
                            span: (self.pos, self.pos + 1),
                        };
                        self.pos += 1;
                        return Ok(Some(token));
                    }
                }
                '>' => {
                    if !buf.is_empty() {
                        let token = Token {
                            kind: TokenKind::Word,
                            lexeme: buf.clone(),
                            span: (token_start, self.pos),
                        };
                        buf.clear();
                        return Ok(Some(token));
                    }
                    let token = Token {
                        kind: TokenKind::RedirectOut,
                        lexeme: ">".to_string(),
                        span: (self.pos, self.pos + 1),
                    };
                    self.pos += 1;
                    return Ok(Some(token));
                }
                '<' => {
                    if !buf.is_empty() {
                        let token = Token {
                            kind: TokenKind::Word,
                            lexeme: buf.clone(),
                            span: (token_start, self.pos),
                        };
                        buf.clear();
                        return Ok(Some(token));
                    }
                    let token = Token {
                        kind: TokenKind::RedirectIn,
                        lexeme: "<".to_string(),
                        span: (self.pos, self.pos + 1),
                    };
                    self.pos += 1;
                    return Ok(Some(token));
                }
                ';' => {
                    if !buf.is_empty() {
                        let token = Token {
                            kind: TokenKind::Word,
                            lexeme: buf.clone(),
                            span: (token_start, self.pos),
                        };
                        buf.clear();
                        return Ok(Some(token));
                    }
                    let token = Token {
                        kind: TokenKind::Semicolon,
                        lexeme: ";".to_string(),
                        span: (self.pos, self.pos + 1),
                    };
                    self.pos += 1;
                    return Ok(Some(token));
                }
                '(' => {
                    if !buf.is_empty() {
                        let token = Token {
                            kind: TokenKind::Word,
                            lexeme: buf.clone(),
                            span: (token_start, self.pos),
                        };
                        buf.clear();
                        return Ok(Some(token));
                    }
                    let token = Token {
                        kind: TokenKind::LParen,
                        lexeme: "(".to_string(),
                        span: (self.pos, self.pos + 1),
                    };
                    self.pos += 1;
                    return Ok(Some(token));
                }
                ')' => {
                    if !buf.is_empty() {
                        let token = Token {
                            kind: TokenKind::Word,
                            lexeme: buf.clone(),
                            span: (token_start, self.pos),
                        };
                        buf.clear();
                        return Ok(Some(token));
                    }
                    let token = Token {
                        kind: TokenKind::RParen,
                        lexeme: ")".to_string(),
                        span: (self.pos, self.pos + 1),
                    };
                    self.pos += 1;
                    return Ok(Some(token));
                }
                '\'' => {
                    self.pos += 1; // Skip the starting quote
                    let start = self.pos;
                    while self.pos < chars.len() {
                        if chars[self.pos] == '\'' {
                            let quoted = self.input[start..self.pos].to_string();
                            let span = (start, self.pos);
                            self.pos += 1; // Consume the closing quote
                            return Ok(Some(Token {
                                kind: TokenKind::Word,
                                lexeme: quoted,
                                span,
                            }));
                        }
                        self.pos += 1;
                    }
                    return Err(LexError::UnterminatedQuote('\'', start - 1));
                }
                '"' => {
                    self.pos += 1; // Skip the starting quote
                    let start = self.pos;
                    while self.pos < chars.len() {
                        if chars[self.pos] == '"' {
                            let quoted = self.input[start..self.pos].to_string();
                            let span = (start, self.pos); // only contents
                            self.pos += 1; // Consume the closing quote
                            return Ok(Some(Token {
                                kind: TokenKind::Word,
                                lexeme: quoted,
                                span,
                            }));
                        }
                        self.pos += 1;
                    }
                    return Err(LexError::UnterminatedQuote('"', start - 1));
                }
                _ => {
                    if buf.is_empty() {
                        token_start = self.pos;
                    }
                    buf.push(ch);
                    self.pos += 1;
                }
            }
        }

        // If there is any buffer left after the loop ends, return the last word token
        if !buf.is_empty() {
            let token = Token {
                kind: TokenKind::Word,
                lexeme: buf,
                span: (token_start, self.pos),
            };
            return Ok(Some(token));
        }

        // If the end is reached, return EOF
        if self.pos >= chars.len() {
            return Ok(Some(Token {
                kind: TokenKind::Eof,
                lexeme: "".to_string(),
                span: (self.pos, self.pos),
            }));
        }

        Ok(None)
    }

    pub fn tokenize_all(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        loop {
            match self.next_token()? {
                Some(token) => {
                    let is_eof = token.kind == TokenKind::Eof;
                    tokens.push(token);
                    if is_eof {
                        break;
                    }
                }
                None => break,
            }
        }
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
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize_all().unwrap();
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
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize_all().unwrap();
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
    fn test_single_quoted_word() {
        let input = "ls 'foo bar'";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize_all().unwrap();
        assert_eq!(
            tokens,
            vec![
                token(TokenKind::Word, "ls", (0, 2)),
                token(TokenKind::Word, "foo bar", (4, 11)),
                token(TokenKind::Eof, "", (12, 12)),
            ]
        );
    }

    #[test]
    fn test_double_quoted_word() {
        let input = "ls \"foo bar\"";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize_all().unwrap();
        assert_eq!(
            tokens,
            vec![
                token(TokenKind::Word, "ls", (0, 2)),
                token(TokenKind::Word, "foo bar", (4, 11)),
                token(TokenKind::Eof, "", (12, 12)),
            ]
        );
    }

    #[test]
    fn test_mixed_quotes() {
        let input = "echo 'foo' \"bar baz\" qux";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize_all().unwrap();
        assert_eq!(
            tokens,
            vec![
                token(TokenKind::Word, "echo", (0, 4)),
                token(TokenKind::Word, "foo", (6, 9)),
                token(TokenKind::Word, "bar baz", (12, 19)),
                token(TokenKind::Word, "qux", (21, 24)),
                token(TokenKind::Eof, "", (24, 24)),
            ]
        );
    }

    #[test]
    fn test_unterminated_single_quote() {
        let input = "echo 'foo";
        let mut lexer = Lexer::new(input);
        let result = lexer.tokenize_all();
        assert!(result.is_err());
        if let Err(LexError::UnterminatedQuote('\'', pos)) = result {
            assert_eq!(pos, 5); // ' の位置
        } else {
            panic!("Should be UnterminatedQuote error");
        }
    }

    #[test]
    fn test_unterminated_double_quote() {
        let input = "echo \"foo";
        let mut lexer = Lexer::new(input);
        let result = lexer.tokenize_all();
        assert!(result.is_err());
        if let Err(LexError::UnterminatedQuote('"', pos)) = result {
            assert_eq!(pos, 5); // " の位置
        } else {
            panic!("Should be UnterminatedQuote error");
        }
    }

    #[test]
    fn test_tokenize_mixed() {
        let input = r#"ls -l | grep 'foo bar' && echo done"#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize_all().unwrap();
        assert_eq!(
            tokens,
            vec![
                token(TokenKind::Word, "ls", (0, 2)),
                token(TokenKind::Word, "-l", (3, 5)),
                token(TokenKind::Pipe, "|", (6, 7)),
                token(TokenKind::Word, "grep", (8, 12)),
                token(TokenKind::Word, "foo bar", (14, 21)),
                token(TokenKind::And, "&&", (23, 25)),
                token(TokenKind::Word, "echo", (26, 30)),
                token(TokenKind::Word, "done", (31, 35)),
                token(TokenKind::Eof, "", (35, 35)),
            ]
        );
    }

    #[test]
    fn test_tokenize_empty() {
        let input = "";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize_all().unwrap();
        assert_eq!(
            tokens,
            vec![
                token(TokenKind::Eof, "", (0, 0)),
            ]
        );
    }
}

