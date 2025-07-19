use std::fs::File;
use std::process::{Command};
use std::os::unix::io::AsRawFd;
use super::super::executor::{ Executor, ExecStatus, ExecError };
use super::super::builtins::BuiltinManager;
use super::super::path_resolver::PathResolver;
use crate::ast::{AstNode, CommandNode, RedirectKind};
use crate::environment::Environment;

pub struct FlattenExecutor {
    stdin_stack: Vec<i32>,
    stdout_stack: Vec<i32>,
    // stderr_stack: Vec<i32>,
    prev_read_fd: Option<i32>, // The read end of the previous pipe
    child_pids: Vec<i32>, // The process id of each child in the pipeline
    in_pipeline: bool, // Whether or not in the pipeline
}

// pub struct DryRunExecutor;
// pub struct LoggingExecutor;

#[derive(Debug, Clone, PartialEq)]
enum ExecStep {
    RunCommand(CommandNode),
    BeginRedirect {
        kind: RedirectKind,
        file: String,
    },
    EndRedirect {
        kind: RedirectKind,
    },
    BeginPipeline,
    EndPipeline,
}

impl Executor for FlattenExecutor {
    fn exec(&mut self, node: &AstNode, env: &mut Environment) -> ExecStatus {
        let mut plan = Vec::new();
        self.flatten_ast(node, &mut plan);
        let mut pipeline_cmds = Vec::new();

        for step in &plan {
            match step {
                ExecStep::RunCommand(cmd) => {
                    if self.in_pipeline {
                        pipeline_cmds.push(cmd.clone());
                    } else {
                        self.run_command(cmd, env)?;
                    }
                }
                ExecStep::BeginRedirect { kind, file } => {
                    self.begin_redirect(kind, file)?;
                }
                ExecStep::EndRedirect { kind } => {
                    self.end_redirect(kind)?;
                }
                ExecStep::BeginPipeline => {
                    self.begin_pipeline()?;
                }
                ExecStep::EndPipeline => {
                    self.exec_pipeline(&pipeline_cmds, env)?;
                    pipeline_cmds.clear();
                }
            }
        }
        Ok(0)
    }
}

impl FlattenExecutor {
    pub fn new() -> Self {
        FlattenExecutor {
            stdin_stack: Vec::new(),
            stdout_stack: Vec::new(),
            // stderr_stack: Vec::new(),
            prev_read_fd: None,
            child_pids: Vec::new(),
            in_pipeline: false,
        }
    }

    fn flatten_ast(&self, node: &AstNode, plan: &mut Vec<ExecStep>) {
        match node {
            AstNode::Command(cmd) => {
                plan.push(ExecStep::RunCommand(cmd.clone()));
            }
            AstNode::Redirect { node: inner, kind, file } => {
                plan.push(ExecStep::BeginRedirect { kind: kind.clone(), file: file.clone() });
                self.flatten_ast(inner, plan);
                plan.push(ExecStep::EndRedirect { kind: kind.clone() });
            }
            AstNode::Pipeline(nodes) => {
                plan.push(ExecStep::BeginPipeline);
                for node in nodes {
                    self.flatten_ast(node, plan);
                }
                plan.push(ExecStep::EndPipeline);
            }
            AstNode::Sequence(seq) => {
                for node in seq {
                    self.flatten_ast(node, plan);
                }
            }
            AstNode::And(left, right) => {
                self.flatten_ast(left, plan);
                // TODO: ExecStep::And
                self.flatten_ast(right, plan);
            }
            AstNode::Or(left, right) => {
                self.flatten_ast(left, plan);
                // TODO: ExecStep::Or
                self.flatten_ast(right, plan);
            }
            AstNode::Subshell(inner) => {
                // TODO: ExecStep::BeginSubshell, ExecStep::EndSubshell
                self.flatten_ast(inner, plan);
            }
            AstNode::Compound(_) => {
                unimplemented!();
            }
        }
    }

    fn begin_redirect(&mut self, kind: &RedirectKind, file: &str) -> ExecStatus {
        use RedirectKind::*;
        match kind {
            In => {
                let f = File::open(file).map_err(ExecError::Io)?;
                let new_fd = f.as_raw_fd();

                // save (0: stdin)
                let saved = unsafe { libc::dup(0) };
                if saved < 0 {
                    return Err(ExecError::Io(std::io::Error::last_os_error()));
                }
                self.stdin_stack.push(saved);

                // Replacement
                if unsafe { libc::dup2(new_fd, 0) } < 0 {
                    return Err(ExecError::Io(std::io::Error::last_os_error()));
                }
            }
            Out => {
                let f = File::create(file).map_err(ExecError::Io)?;
                let new_fd = f.as_raw_fd();

                let saved = unsafe { libc::dup(1) };
                if saved < 0 {
                    return Err(ExecError::Io(std::io::Error::last_os_error()));
                }
                self.stdout_stack.push(saved);

                if unsafe { libc::dup2(new_fd, 1) } < 0 {
                    return Err(ExecError::Io(std::io::Error::last_os_error()));
                }
            }
            Append => {
                let f = std::fs::OpenOptions::new()
                    .write(true).append(true).create(true)
                    .open(file)
                    .map_err(ExecError::Io)?;
                let new_fd = f.as_raw_fd();

                let saved = unsafe { libc::dup(1) };
                if saved < 0 {
                    return Err(ExecError::Io(std::io::Error::last_os_error()));
                }
                self.stdout_stack.push(saved);

                if unsafe { libc::dup2(new_fd, 1) } < 0 {
                    return Err(ExecError::Io(std::io::Error::last_os_error()));
                }
            }
        }
        Ok(0)
    }

    fn end_redirect(&mut self, kind: &RedirectKind) -> ExecStatus {
        use RedirectKind::*;
        match kind {
            In => {
                if let Some(saved) = self.stdin_stack.pop() {
                    if unsafe { libc::dup2(saved, 0) } < 0 {
                        return Err(ExecError::Io(std::io::Error::last_os_error()));
                    }
                    unsafe { libc::close(saved); }
                }
            }
            Out | Append => {
                if let Some(saved) = self.stdout_stack.pop() {
                    if unsafe { libc::dup2(saved, 1) } < 0 {
                        return Err(ExecError::Io(std::io::Error::last_os_error()));
                    }
                    unsafe { libc::close(saved); }
                }
            }
        }
        Ok(0)
    }

    fn begin_pipeline(&mut self) -> ExecStatus {
        self.prev_read_fd = None;
        self.child_pids.clear();
        self.in_pipeline = true;
        Ok(0)
    }

    fn exec_pipeline(&mut self, cmds: &[CommandNode], env: &mut Environment) -> ExecStatus {
        if cmds.len() < 2 {
            return Err(ExecError::Custom("Pipeline must have at least two commands".into()));
        }

        let mut prev_read_fd: Option<i32> = None;
        let mut child_pids = Vec::new();

        for (i, cmd) in cmds.iter().enumerate() {
            let is_last = i == cmds.len() - 1;
            let mut pipefds = [0; 2];

            if !is_last {
                if unsafe { libc::pipe(pipefds.as_mut_ptr()) } == -1 {
                    return Err(ExecError::Io(std::io::Error::last_os_error()));
                }
            }

            let pid = unsafe { libc::fork() };
            if pid < 0 {
                return Err(ExecError::Io(std::io::Error::last_os_error()));
            }

            if pid == 0 {
                // Child process
                if let Some(read_fd) = prev_read_fd {
                    unsafe {
                        libc::dup2(read_fd, 0); // Redirect the read end of the previous pipe to stdin
                        libc::close(read_fd);
                    }
                }
                if !is_last {
                    unsafe {
                        libc::close(pipefds[0]); // the read end is not needed
                        libc::dup2(pipefds[1], 1); // redirect the write end to stdout
                        libc::close(pipefds[1]);
                    }
                }
                std::process::exit(self.run_command(cmd, env).unwrap_or(1));
            } else {
                // Parent process
                if let Some(read_fd) = prev_read_fd {
                    unsafe { libc::close(read_fd); }
                }
                if !is_last {
                    unsafe { libc::close(pipefds[1]); }
                    prev_read_fd = Some(pipefds[0]);
                } else {
                    prev_read_fd = None;
                }
                child_pids.push(pid);
            }
        }
        // Wait for all child processes
        for pid in child_pids {
            let mut status_code = 0;
            unsafe { libc::waitpid(pid, &mut status_code, 0); }
        }
        Ok(0)
    }

    fn run_command(&mut self, cmd: &CommandNode, env: &mut Environment) -> ExecStatus {
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
        // command.args(&cmd.args);
        for arg in &cmd.args {
            command.arg(arg);
        }

        // for (k, v) in &cmd.assignments {
        //     command.env(k, v);
        // }
        // for (k, v) in &env.vars {
        //     command.env(k, v);
        // }

        match command.status() {
            Ok(status) => Ok(status.code().unwrap_or(1)),
            Err(e) => Err(ExecError::Io(e)),
        }
    }
}

