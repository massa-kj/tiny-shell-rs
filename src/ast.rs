#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    Command {
        name: String,
        args: Vec<String>,
        kind: CommandKind,
    },
    Pipeline(Box<AstNode>, Box<AstNode>),
    Redirect {
        node: Box<AstNode>,
        kind: RedirectKind,
        file: String,
    },
    // Subshell
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandKind {
    Builtin,
    External,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedirectKind {
    In,
    Out,
    Append,
}

