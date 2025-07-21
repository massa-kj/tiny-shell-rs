use std::collections::HashMap;
use crate::executor::{ ExecStatus, ExecError };
use crate::environment::Environment;
use crate::executor::builtin::commands::{
    HelpCommand,
    CdCommand,
    ExitCommand,
    ExportCommand,
};

pub trait BuiltinCommand {
    fn name(&self) -> &'static str;
    fn run(&self, args: &[String], env: &mut Environment) -> ExecStatus;
}

pub struct BuiltinManager {
    commands: HashMap<String, Box<dyn BuiltinCommand>>,
}

impl BuiltinManager {
    pub fn new() -> Self {
        let mut mgr = BuiltinManager {
            commands: HashMap::new(),
        };
        mgr.register(Box::new(HelpCommand {}));
        mgr.register(Box::new(CdCommand {}));
        mgr.register(Box::new(ExitCommand {}));
        mgr.register(Box::new(ExportCommand {}));
        mgr
    }

    pub fn register(&mut self, cmd: Box<dyn BuiltinCommand>) {
        self.commands.insert(cmd.name().to_string(), cmd);
    }

    pub fn is_builtin(&self, name: &str) -> bool {
        self.commands.contains_key(name)
    }

    pub fn execute(
        &self,
        name: &str,
        args: &[String],
        env: &mut Environment,
    ) -> ExecStatus {
        if let Some(cmd) = self.commands.get(name) {
            cmd.run(args, env)
        } else {
            Err(ExecError::NoSuchBuiltin(name.to_string()))
        }
    }
}

