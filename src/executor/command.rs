use crate::ast::AstNode;
use crate::env::Environment;
use crate::builtins::BuiltinManager;
// use crate::executor::builtin;
use std::process::Command;

pub fn execute_command(node: &AstNode, env: &mut Environment) -> i32 {
    let (name, args) = match node {
        AstNode::Command { name, args, .. } => (name, args),
        _ => return 1,
    };

    // Built-in command execution
    let builtin_manager = BuiltinManager::new();
    if builtin_manager.is_builtin(name) {
        return builtin_manager.execute(name, args, env);
    }

    // External command execution
    let mut cmd = Command::new(name);
    cmd.args(args);

    // Pass shell environment variables to external commands (empty for now, will be expanded from env.vars/env.envs etc.)
    // e.g. for (k, v) in &env.envs { cmd.env(k, v); }

    // execution & wait
    match cmd.status() {
        // TODO: &: background execution -> spawn
        Ok(status) => status.code().unwrap_or(1),
        Err(e) => {
            eprintln!("tiny-shell: command not found or failed: {}", e);
            127 // The shell's standard "command not found" exit code
        }
    }
}

