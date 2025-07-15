use crate::parser::{Parser, ParseError};
use crate::ast::{AstNode, CommandNode};
use crate::lexer::{Token, TokenKind};

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
        let mut node = self.parse_or()?;
        while self.consume(&TokenKind::Semicolon) {
            let rhs = self.parse_or()?;
            let seq = vec![node, rhs];
            node = AstNode::Sequence(seq);
        }
        Ok(node)
    }

    fn parse_or(&mut self) -> Result<AstNode, ParseError> {
        let mut node = self.parse_and()?;

        while self.consume(&TokenKind::Or) {
            let rhs = self.parse_and()?;
            node = AstNode::Or(Box::new(node), Box::new(rhs));
        }
        Ok(node)
    }

    fn parse_and(&mut self) -> Result<AstNode, ParseError> {
        let mut node = self.parse_pipeline()?;

        while self.consume(&TokenKind::And) {
            let rhs = self.parse_pipeline()?;
            node = AstNode::And(Box::new(node), Box::new(rhs));
        }
        Ok(node)
    }

    fn parse_pipeline(&mut self) -> Result<AstNode, ParseError> {
        // First, get the smallest syntactic unit.
        let mut nodes = vec![self.parse_command_like()?];
        // Connected by pipes
        while self.consume(&TokenKind::Pipe) {
            let rhs = self.parse_command_like()?;
            nodes.push(rhs);
        }
        // Add a redirect to the entire pipe
        if nodes.len() == 1 {
            return self.parse_with_redirect(nodes.remove(0));
        } else {
            return self.parse_with_redirect(AstNode::Pipeline(nodes))
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::ast::{AstNode, RedirectKind, CommandNode, CommandKind};

    fn lex_and_parse(src: &str) -> AstNode {
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize_all();
        let mut parser = match tokens {
            Ok(ref toks) => DefaultParser::new(toks),
            Err(ref e) => {
                eprintln!("{}", e);
                panic!("Failed to tokenize input");
            }
        };
        parser.parse().unwrap()
    }

    // Parsing empty input (ParseError::EmptyInput)
    #[test]
    fn test_empty_input() {
        let mut lexer = Lexer::new("");
        let tokens = lexer.tokenize_all();
        let mut parser = match tokens {
            Ok(ref toks) => DefaultParser::new(toks),
            Err(ref e) => {
                eprintln!("{}", e);
                panic!("Failed to tokenize input");
            }
        };
        assert!(matches!(parser.parse(), Err(ParseError::EmptyInput)));
    }

    // Simple command (e.g., echo hello)
    #[test]
    fn test_simple_command() {
        let ast = lex_and_parse("echo hello");
        assert_eq!(
            ast,
            AstNode::Command(CommandNode {
                name: "echo".to_string(),
                args: vec!["hello".to_string()],
                kind: CommandKind::Simple,
            })
        );
    }

    // Parsing command arguments (e.g., ls -l /tmp)
    #[test]
    fn test_command_with_args() {
        let ast = lex_and_parse("ls -l /tmp");
        assert_eq!(
            ast,
            AstNode::Command(CommandNode {
                name: "ls".to_string(),
                args: vec!["-l".to_string(), "/tmp".to_string()],
                kind: CommandKind::Simple,
            })
        );
    }

    // Semicolon-separated command sequence (e.g., ls; pwd)
    #[test]
    fn test_command_sequence() {
        let ast = lex_and_parse("ls; pwd");
        assert_eq!(
            ast,
            AstNode::Sequence(vec![
                AstNode::Command(CommandNode {
                    name: "ls".to_string(),
                    args: vec![],
                    kind: CommandKind::Simple,
                }),
                AstNode::Command(CommandNode {
                    name: "pwd".to_string(),
                    args: vec![],
                    kind: CommandKind::Simple,
                })
            ])
        );
    }

    // Parsing AND/OR operators (e.g., true && false, true || false)
    #[test]
    fn test_and_or_operators() {
        let ast = lex_and_parse("true && false || true");
        assert_eq!(
            ast,
            AstNode::Or(
                Box::new(AstNode::And(
                    Box::new(AstNode::Command(CommandNode {
                        name: "true".to_string(),
                        args: vec![],
                        kind: CommandKind::Simple,
                    })),
                    Box::new(AstNode::Command(CommandNode {
                        name: "false".to_string(),
                        args: vec![],
                        kind: CommandKind::Simple,
                    }))
                )),
                Box::new(AstNode::Command(CommandNode {
                    name: "true".to_string(),
                    args: vec![],
                    kind: CommandKind::Simple,
                }))
            )
        );
    }

    // Parsing pipe operator (e.g., ls | grep foo)
    #[test]
    fn test_pipeline() {
        let ast = lex_and_parse("ls | grep foo");
        assert_eq!(
            ast,
            AstNode::Pipeline(vec![
                AstNode::Command(CommandNode {
                    name: "ls".to_string(),
                    args: vec![],
                    kind: CommandKind::Simple,
                }),
                AstNode::Command(CommandNode {
                    name: "grep".to_string(),
                    args: vec!["foo".to_string()],
                    kind: CommandKind::Simple,
                }),
            ])
        );
    }

    // Parsing redirection (output/input) (e.g., echo foo > out.txt, cat < in.txt)
    #[test]
    fn test_redirection() {
        let ast = lex_and_parse("echo foo > out.txt");
        assert_eq!(
            ast,
            AstNode::Redirect {
                node: Box::new(AstNode::Command(CommandNode {
                    name: "echo".to_string(),
                    args: vec!["foo".to_string()],
                    kind: CommandKind::Simple,
                })),
                kind: RedirectKind::Out,
                file: "out.txt".to_string(),
            }
        );

        let ast = lex_and_parse("cat < in.txt");
        assert_eq!(
            ast,
            AstNode::Redirect {
                node: Box::new(AstNode::Command(CommandNode {
                    name: "cat".to_string(),
                    args: vec![],
                    kind: CommandKind::Simple,
                })),
                kind: RedirectKind::In,
                file: "in.txt".to_string(),
            }
        );
    }

    // Parsing subshells (e.g., (echo foo; ls))
    #[test]
    fn test_subshell() {
        let ast = lex_and_parse("(echo foo; ls)");
        assert_eq!(
            ast,
            AstNode::Subshell(
                Box::new(AstNode::Sequence(vec![
                    AstNode::Command(CommandNode {
                        name: "echo".to_string(),
                        args: vec!["foo".to_string()],
                        kind: CommandKind::Simple,
                    }),
                    AstNode::Command(CommandNode {
                        name: "ls".to_string(),
                        args: vec![],
                        kind: CommandKind::Simple,
                    })
                ])
            ))
        );
    }

    // Combinations inside subshells (redirection, pipe, AND/OR)
    #[test]
    fn test_complex_subshell() {
        let ast = lex_and_parse("(ls | grep foo) && echo ok > result.txt");
        assert_eq!(
            ast,
            AstNode::And(
                Box::new(AstNode::Subshell(
                    Box::new(AstNode::Pipeline(vec![
                        AstNode::Command(CommandNode {
                            name: "ls".to_string(),
                            args: vec![],
                            kind: CommandKind::Simple,
                        }),
                        AstNode::Command(CommandNode {
                            name: "grep".to_string(),
                            args: vec!["foo".to_string()],
                            kind: CommandKind::Simple,
                        })
                    ])
                ))),
                Box::new(AstNode::Redirect {
                    node: Box::new(AstNode::Command(CommandNode {
                        name: "echo".to_string(),
                        args: vec!["ok".to_string()],
                        kind: CommandKind::Simple,
                    })),
                    kind: RedirectKind::Out,
                    file: "result.txt".to_string(),
                })
            )
        );
    }

    // Invalid tokens (e.g., unknown symbols or malformed syntax)
    // #[test]
    // fn test_invalid_tokens() {
    //     let invalid_cases = vec![
    //         "echo > out.txt", // Missing command before redirection
    //         "ls |",           // Pipe without command after
    //         "&& echo ok",     // AND without left-hand command
    //         "|| echo err",    // OR without left-hand command
    //         "(echo foo",      // Unmatched parenthesis
    //         "echo foo > out.txt < in.txt", // Multiple redirections
    //     ];
    //
    //     for case in invalid_cases {
    //         let ast = lex_and_parse(case);
    //         assert!(matches!(ast, Err(ParseError::UnexpectedToken { .. })));
    //     }
    // }

    // Unmatched parentheses (e.g., (echo foo)
    // #[test]
    // fn test_unmatched_parentheses() {
    //     let ast = lex_and_parse("(echo foo");
    //     assert!(matches!(ast, Err(ParseError::UnmatchedParen { .. })));
    // }

    // Empty command (e.g., ;, ||, && followed by no command)
    // #[test]
    // fn test_empty_command() {
    //     let ast = lex_and_parse(";");
    //     assert!(matches!(ast, Err(ParseError::EmptyInput)));
    //
    //     let ast = lex_and_parse("||");
    //     assert!(matches!(ast, Err(ParseError::EmptyInput)));
    //
    //     let ast = lex_and_parse("&&");
    //     assert!(matches!(ast, Err(ParseError::EmptyInput)));
    // }

    // Multiple redirections (e.g., echo foo > out.txt < in.txt)
    #[test]
    fn test_multiple_redirections() {
        let ast = lex_and_parse("echo foo > out.txt < in.txt");
        assert_eq!(
            ast,
            AstNode::Redirect {
                node: Box::new(AstNode::Redirect {
                    node: Box::new(AstNode::Command(CommandNode {
                        name: "echo".to_string(),
                        args: vec!["foo".to_string()],
                        kind: CommandKind::Simple,
                    })),
                    kind: RedirectKind::Out,
                    file: "out.txt".to_string(),
                }),
                kind: RedirectKind::In,
                file: "in.txt".to_string(),
            }
        );
    }

    // Complex syntax combinations (e.g., (ls | grep foo) && echo ok > result.txt)
    // #[test]
    // fn test_complex_syntax() {
    //     let ast = lex_and_parse("ls -l > ls.txt; cat < ls.txt | grep \"txt\" | wc > output.txt");
    //     // let ast = lex_and_parse("(ls | grep foo) && echo ok > result.txt");
    //     assert_eq!(
    //         ast,
    //         AstNode::Sequence(
    //             Box::new(AstNode::Redirect {
    //                 node: Box::new(AstNode::Command(CommandNode {
    //                     name: "ls".to_string(),
    //                     args: vec!["-l".to_string()],
    //                     kind: CommandKind::Simple,
    //                 })),
    //                 kind: RedirectKind::Out,
    //                 file: "ls.txt".to_string(),
    //             }),
    //             Box::new(AstNode::Pipeline(
    //                 Box::new(AstNode::Redirect {
    //                     node: Box::new(AstNode::Command(CommandNode {
    //                         name: "cat".to_string(),
    //                         args: vec!["ls.txt".to_string()],
    //                         kind: CommandKind::Simple,
    //                     })),
    //                     kind: RedirectKind::In,
    //                     file: "".to_string(), // Input redirection does not require a file
    //                 }),
    //                 Box::new(AstNode::Pipeline(
    //                     Box::new(AstNode::Command(CommandNode {
    //                         name: "grep".to_string(),
    //                         args: vec!["txt".to_string()],
    //                         kind: CommandKind::Simple,
    //                     })),
    //                     Box::new(AstNode::Redirect {
    //                         node: Box::new(AstNode::Command(CommandNode {
    //                             name: "wc".to_string(),
    //                             args: vec![],
    //                             kind: CommandKind::Simple,
    //                         })),
    //                         kind: RedirectKind::Out,
    //                         file: "output.txt".to_string(),
    //                     }),
    //                 )),
    //             )),
    //         )
    //     );
    // }
}
