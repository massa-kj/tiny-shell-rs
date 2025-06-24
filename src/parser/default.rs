use crate::parser::Parser;
use crate::ast::{AstNode, TokenKind};

pub struct DefaultParser<'a> {
    tokens: &'a [TokenKind],
    pos: usize,
}

impl<'a> DefaultParser<'a> {
    pub fn new(tokens: &'a [TokenKind]) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&TokenKind> {
        self.tokens.get(self.pos)
    }
    fn next(&mut self) -> Option<&TokenKind> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }
    fn expect_word(&mut self) -> Result<String, String> {
        match self.next() {
            Some(TokenKind::Word(s)) => Ok(s.clone()),
            Some(t) => Err(format!("unexpected token: {:?}", t)),
            None => Err("unexpected end of input".to_string()),
        }
    }
    fn consume(&mut self, pat: &TokenKind) -> bool {
        if self.tokens.get(self.pos) == Some(pat) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
}

// Top-down recursive descent parser
impl<'a> Parser for DefaultParser<'a> {
    fn parse(&mut self) -> Result<AstNode, String> {
        self.parse_sequence()
    }
}

impl<'a> DefaultParser<'a> {
    fn parse_sequence(&mut self) -> Result<AstNode, String> {
        let mut node = self.parse_and_or()?;
        while self.consume(&TokenKind::Semicolon) {
            let rhs = self.parse_and_or()?;
            node = AstNode::Sequence(Box::new(node), Box::new(rhs));
        }
        Ok(node)
    }

    fn parse_and_or(&mut self) -> Result<AstNode, String> {
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

    fn parse_pipeline(&mut self) -> Result<AstNode, String> {
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
    fn parse_command_like(&mut self) -> Result<AstNode, String> {
        if self.consume(&TokenKind::LParen) {
            let node = self.parse_sequence()?;
            if !self.consume(&TokenKind::RParen) {
                return Err("missing ')'".to_string());
            }
            Ok(AstNode::Subshell(Box::new(node)))
        } else {
            // Command alone
            let mut args = Vec::new();
            while let Some(TokenKind::Word(s)) = self.peek() {
                args.push(s.clone());
                self.pos += 1;
            }
            if args.is_empty() {
                return Err("expected command".to_string());
            }
            Ok(AstNode::Command {
                name: args[0].clone(),
                args: args[1..].to_vec(),
                kind: crate::ast::CommandKind::Simple,
            })
        }
    }

    // Add a redirect after any node
    fn parse_with_redirect(&mut self, mut node: AstNode) -> Result<AstNode, String> {
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

