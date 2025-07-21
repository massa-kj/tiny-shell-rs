use crate::ast::{AstNode};
use crate::executor::{Executor, ExecStatus, ExecOutcome, ExecError};
use crate::environment::Environment;

pub struct MockExecutor {
    pub last_cmd: Option<String>,
    pub last_args: Vec<String>,
}

impl MockExecutor {
    pub fn new() -> Self {
        Self { last_cmd: None, last_args: Vec::new() }
    }
}

impl Executor for MockExecutor {
    fn exec(&mut self, node: &AstNode, _env: &mut Environment) -> ExecStatus {
        if let AstNode::Command(cmd) = node {
            self.last_cmd = Some(cmd.name.clone());
            self.last_args = cmd.args.clone();
            Ok(ExecOutcome::Code(0))
        } else {
            Err(ExecError::Custom("Mock: Not CommandNode".into()))
        }
    }
}

