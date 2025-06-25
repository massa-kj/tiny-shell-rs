use crate::ast::AstNode;
use crate::environment::Environment;

pub fn expand(node: &AstNode, _env: &Environment) -> AstNode {
    node.clone()
}

