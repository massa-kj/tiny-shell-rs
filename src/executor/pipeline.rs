use crate::ast::AstNode;
use crate::env::Environment;

pub fn execute_pipeline(lhs: &AstNode, rhs: &AstNode, env: &mut Environment) -> i32 {
    // When executing a pipe, the environment is referenced/duplicated as necessary for each process.
    0
}

