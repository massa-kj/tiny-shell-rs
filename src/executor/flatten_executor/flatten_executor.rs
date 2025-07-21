use std::fs::File;
use std::process::{Command};
use std::os::unix::io::AsRawFd;
use crate::executor::{ Executor, ExecStatus, ExecOutcome, ExecError };
use crate::executor::builtins::BuiltinManager;
use crate::executor::path_resolver::PathResolver;
use crate::executor::pipeline::PipelineHandler;
use crate::ast::{AstNode, CommandNode, RedirectKind};
use crate::environment::Environment;

pub struct FlattenExecutor {
    stdin_stack: Vec<i32>,
    stdout_stack: Vec<i32>,
    // stderr_stack: Vec<i32>,
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
                    self.end_pipeline(&pipeline_cmds, env)?;
                    pipeline_cmds.clear();
                }
            }
        }
        Ok(ExecOutcome::Code(0))
    }
}

impl FlattenExecutor {
    pub fn new() -> Self {
        FlattenExecutor {
            stdin_stack: Vec::new(),
            stdout_stack: Vec::new(),
            // stderr_stack: Vec::new(),
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
        Ok(ExecOutcome::Code(0))
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
        Ok(ExecOutcome::Code(0))
    }

    fn begin_pipeline(&mut self) -> ExecStatus {
        self.in_pipeline = true;
        Ok(ExecOutcome::Code(0))
    }

    fn end_pipeline(&mut self, cmds: &[CommandNode], env: &mut Environment) -> ExecStatus {
        PipelineHandler::exec_pipeline_generic(cmds, |cmd| self.run_command(cmd, env))?;
        self.in_pipeline = false;
        Ok(ExecOutcome::Code(0))
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
                return Ok(ExecOutcome::Code(127)) // The shell's standard "command not found" exit code
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
            Ok(status) => Ok(ExecOutcome::Code(status.code().unwrap_or(1))),
            Err(e) => Err(ExecError::Io(e)),
        }
    }
}

