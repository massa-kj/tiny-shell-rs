use crate::parser::{Parser, ParseError};
use crate::ast::{AstNode, CommandNode};
use crate::lexer::token::{Token, TokenKind};

pub struct DefaultParser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> DefaultParser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }
    fn next(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }
    fn expect_word(&mut self) -> Result<String, ParseError> {
        match self.next() {
            Some(tok) if matches!(tok.kind, TokenKind::Word) => Ok(tok.lexeme.clone()),
            Some(t) => Err(ParseError::UnexpectedToken {
                found: format!("{:?}", t.kind),
                expected: vec!["Word".to_string()],
                pos: self.pos,
            }),
            None => Err(ParseError::EmptyInput),
        }
    }
    fn consume(&mut self, pat: &TokenKind) -> bool {
        if let Some(tok) = self.tokens.get(self.pos) {
            if &tok.kind == pat {
                self.pos += 1;
                return true;
            }
        }
        false
    }
}

// Top-down recursive descent parser
impl<'a> Parser for DefaultParser<'a> {
    fn parse(&mut self) -> Result<AstNode, ParseError> {
        if self.tokens.is_empty() {
            return Err(ParseError::EmptyInput);
        }
        self.parse_sequence()
    }
}

impl<'a> DefaultParser<'a> {
    fn parse_sequence(&mut self) -> Result<AstNode, ParseError> {
        let mut node = self.parse_and_or()?;
        while self.consume(&TokenKind::Semicolon) {
            let rhs = self.parse_and_or()?;
            node = AstNode::Sequence(Box::new(node), Box::new(rhs));
        }
        Ok(node)
    }

    fn parse_and_or(&mut self) -> Result<AstNode, ParseError> {
        let mut node = self.parse_pipeline()?;
        loop {
            if self.consume(&TokenKind::And) {
                let rhs = self.parse_pipeline()?;
                node = AstNode::And(Box::new(node), Box::new(rhs));
            } else if self.consume(&TokenKind::Or) {
                let rhs = self.parse_pipeline()?;
                node = AstNode::Or(Box::new(node), Box::new(rhs));
            } else {
                break;
            }
        }
        Ok(node)
    }

    fn parse_pipeline(&mut self) -> Result<AstNode, ParseError> {
        // First, get the smallest syntactic unit.
        let mut node = self.parse_command_like()?;
        // Connected by pipes
        while self.consume(&TokenKind::Pipe) {
            let rhs = self.parse_command_like()?;
            node = AstNode::Pipeline(Box::new(node), Box::new(rhs));
        }
        // Add a redirect to the entire pipe
        self.parse_with_redirect(node)
    }

    // build "pipe elements" such as commands and subshells
    fn parse_command_like(&mut self) -> Result<AstNode, ParseError> {
        if self.consume(&TokenKind::LParen) {
            let node = self.parse_sequence()?;
            if !self.consume(&TokenKind::RParen) {
                return Err(ParseError::UnmatchedParen {
                    pos: self.pos,
                });
            }
            Ok(AstNode::Subshell(Box::new(node)))
        } else {
            // Command alone
            let mut args = Vec::new();
            while let Some(tok) = self.peek() {
                if let TokenKind::Word = &tok.kind {
                    args.push(tok.lexeme.clone());
                    self.pos += 1;
                } else {
                    break;
                }
            }
            if args.is_empty() {
                return Err(ParseError::EmptyInput);
            }
            Ok(AstNode::Command(CommandNode {
                name: args[0].clone(),
                args: args[1..].to_vec(),
                kind: crate::ast::CommandKind::Simple,
            }))
        }
    }

    // Add a redirect after any node
    fn parse_with_redirect(&mut self, mut node: AstNode) -> Result<AstNode, ParseError> {
        loop {
            if self.consume(&TokenKind::RedirectOut) {
                let filename = self.expect_word()?;
                node = AstNode::Redirect {
                    node: Box::new(node),
                    kind: crate::ast::RedirectKind::Out,
                    file: filename,
                };
            } else if self.consume(&TokenKind::RedirectIn) {
                let filename = self.expect_word()?;
                node = AstNode::Redirect {
                    node: Box::new(node),
                    kind: crate::ast::RedirectKind::In,
                    file: filename,
                };
            } else {
                break;
            }
        }
        Ok(node)
    }
}

