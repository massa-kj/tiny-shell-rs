mod command;
mod pipeline;

use crate::ast::AstNode;
use crate::env::Environment;

pub fn execute(node: &AstNode, env: &mut Environment) -> i32 {
    match node {
        AstNode::Command { .. } => command::execute_command(node, env),
        AstNode::Pipeline(lhs, rhs) => pipeline::execute_pipeline(lhs, rhs, env),
        // AstNode::Redirect { .. } => redirect::execute_redirect(node, env),
        // AstNode::Subshell(sub) => subshell::execute_subshell(sub, env),
        // ...etc
        _ => 0,
    }
}

