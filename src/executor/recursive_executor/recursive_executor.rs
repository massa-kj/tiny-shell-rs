use std::process::Command;
use super::redirect::RedirectHandler;
use crate::executor::{ Executor, ExecStatus, ExecOutcome, ExecError };
use crate::executor::builtins::BuiltinManager;
use crate::executor::path_resolver::PathResolver;
use crate::executor::pipeline::PipelineHandler;
use crate::ast::{AstNode, CommandNode, CommandKind};
use crate::environment::Environment;

pub struct RecursiveExecutor<'a> {
    builtin_manager: &'a BuiltinManager,
    // pub path_resolver: PathResolver,
    // pub redirect_handler: RedirectHandler,
    // pub signal_handler: SignalHandler,
}

impl<'a> Executor for RecursiveExecutor<'a> {
    fn exec(&mut self, node: &AstNode, env: &mut Environment) -> ExecStatus {
        match node {
            AstNode::Command(cmd) => {
                self.exec_command(cmd, env)
            }
            AstNode::Redirect { node: inner, kind, file } => {
                RedirectHandler::handle_redirect(inner, kind, file, self, env)
            }
            AstNode::Pipeline(nodes) => {
                PipelineHandler::exec_pipeline_generic(nodes, |node| self.exec(node, env))
            }
            AstNode::Sequence(seq) => {
                for node in seq {
                    self.exec(node, env)?;
                }
                Ok(ExecOutcome::Code(0))
            }
            AstNode::And(left, right) => {
                if self.exec(left, env)? == ExecOutcome::Code(0) {
                    self.exec(right, env)
                } else {
                    Ok(ExecOutcome::Code(1))
                }
            }
            AstNode::Or(left, right) => {
                if self.exec(left, env)? != ExecOutcome::Code(0) {
                    self.exec(right, env)
                } else {
                    Ok(ExecOutcome::Code(0))
                }
            }
            AstNode::Subshell(_inner) => {
                Err(ExecError::NotImplemented("Not implemented".to_string()))
            }
            _ => Err(ExecError::NotImplemented("Not implemented".to_string())),
        }
    }
}

impl<'a> RecursiveExecutor<'a> {
    pub fn new(builtin_manager: &'a BuiltinManager) -> Self {
        RecursiveExecutor {
            builtin_manager,
            // path_resolver: PathResolver,
            // redirect_handler: RedirectHandler::new(),
            // signal_handler: SignalHandler::new(),
        }
    }

    fn exec_command(
        &mut self,
        cmd: &CommandNode,
        env: &mut Environment,
    ) -> ExecStatus {
        match cmd.kind {
            CommandKind::Builtin => {
                // if let Some(builtin) = self.builtin_manager.find(&cmd.name) {
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
                if self.builtin_manager.is_builtin(&cmd.name) {
                    return self.builtin_manager.execute(&cmd.name, &cmd.args, env);
                }

                let resolver = PathResolver;
                let path = match resolver.resolve(&cmd.name) {
                    Some(p) => p,
                    None => {
                        eprintln!("tiny-shell: command not found or failed");
                        return Ok(ExecOutcome::Code(127)) // The shell's standard "command not found" exit code
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
                    Ok(status) => Ok(ExecOutcome::Code(status.code().unwrap_or(1))),
                    Err(e) => Err(ExecError::Io(e)),
                }
            }
        }
    }
}

