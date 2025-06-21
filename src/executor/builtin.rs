use crate::ast::AstNode;
use crate::env::Environment;

pub fn execute_builtin(name: &str, args: &[String], env: &mut Environment) -> i32 {
    // cd, export, unset, exit... always refer to/update env
    0
}

