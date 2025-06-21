#[derive(Debug)]
pub enum TokenKind {
    Word,
    // Operator, LParen, ...
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandKind {
    Simple,
//     Builtin,
//     External,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    Command {
        name: String,
        args: Vec<String>,
        kind: CommandKind,
    },
    Empty,
    // Pipeline(Box<AstNode>, Box<AstNode>),
    // Redirect {
    //     node: Box<AstNode>,
    //     kind: RedirectKind,
    //     file: String,
    // },
    // Subshell,
}

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub enum RedirectKind {
//     In,
//     Out,
//     Append,
// }
//
