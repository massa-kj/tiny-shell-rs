pub mod default;

use crate::ast::{AstNode, CommandNode};
use crate::environment::Environment;
// use crate::redirect::{RedirectKind};
use crate::error::ExecError;

pub type ExecResult = Result<i32, ExecError>;

pub trait Executor {
    fn exec(&mut self, node: &AstNode, env: &mut Environment) -> ExecResult;
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
        fn exec(&mut self, node: &AstNode, env: &mut Environment) -> ExecResult {
            match node {
                AstNode::Command(cmd) => {
                    self.log.push(format!("command: {} {:?}", cmd.name, cmd.args));
                    Ok(0)
                }
                AstNode::Pipeline(left, right) => {
                    self.log.push("pipeline".to_string());
                    self.exec(left, env)?;
                    self.exec(right, env)
                }
                AstNode::Redirect { node, kind, file } => {
                    self.log.push(format!("redirect: {:?} {}", kind, file));
                    self.exec(node, env)
                }
                AstNode::Subshell(sub) => {
                    self.log.push("subshell".to_string());
                    self.exec(sub, &mut env.clone()) // サブシェルはcloneで
                }
                AstNode::Sequence(lhs, rhs) => {
                    self.log.push("sequence".to_string());
                    self.exec(lhs, env)?;
                    self.exec(rhs, env)
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
        let ast = AstNode::Pipeline(
            Box::new(dummy_cmd("ls", &[])),
            Box::new(dummy_cmd("wc", &[])),
        );
        let mut env = Environment::new();
        let mut exec = TestExecutor::new();
        let result = exec.exec(&ast, &mut env);
        assert!(matches!(result, Ok(0)));
        assert_eq!(exec.log, vec!["pipeline", "command: ls []", "command: wc []"]);
    }
}

