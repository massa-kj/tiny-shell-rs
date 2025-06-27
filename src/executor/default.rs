use std::env;
use std::fs::File;
use std::fs::{self};
use std::path::Path;
use std::process::{Command, Stdio};
use nix;
use crate::ast::{AstNode, CommandNode, RedirectKind};
use crate::environment::Environment;
// use crate::redirect::{RedirectKind};
// use crate::error::ExecError;
use crate::executor::{ Executor, ExecResult };
use crate::builtins::BuiltinManager;
use crate::error::ExecError;

pub struct DefaultExecutor;
// pub struct DryRunExecutor;
// pub struct LoggingExecutor;

impl Executor for DefaultExecutor {
    fn exec(&mut self, node: &AstNode, env: &mut Environment) -> ExecResult {
        match node {
            AstNode::Command(cmd) => self.exec_command(cmd, env),
            AstNode::Pipeline(_, _) => self.exec_pipeline(node, env),
            AstNode::Redirect { node, kind, file } => self.exec_redirect(node, kind, file, env),
            AstNode::Subshell(sub) => self.exec_subshell(sub, env),
            AstNode::Sequence(lhs, rhs) => {
                self.exec(lhs, env)?;
                self.exec(rhs, env)
            }
            AstNode::And(lhs, rhs) => {
                if self.exec(lhs, env)? == 0 {
                    self.exec(rhs, env)
                } else {
                    Ok(1)
                }
            }
            AstNode::Or(lhs, rhs) => {
                if self.exec(lhs, env)? != 0 {
                    self.exec(rhs, env)
                } else {
                    Ok(0)
                }
            }
            AstNode::Compound(_) => unimplemented!(), // 今後拡張
        }
    }
}

