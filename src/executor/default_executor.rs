use std::fs::File;
use std::process::{Command, Stdio};
use nix;
use crate::ast::{AstNode, CommandNode, RedirectKind};
use crate::environment::Environment;
use crate::executor::{ Executor, ExecStatus };
use super::executor::ExecError;
use super::builtins::BuiltinManager;
use super::path_resolver::PathResolver;

pub struct DefaultExecutor;
// pub struct DryRunExecutor;
// pub struct LoggingExecutor;

impl Executor for DefaultExecutor {
    fn exec(&mut self, node: &AstNode, env: &mut Environment) -> ExecStatus {
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
    fn exec_command(&mut self, cmd: &CommandNode, env: &mut Environment) -> ExecStatus {
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

    fn exec_redirect(
        &mut self,
        node: &AstNode,
        kind: &RedirectKind,
        file: &String,
        env: &mut Environment,
    ) -> ExecStatus {
        use std::fs::File;
        use std::process::Stdio;

        // 1. 全リダイレクトをVecでフラットに集める
        let mut redirects = vec![RedirectInfo { kind, file }];
        let (cmd_node, mut more_redirects) = flatten_redirects(node, Vec::new());
        redirects.append(&mut more_redirects);

        // 2. ファイルハンドルを種別ごとにセット
        let mut stdin_file = None;
        let mut stdout_file = None;
        let mut stderr_file = None;

        for r in redirects {
            match r.kind {
                RedirectKind::In => stdin_file = Some(File::open(r.file).map_err(ExecError::Io)?),
                RedirectKind::Out => stdout_file = Some(File::create(r.file).map_err(ExecError::Io)?),
                // RedirectKind::Append => {
                //     stdout_file = Some(
                //         std::fs::OpenOptions::new()
                //             .write(true)
                //             .append(true)
                //             .create(true)
                //             .open(r.file)
                //             .map_err(ExecError::Io)?
                //     );
                // }
                // RedirectKind::Err => stderr_file = Some(File::create(r.file).map_err(ExecError::Io)?),
            }
        }

        self.exec_with_stdio(cmd_node, env, stdout_file, stdin_file, stderr_file)
    }

    fn exec_pipeline(
        &mut self,
        node: &AstNode,
        env: &mut Environment,
    ) -> ExecStatus {
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

    fn exec_subshell(&mut self, sub: &AstNode, env: &mut Environment) -> ExecStatus {
        // todo: fork/exec or Command::new("sh -c ...")
        // Run sub shell in a new environment
        println!("exec_subshell");
        self.exec(sub, &mut env.clone())
    }

    fn exec_with_stdio(
        &mut self,
        node: &AstNode,
        env: &mut Environment,
        stdout: Option<File>,
        stdin: Option<File>,
        stderr: Option<File>,
    ) -> ExecStatus {
        match node {
            AstNode::Command(cmd) => {
                let resolver = PathResolver;
                let path = match resolver.resolve(&cmd.name) {
                    Some(p) => p,
                    None => {
                        eprintln!("tiny-shell: command not found or failed");
                        return Ok(127);
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

                // 標準入出力の差し替え
                if let Some(f) = stdin {
                    command.stdin(Stdio::from(f));
                } else {
                    command.stdin(Stdio::inherit());
                }
                if let Some(f) = stdout {
                    command.stdout(Stdio::from(f));
                } else {
                    command.stdout(Stdio::inherit());
                }
                if let Some(f) = stderr {
                    command.stderr(Stdio::from(f));
                } else {
                    command.stderr(Stdio::inherit());
                }

                let status = command.status().map_err(ExecError::Io)?;
                Ok(status.code().unwrap_or(1))
            }
            AstNode::Pipeline(_, _) => {
                // パイプラインはexec_pipelineで処理
                self.exec_pipeline_with_redirect(node, env, stdout, stdin, stderr)
            }
            _ => self.exec(node, env)
        }
    }

    fn exec_pipeline_with_redirect(
        &mut self,
        node: &AstNode,
        env: &mut Environment,
        stdout: Option<File>,
        stdin: Option<File>,
        stderr: Option<File>,
    ) -> ExecStatus {
        use std::process::Stdio;

        let mut cmds = Vec::new();
        flatten_pipeline(node, &mut cmds);

        let n = cmds.len();
        let mut children = Vec::with_capacity(n);

        let mut prev_stdout = None;

        for (i, &cmd_node) in cmds.iter().enumerate() {
            let mut command = self.prepare_command(cmd_node, env)?;

            // 入力リダイレクトはパイプラインの最初だけ
            if i == 0 {
                if let Some(ref f) = stdin {
                    command.stdin(Stdio::from(f.try_clone().map_err(ExecError::Io)?));
                }
            } else if let Some(stdin) = prev_stdout.take() {
                command.stdin(stdin);
            }

            // 出力リダイレクトはパイプラインの最後だけ
            if i == n - 1 {
                if let Some(ref f) = stdout {
                    command.stdout(Stdio::from(f.try_clone().map_err(ExecError::Io)?));
                }
            } else {
                let (pipe_read, pipe_write) = nix::unistd::pipe()
                    .map_err(|e| ExecError::Io(std::io::Error::from_raw_os_error(e as i32)))?;
                let write = Stdio::from(pipe_write);
                command.stdout(write);
                prev_stdout = Some(Stdio::from(pipe_read));
            }

            // エラーストリーム（必要なら最後のコマンドだけでOK）
            if i == n - 1 {
                if let Some(ref f) = stderr {
                    command.stderr(Stdio::from(f.try_clone().map_err(ExecError::Io)?));
                }
            }

            let child = command.spawn().map_err(ExecError::Io)?;
            children.push(child);
        }

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
            let resolver = PathResolver;
            let path = match resolver.resolve(&cmd.name) {
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

fn flatten_pipeline<'a>(node: &'a AstNode, result: &mut Vec<&'a AstNode>) {
    match node {
        AstNode::Pipeline(left, right) => {
            flatten_pipeline(left, result);
            flatten_pipeline(right, result);
        }
        _ => result.push(node),
    }
}

#[derive(Debug)]
pub struct RedirectInfo<'a> {
    pub kind: &'a RedirectKind,
    pub file: &'a String,
}

fn flatten_redirects<'a>(mut node: &'a AstNode, mut redirects: Vec<RedirectInfo<'a>>) -> (&'a AstNode, Vec<RedirectInfo<'a>>) {
    loop {
        match node {
            AstNode::Redirect { node: inner, kind, file } => {
                redirects.push(RedirectInfo { kind, file });
                node = inner;
            }
            _ => return (node, redirects),
        }
    }
}

