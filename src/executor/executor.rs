use std::{io, fmt};
use crate::ast::{AstNode};
use crate::environment::Environment;

pub type ExecStatus = Result<i32, ExecError>;

#[derive(Debug)]
pub enum ExecError {
    CommandNotFound(String),
    Io(io::Error),
    PermissionDenied(String),
    InvalidArgument(String),
    PipelineError(String),
    RedirectError(String),
    SubshellError(String),
    NoSuchBuiltin(String),
    NotImplemented(String),
    Custom(String),
}
impl fmt::Display for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecError::CommandNotFound(cmd) => write!(f, "Command not found: {}", cmd),
            ExecError::Io(e) => write!(f, "IO error: {}", e),
            ExecError::PermissionDenied(cmd) => write!(f, "Permission denied: {}", cmd),
            ExecError::InvalidArgument(arg) => write!(f, "Invalid argument: {}", arg),
            ExecError::PipelineError(msg) => write!(f, "Pipeline error: {}", msg),
            ExecError::RedirectError(msg) => write!(f, "Redirect error: {}", msg),
            ExecError::SubshellError(msg) => write!(f, "Subshell error: {}", msg),
            ExecError::NoSuchBuiltin(name) => write!(f, "No such builtin command: {}", name),
            ExecError::NotImplemented(feature) => write!(f, "Feature not implemented: {}", feature),
            ExecError::Custom(msg) => write!(f, "Execution error: {}", msg),
        }
    }
}

pub trait Executor {
    fn exec(&mut self, node: &AstNode, env: &mut Environment) -> ExecStatus;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{AstNode, CommandNode, CommandKind, RedirectKind};
    use crate::environment::Environment;

    struct TestExecutor {
        // ログや記録用フィールドなど
        pub log: Vec<String>,
    }

    impl Executor for TestExecutor {
        fn exec(&mut self, node: &AstNode, env: &mut Environment) -> ExecStatus {
            match node {
                AstNode::Command(cmd) => {
                    self.log.push(format!("command: {} {:?}", cmd.name, cmd.args));
                    Ok(0)
                }
                AstNode::Pipeline(nodes) => {
                    self.log.push("pipeline".to_string());
                    for node in nodes {
                        self.exec(node, env)?;
                    }
                    Ok(0)
                }
                AstNode::Redirect { node, kind, file } => {
                    self.log.push(format!("redirect: {:?} {}", kind, file));
                    self.exec(node, env)
                }
                AstNode::Subshell(sub) => {
                    self.log.push("subshell".to_string());
                    self.exec(sub, &mut env.clone()) // サブシェルはcloneで
                }
                AstNode::Sequence(seq) => {
                    self.log.push("sequence".to_string());
                    for node in seq {
                        self.exec(node, env)?;
                    }
                    Ok(0)
                }
                AstNode::And(lhs, rhs) => {
                    self.log.push("and".to_string());
                    if self.exec(lhs, env)? == 0 {
                        self.exec(rhs, env)
                    } else {
                        Ok(1)
                    }
                }
                AstNode::Or(lhs, rhs) => {
                    self.log.push("or".to_string());
                    if self.exec(lhs, env)? != 0 {
                        self.exec(rhs, env)
                    } else {
                        Ok(0)
                    }
                }
                AstNode::Compound(_) => {
                    self.log.push("compound".to_string());
                    Ok(0)
                }
            }
        }
    }

    impl TestExecutor {
        fn new() -> Self {
            Self { log: vec![] }
        }
    }

    fn dummy_cmd(name: &str, args: &[&str]) -> AstNode {
        AstNode::Command(CommandNode {
            name: name.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            kind: CommandKind::Simple,
            // assignments: vec![],
        })
    }

    #[test]
    fn test_simple_command() {
        let ast = dummy_cmd("echo", &["hello"]);
        let mut env = Environment::new();
        let mut exec = TestExecutor::new();
        let result = exec.exec(&ast, &mut env);
        assert!(matches!(result, Ok(0)));
        assert_eq!(exec.log, vec!["command: echo [\"hello\"]"]);
    }

    #[test]
    fn test_pipeline() {
        let ast = AstNode::Pipeline(vec![
            dummy_cmd("ls", &[]),
            dummy_cmd("wc", &[]),
        ]);
        let mut env = Environment::new();
        let mut exec = TestExecutor::new();
        let result = exec.exec(&ast, &mut env);
        assert!(matches!(result, Ok(0)));
        assert_eq!(exec.log, vec!["pipeline", "command: ls []", "command: wc []"]);
    }

    #[test]
    fn test_redirect() {
        let ast = AstNode::Redirect {
            node: Box::new(dummy_cmd("ls", &[])),
            kind: RedirectKind::Out,
            file: "out.txt".to_string(),
        };
        let mut env = Environment::new();
        let mut exec = TestExecutor::new();
        let result = exec.exec(&ast, &mut env);
        assert!(matches!(result, Ok(0)));
        assert_eq!(exec.log, vec!["redirect: Out out.txt", "command: ls []"]);
    }

    #[test]
    fn test_subshell() {
        let ast = AstNode::Subshell(Box::new(dummy_cmd("ls", &[])));
        let mut env = Environment::new();
        let mut exec = TestExecutor::new();
        let result = exec.exec(&ast, &mut env);
        assert!(matches!(result, Ok(0)));
        assert_eq!(exec.log, vec!["subshell", "command: ls []"]);
    }

    #[test]
    fn test_complex_pipeline_with_redirect_and_subshell() {
        // (ls | grep foo) > out.txt
        let ast = AstNode::Redirect {
            node: Box::new(AstNode::Subshell(Box::new(AstNode::Pipeline(vec![
                dummy_cmd("ls", &[]),
                dummy_cmd("grep", &["foo"]),
            ])))),
            kind: RedirectKind::Out,
            file: "out.txt".to_string(),
        };
        let mut env = Environment::new();
        let mut exec = TestExecutor::new();
        let result = exec.exec(&ast, &mut env);
        assert!(matches!(result, Ok(0)));
        assert_eq!(
            exec.log,
            vec![
                "redirect: Out out.txt",
                "subshell",
                "pipeline",
                "command: ls []",
                "command: grep [\"foo\"]"
            ]
        );
    }

    #[test]
    fn test_sequence_and_and_or() {
        // echo hi && false || echo fallback; echo done
        let ast = AstNode::Sequence(vec![
            AstNode::Or(
                Box::new(AstNode::And(
                    Box::new(dummy_cmd("echo", &["hi"])),
                    Box::new(dummy_cmd("false", &[])),
                )),
                Box::new(dummy_cmd("echo", &["fallback"])),
            ),
            dummy_cmd("echo", &["done"]),
        ]);
        let mut env = Environment::new();
        let mut exec = TestExecutor::new();
        let result = exec.exec(&ast, &mut env);
        assert!(matches!(result, Ok(0)));
        assert_eq!(
            exec.log,
            vec![
                "sequence", "or", "and",
                "command: echo [\"hi\"]",
                "command: false []",
                "command: echo [\"fallback\"]",
                "command: echo [\"done\"]"
            ]
        );
    }
}

