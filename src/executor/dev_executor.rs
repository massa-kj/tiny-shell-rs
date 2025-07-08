use crate::ast::AstNode;
use crate::environment::Environment;
use crate::executor::{
    Executor,
    ExecStatus, ExecError,
    builtins::BuiltinManager,
    path_resolver::PathResolver,
    redirect::RedirectHandler,
    // signal::SignalHandler,
};

pub struct DefaultExecutor {
    pub builtin_registry: BuiltinManager,
    pub path_resolver: PathResolver,
    // pub redirect_handler: RedirectHandler,
    // pub signal_handler: SignalHandler,
}

impl DefaultExecutor {
    pub fn new() -> Self {
        DefaultExecutor {
            builtin_registry: BuiltinManager::new(),
            path_resolver: PathResolver,
            // redirect_handler: RedirectHandler::new(),
            // signal_handler: SignalHandler::new(),
        }
    }

    fn exec_command(&mut self, cmd: &crate::ast::CommandNode, env: &mut Environment) -> ExecStatus {
        Err(ExecError::NotImplemented("Not implemented".to_string()))
    }
}

impl Executor for DefaultExecutor {
    fn exec(&mut self, node: &AstNode, env: &mut Environment) -> ExecStatus {
        match node {
            AstNode::Command(cmd) => {
                self.exec_command(cmd, env)
            }
            AstNode::Redirect { node: inner, kind, file } => {
                RedirectHandler::handle_redirect(inner, kind, file, self, env)
            }
            AstNode::Pipeline(left, right) => {
                RedirectHandler::handle_pipeline(left, right, self, env)
            }
            AstNode::Sequence(left, right) => {
                self.exec(left, env)?;
                self.exec(right, env)
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
            AstNode::Subshell(inner) => {
                Err(ExecError::NotImplemented("Not implemented".to_string()))
            }
            _ => Err(ExecError::NotImplemented("Not implemented".to_string())),
        }
    }
}

