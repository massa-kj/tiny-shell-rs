pub mod default;

use std::fmt;
use crate::ast::{AstNode};

pub trait Parser {
    fn parse(&mut self) -> Result<AstNode, ParseError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    UnexpectedEof,
    UnexpectedToken {
        found: String,
        expected: Vec<String>,
        pos: usize,
    },
    UnmatchedParen {
        pos: usize,
    },
    UnclosedQuote {
        pos: usize,
        quote: char,
    },
    EmptyInput,
}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedEof => write!(f, "Unexpected end of input"),
            ParseError::UnexpectedToken { found, expected, pos } => {
                write!(f, "Unexpected token '{}' at position {}. Expected: {:?}", found, pos, expected)
            }
            ParseError::UnmatchedParen { pos } => write!(f, "Unmatched parenthesis at position {}", pos),
            ParseError::UnclosedQuote { pos, quote } => write!(f, "Unclosed quote '{}' at position {}", quote, pos),
            ParseError::EmptyInput => write!(f, "Input is empty"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{AstNode, RedirectKind, CommandNode, CommandKind};
    use crate::parser::default::DefaultParser;
    use crate::lexer::Lexer;

    fn lex_and_parse(src: &str) -> AstNode {
        let tokens = Lexer::tokenize_all(src);
        let mut parser = match tokens {
            Ok(ref toks) => DefaultParser::new(toks),
            Err(ref e) => {
                eprintln!("{}", e);
                panic!("Failed to tokenize input");
            }
        };
        parser.parse().unwrap()
    }

    #[test]
    fn test_simple_command() {
        let ast = lex_and_parse("echo hello");
        assert_eq!(
            ast,
            AstNode::Command(CommandNode {
                name: "echo".to_string(),
                args: vec!["hello".to_string()],
                kind: CommandKind::Simple,
            }),
        );
    }

    #[test]
    fn test_command_with_args() {
        let ast = lex_and_parse("grep foo bar");
        assert_eq!(
            ast,
            AstNode::Command(CommandNode {
                name: "grep".to_string(),
                args: vec!["foo".to_string(), "bar".to_string()],
                kind: CommandKind::Simple,
            }),
        );
    }

    #[test]
    fn test_pipeline() {
        let ast = lex_and_parse("ls | wc");
        assert_eq!(
            ast,
            AstNode::Pipeline(
                Box::new(AstNode::Command(CommandNode {
                    name: "ls".to_string(),
                    args: vec![],
                    kind: CommandKind::Simple,
                })),
                Box::new(AstNode::Command(CommandNode {
                    name: "wc".to_string(),
                    args: vec![],
                    kind: CommandKind::Simple,
                })),
            )
        );
    }

    #[test]
    fn test_multistage_pipeline() {
        let ast = lex_and_parse("ls | grep foo | wc");
        assert_eq!(
            ast,
            AstNode::Pipeline(
                Box::new(AstNode::Pipeline(
                    Box::new(AstNode::Command(CommandNode {
                        name: "ls".to_string(),
                        args: vec![],
                        kind: CommandKind::Simple,
                    })),
                    Box::new(AstNode::Command(CommandNode {
                        name: "grep".to_string(),
                        args: vec!["foo".to_string()],
                        kind: CommandKind::Simple,
                    })),
                )),
                Box::new(AstNode::Command(CommandNode {
                    name: "wc".to_string(),
                    args: vec![],
                    kind: CommandKind::Simple,
                })),
            )
        );
    }

    #[test]
    fn test_redirect_out() {
        let ast = lex_and_parse("ls > out.txt");
        assert_eq!(
            ast,
            AstNode::Redirect {
                node: Box::new(AstNode::Command(CommandNode {
                    name: "ls".to_string(),
                    args: vec![],
                    kind: CommandKind::Simple,
                })),
                kind: RedirectKind::Out,
                file: "out.txt".to_string(),
            }
        );
    }

    #[test]
    fn test_pipeline_with_redirect() {
        let ast = lex_and_parse("ls | wc > out.txt");
        assert_eq!(
            ast,
            AstNode::Redirect {
                node: Box::new(AstNode::Pipeline(
                    Box::new(AstNode::Command(CommandNode {
                        name: "ls".to_string(),
                        args: vec![],
                        kind: CommandKind::Simple,
                    })),
                    Box::new(AstNode::Command(CommandNode {
                        name: "wc".to_string(),
                        args: vec![],
                        kind: CommandKind::Simple,
                    })),
                )),
                kind: RedirectKind::Out,
                file: "out.txt".to_string(),
            }
        );
    }

    #[test]
    fn test_and_or_sequence() {
        let ast = lex_and_parse("echo ok && ls || echo err; echo end");
        assert_eq!(
            ast,
            AstNode::Sequence(
                Box::new(AstNode::Or(
                    Box::new(AstNode::And(
                        Box::new(AstNode::Command(CommandNode {
                            name: "echo".to_string(),
                            args: vec!["ok".to_string()],
                            kind: CommandKind::Simple,
                        })),
                        Box::new(AstNode::Command(CommandNode {
                            name: "ls".to_string(),
                            args: vec![],
                            kind: CommandKind::Simple,
                        })),
                    )),
                    Box::new(AstNode::Command(CommandNode {
                        name: "echo".to_string(),
                        args: vec!["err".to_string()],
                        kind: CommandKind::Simple,
                    })),
                )),
                Box::new(AstNode::Command(CommandNode {
                    name: "echo".to_string(),
                    args: vec!["end".to_string()],
                    kind: CommandKind::Simple,
                })),
            )
        );
    }

    #[test]
    fn test_subshell() {
        let ast = lex_and_parse("(ls | wc)");
        assert_eq!(
            ast,
            AstNode::Subshell(Box::new(AstNode::Pipeline(
                Box::new(AstNode::Command(CommandNode {
                    name: "ls".to_string(),
                    args: vec![],
                    kind: CommandKind::Simple,
                })),
                Box::new(AstNode::Command(CommandNode {
                    name: "wc".to_string(),
                    args: vec![],
                    kind: CommandKind::Simple,
                })),
            )))
        );
    }

    #[test]
    fn test_command_with_multiple_redirects() {
        let ast = lex_and_parse("cat < in.txt > out.txt");
        assert_eq!(
            ast,
            AstNode::Redirect {
                node: Box::new(AstNode::Redirect {
                    node: Box::new(AstNode::Command(CommandNode {
                        name: "cat".to_string(),
                        args: vec![],
                        kind: CommandKind::Simple,
                    })),
                    kind: RedirectKind::In,
                    file: "in.txt".to_string(),
                }),
                kind: RedirectKind::Out,
                file: "out.txt".to_string(),
            }
        );
    }

    #[test]
    fn test_complex_oneliner() {
        let ast = lex_and_parse("cat | grep -q foo | wc < in.txt > out.txt");
        assert_eq!(
            ast,
            AstNode::Redirect {
                node: Box::new(AstNode::Redirect {
                    node: Box::new(AstNode::Pipeline(
                        Box::new(AstNode::Pipeline(
                            Box::new(AstNode::Command(CommandNode {
                                name: "cat".to_string(),
                                args: vec![],
                                kind: CommandKind::Simple,
                            })),
                            Box::new(AstNode::Command(CommandNode {
                                name: "grep".to_string(),
                                args: vec!["-q".to_string(), "foo".to_string()],
                                kind: CommandKind::Simple,
                            })),
                        )),
                        Box::new(AstNode::Command(CommandNode {
                            name: "wc".to_string(),
                            args: vec![],
                            kind: CommandKind::Simple,
                        })),
                    )),
                    kind: RedirectKind::In,
                    file: "in.txt".to_string(),
                }),
                kind: RedirectKind::Out,
                file: "out.txt".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_error() {
        let tokens = Lexer::tokenize_all("&& ls");
        let mut parser = match tokens {
            Ok(ref toks) => DefaultParser::new(toks),
            Err(ref e) => {
                eprintln!("{}", e);
                panic!("Failed to tokenize input");
            }
        };
        assert!(parser.parse().is_err());
    }
}

