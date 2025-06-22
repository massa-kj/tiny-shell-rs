use crate::ast::{AstNode, TokenKind, CommandKind, RedirectKind};

// use crate::builtins::is_builtin_command;

pub struct Parser<'a> {
    tokens: &'a [TokenKind],
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [TokenKind]) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<AstNode, String> {
        self.parse_sequence()
    }

    // separated by ;
    fn parse_sequence(&mut self) -> Result<AstNode, String> {
        let mut node = self.parse_and_or()?;
        while self.consume(&TokenKind::Semicolon) {
            let rhs = self.parse_and_or()?;
            node = AstNode::Sequence(Box::new(node), Box::new(rhs));
        }
        Ok(node)
    }

    // AND/OR (&&, ||)
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

    // pipe
    fn parse_pipeline(&mut self) -> Result<AstNode, String> {
        let mut node = self.parse_command()?;
        while self.consume(&TokenKind::Pipe) {
            let rhs = self.parse_command()?;
            node = AstNode::Pipeline(Box::new(node), Box::new(rhs));
        }
        Ok(node)
    }

    // command, ridirect, subshell
    fn parse_command(&mut self) -> Result<AstNode, String> {
        // 1. subshell
        if self.consume(&TokenKind::LParen) {
            let node = self.parse_sequence()?;
            if !self.consume(&TokenKind::RParen) {
                return Err("missing ')'".into());
            }
            Ok(AstNode::Subshell(Box::new(node)))
        } else {
            // 2. simple command
            let mut args = Vec::new();
            while let Some(TokenKind::Word(s)) = self.peek() {
                args.push(s.clone());
                self.pos += 1;
            }
            if args.is_empty() {
                return Err("expected command".into());
            }
            // 3. ridirect
            let mut node = AstNode::Command {
                name: args[0].clone(),
                args: args[1..].to_vec(),
                kind: crate::ast::CommandKind::Simple,
            };
            // (Wrap any redirects on the right)
            // loop {
            //     if self.consume(&Token::RedirectOut) {
            //         if let Some(Token::Word(file)) = self.peek() {
            //             self.pos += 1;
            //             node = AstNode::Redirect {
            //                 node: Box::new(node),
            //                 kind: crate::ast::RedirectKind::Out,
            //                 file: file.clone(),
            //             };
            //         } else {
            //             return Err("expected filename after '>'".into());
            //         }
            //     } else if self.consume(&Token::RedirectIn) {
            //         if let Some(Token::Word(file)) = self.peek() {
            //             self.pos += 1;
            //             node = AstNode::Redirect {
            //                 node: Box::new(node),
            //                 kind: crate::ast::RedirectKind::In,
            //                 file: file.clone(),
            //             };
            //         } else {
            //             return Err("expected filename after '<'".into());
            //         }
            //     } else {
            //         break;
            //     }
            // }
            Ok(node)
        }
    }

    // Check current token
    fn peek(&self) -> Option<&TokenKind> {
        self.tokens.get(self.pos)
    }

    // Consume specific tokens
    fn consume(&mut self, tok: &TokenKind) -> bool {
        if self.tokens.get(self.pos) == Some(tok) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
}

// pub fn parse(tokens: &[TokenKind]) -> Result<AstNode, String> {
//     let mut token_slice = tokens;
//     parse_pipeline(&mut token_slice)
// }
//
// fn parse_pipeline(tokens: &mut &[TokenKind]) -> Result<AstNode, String> {
//     let mut node = parse_command(tokens)?;
//
//     while let Some(TokenKind::Pipe) = tokens.first() {
//         *tokens = &tokens[1..];
//         let right = parse_command(tokens)?;
//         node = AstNode::Pipeline(Box::new(node), Box::new(right));
//     }
//     Ok(node)
// }
//
// fn parse_command(tokens: &mut &[TokenKind]) -> Result<AstNode, String> {
//     let mut args = Vec::new();
//     let mut kind = CommandKind::External;
//     while let Some(TokenKind::Word(word)) = tokens.first() {
//         args.push(word.clone());
//         *tokens = &tokens[1..];
//     }
//     if args.is_empty() {
//         return Err("Empty command".to_string());
//     }
//     if is_builtin_command(&args[0]) {
//         kind = CommandKind::Builtin;
//     }
//
//     if let Some(TokenKind::RedirectOut) = tokens.first() {
//         *tokens = &tokens[1..];
//         if let Some(TokenKind::Word(file)) = tokens.first() {
//             let file = file.clone();
//             *tokens = &tokens[1..];
//             let node = AstNode::Command {
//                 name: args[0].clone(),
//                 args: args[1..].to_vec(),
//                 kind,
//             };
//             return Ok(AstNode::Redirect {
//                 node: Box::new(node),
//                 kind: RedirectKind::Out,
//                 file,
//             });
//         } else {
//             return Err("Expected file after '>'".to_string());
//         }
//     }
//
//     Ok(AstNode::Command {
//         name: args[0].clone(),
//         args: args[1..].to_vec(),
//         kind,
//     })
// }

