use crate::ast::AstNode;
use crate::env::Environment;

pub fn expand(node: &AstNode, _env: &Environment) -> AstNode {
    node.clone()
}

