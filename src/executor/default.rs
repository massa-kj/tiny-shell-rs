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

impl DefaultExecutor {
    fn exec_command(&mut self, cmd: &CommandNode, env: &mut Environment) -> ExecResult {
        // Built-in command execution
        let builtin_manager = BuiltinManager::new();
        if builtin_manager.is_builtin(&cmd.name) {
            // return builtin_manager.execute(&cmd.name, &cmd.args, env);
            return Ok(0)
        }

        let path = match resolve_command_path(&cmd.name) {
            Some(p) => p,
            None => {
                eprintln!("tiny-shell: command not found or failed");
                return Ok(127) // The shell's standard "command not found" exit code
            }
        };

        // External command execution
        let mut command = Command::new(path);
        command.args(&cmd.args);

        // 環境変数
        // for (k, v) in &cmd.assignments {
        //     command.env(k, v);
        // }
        // shell全体の環境変数
        // for (k, v) in &env.vars {
        //     command.env(k, v);
        // }

        // 標準入出力リダイレクト、背景プロセス等がある場合はここでStdio制御
        // command.stdin(Stdio::inherit()).stdout(...) など

        let status = command
            .status()
            .map_err(ExecError::Io)?;

        Ok(status.code().unwrap_or(1))
    }

    fn exec_pipeline(
        &mut self,
        node: &AstNode,
        env: &mut Environment,
    ) -> ExecResult {
        use std::process::{Command, Stdio};
        use std::os::unix::io::FromRawFd;

        // 1. 全コマンドノードをVecでフラットに集める
        let mut cmds = Vec::new();
        flatten_pipeline(node, &mut cmds);

        let n = cmds.len();
        let mut children = Vec::with_capacity(n);

        let mut prev_stdout = None;

        for (i, &cmd_node) in cmds.iter().enumerate() {
            let mut command = self.prepare_command(cmd_node, env)?;
            // 最初以外はstdinをパイプでつなぐ
            if let Some(stdin) = prev_stdout.take() {
                command.stdin(stdin);
            }
            // 最後以外はstdoutをパイプでつなぐ
            if i != n - 1 {
                let (pipe_read, pipe_write) = nix::unistd::pipe()
                    .map_err(|e| ExecError::Io(std::io::Error::from_raw_os_error(e as i32)))?;

                let write = Stdio::from(pipe_write);
                command.stdout(write);

                prev_stdout = Some(Stdio::from(pipe_read));
            }
            // spawn
            let child = command.spawn().map_err(ExecError::Io)?;
            children.push(child);
        }

        // すべての子プロセスをwait（実用ではエラー処理やパイプのクローズも忘れずに）
        let mut status = 0;
        for mut child in children {
            status = child.wait().map_err(ExecError::Io)?.code().unwrap_or(1);
        }
        Ok(status)
    }

    fn prepare_command(
        &mut self,
        node: &AstNode,
        env: &mut Environment,
    ) -> Result<Command, ExecError> {
        // AstNode::CommandからCommand構築
        if let AstNode::Command(cmd) = node {
            let path = match resolve_command_path(&cmd.name) {
                Some(p) => p,
                None => {
                    return Err(ExecError::Custom("Not found command".to_string()));
                }
            };
            let mut command = Command::new(path);
            for arg in &cmd.args {
                command.arg(arg);
            }
            // for (k, v) in &cmd.assignments {
            //     command.env(k, v);
            // }
            // for (k, v) in &env.vars {
            //     command.env(k, v);
            // }
            Ok(command)
        } else {
            Err(ExecError::Custom("Not a command node".to_string()))
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

fn flatten_pipeline<'a>(node: &'a AstNode, result: &mut Vec<&'a AstNode>) {
    match node {
        AstNode::Pipeline(left, right) => {
            flatten_pipeline(left, result);
            flatten_pipeline(right, result);
        }
        _ => result.push(node),
    }
}

