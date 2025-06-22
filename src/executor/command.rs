use std::env;
use std::fs::{self};
use std::path::Path;
use std::process::Command;

use crate::ast::AstNode;
use crate::env::Environment;
use crate::builtins::BuiltinManager;

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

    let path = match super::command::resolve_command_path(name) {
        Some(p) => p,
        None => {
            eprintln!("tiny-shell: command not found or failed");
            return 127 // The shell's standard "command not found" exit code
        }
    };

    // External command execution
    let mut cmd = Command::new(path);
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

fn resolve_command_path(cmd: &str) -> Option<String> {
    if cmd.contains('/') {
        let path = Path::new(cmd);
        if path.exists() && path.is_file() {
            return Some(cmd.to_string());
        } else {
            return None;
        }
    }

    // Otherwise, search in PATH
    if let Ok(paths) = env::var("PATH") {
        for dir in env::split_paths(&paths) {
            let full_path = dir.join(cmd);
            if full_path.exists() && fs::metadata(&full_path).map(|m| m.is_file()).unwrap_or(false) {
                return Some(full_path.to_string_lossy().to_string());
            }
        }
    }

    None
}

