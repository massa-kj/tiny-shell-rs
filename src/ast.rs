#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    Command(CommandNode),
    Pipeline(Vec<AstNode>),
    Redirect {
        node: Box<AstNode>,
        kind: RedirectKind,
        file: String,
    },
    Sequence(Box<AstNode>, Box<AstNode>),
    And(Box<AstNode>, Box<AstNode>),
    Or(Box<AstNode>, Box<AstNode>),
    Subshell(Box<AstNode>),
    Compound(CompoundNode),
    // Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandNode {
    pub name: String,
    pub args: Vec<String>,
    pub kind: CommandKind,
    // pub assignments: Vec<(String, String)>, // FOO=bar cmd
    // heredoc
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandKind {
    Simple,
    Builtin,
    External,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedirectKind {
    In,
    Out,
    Append,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompoundNode {
    Group(Vec<AstNode>),
    If {
        cond: Box<AstNode>,
        then_branch: Vec<AstNode>,
        else_branch: Option<Vec<AstNode>>,
    },
    While {
        cond: Box<AstNode>,
        body: Vec<AstNode>,
    },
    // for, function, etc
}

