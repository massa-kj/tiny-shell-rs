#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Word(String),              // command or argument
    Pipe,                      // |
    RedirectIn,                // <
    RedirectOut,               // > (file, append)
    Semicolon,                 // ;
    And,                       // &&
    Or,                        // ||
    LParen,                    // (
    RParen,                    // )
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    let mut buf = String::new();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' | '\n' => {
                if !buf.is_empty() {
                    tokens.push(Token::Word(buf.clone()));
                    buf.clear();
                }
                chars.next();
            }
            '|' => {
                if !buf.is_empty() {
                    tokens.push(Token::Word(buf.clone()));
                    buf.clear();
                }
                chars.next();
                if chars.peek() == Some(&'|') {
                    chars.next();
                    tokens.push(Token::Or);
                } else {
                    tokens.push(Token::Pipe);
                }
            }
            '&' => {
                if !buf.is_empty() {
                    tokens.push(Token::Word(buf.clone()));
                    buf.clear();
                }
                chars.next();
                if chars.peek() == Some(&'&') {
                    chars.next();
                    tokens.push(Token::And);
                }
            }
            '>' => {
                if !buf.is_empty() {
                    tokens.push(Token::Word(buf.clone()));
                    buf.clear();
                }
                chars.next();
                tokens.push(Token::RedirectOut);
            }
            '<' => {
                if !buf.is_empty() {
                    tokens.push(Token::Word(buf.clone()));
                    buf.clear();
                }
                chars.next();
                tokens.push(Token::RedirectIn);
            }
            ';' => {
                if !buf.is_empty() {
                    tokens.push(Token::Word(buf.clone()));
                    buf.clear();
                }
                chars.next();
                tokens.push(Token::Semicolon);
            }
            '(' => {
                if !buf.is_empty() {
                    tokens.push(Token::Word(buf.clone()));
                    buf.clear();
                }
                chars.next();
                tokens.push(Token::LParen);
            }
            ')' => {
                if !buf.is_empty() {
                    tokens.push(Token::Word(buf.clone()));
                    buf.clear();
                }
                chars.next();
                tokens.push(Token::RParen);
            }
            '"' | '\'' => {
                chars.next();
                while let Some(&nc) = chars.peek() {
                    if nc == '"' {
                        chars.next();
                        break;
                    } else {
                        buf.push(nc);
                        chars.next();
                    }
                }
            }
            _ => {
                buf.push(ch);
                chars.next();
            }
        }
    }

    if !buf.is_empty() {
        tokens.push(Token::Word(buf));
    }

    tokens
}

