use std::process::Command;
use super::redirect::RedirectHandler;
use crate::ast::{AstNode, CommandNode, CommandKind};
use crate::environment::Environment;
use crate::executor::{
    Executor,
    ExecStatus, ExecError,
    builtins::BuiltinManager,
    path_resolver::PathResolver,
    // signal::SignalHandler,
};

pub struct RecursiveExecutor {
    pub builtin_registry: BuiltinManager,
    pub path_resolver: PathResolver,
    // pub redirect_handler: RedirectHandler,
    // pub signal_handler: SignalHandler,
}

impl RecursiveExecutor {
    pub fn new() -> Self {
        RecursiveExecutor {
            builtin_registry: BuiltinManager::new(),
            path_resolver: PathResolver,
            // redirect_handler: RedirectHandler::new(),
            // signal_handler: SignalHandler::new(),
        }
    }

    pub fn exec_command(
        &mut self,
        cmd: &CommandNode,
        env: &mut Environment,
    ) -> ExecStatus {
        match cmd.kind {
            CommandKind::Builtin => {
                // if let Some(builtin) = self.builtin_registry.find(&cmd.name) {
                //     builtin.execute(&cmd.args, env).map_err(ExecError::Custom(
                //         format!("Builtin command '{}' failed", cmd.name)
                //     ))
                // } else {
                //     Err(ExecError::CommandNotFound(cmd.name.clone()))
                // }
                Err(ExecError::NotImplemented("Not implemented".to_string()))
            }
            CommandKind::External | CommandKind::Simple => {
                // Built-in command execution
                let builtin_manager = BuiltinManager::new();
                if builtin_manager.is_builtin(&cmd.name) {
                    return builtin_manager.execute(&cmd.name, &cmd.args, env);
                }

                let resolver = PathResolver;
                let path = match resolver.resolve(&cmd.name) {
                    Some(p) => p,
                    None => {
                        eprintln!("tiny-shell: command not found or failed");
                        return Ok(127) // The shell's standard "command not found" exit code
                        // return Err(ExecError::CommandNotFound(cmd.name.clone()));
                    }
                };

                // External command execution
                let mut command = Command::new(path);

                // command.args(&cmd.args);
                for arg in &cmd.args {
                    command.arg(arg);
                }
                // for (key, value) in env.all() {
                //     command.env(&key, &value);
                // }

                match command.status() {
                    Ok(status) => Ok(status.code().unwrap_or(1)),
                    Err(e) => Err(ExecError::Io(e)),
                }
            }
        }
    }
}

impl Executor for RecursiveExecutor {
    fn exec(&mut self, node: &AstNode, env: &mut Environment) -> ExecStatus {
        match node {
            AstNode::Command(cmd) => {
                self.exec_command(cmd, env)
            }
            AstNode::Redirect { node: inner, kind, file } => {
                RedirectHandler::handle_redirect(inner, kind, file, self, env)
            }
            AstNode::Pipeline(nodes) => {
                RedirectHandler::handle_pipeline(nodes, self, env)
            }
            AstNode::Sequence(seq) => {
                for node in seq {
                    self.exec(node, env)?;
                }
                Ok(0)
            }
            AstNode::And(left, right) => {
                if self.exec(left, env)? == 0 {
                    self.exec(right, env)
                } else {
                    Ok(1)
                }
            }
            AstNode::Or(left, right) => {
                if self.exec(left, env)? != 0 {
                    self.exec(right, env)
                } else {
                    Ok(0)
                }
            }
            AstNode::Subshell(_inner) => {
                Err(ExecError::NotImplemented("Not implemented".to_string()))
            }
            _ => Err(ExecError::NotImplemented("Not implemented".to_string())),
        }
    }
}

