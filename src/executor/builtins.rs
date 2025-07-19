use std::collections::HashMap;

use crate::environment::Environment;
use crate::executor::{ ExecStatus };
use super::executor::ExecError;

pub trait BuiltinCommand {
    fn name(&self) -> &'static str;
    fn run(&self, args: &[String], env: &mut Environment) -> i32;
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
            Ok(cmd.run(args, env))
        } else {
            Err(ExecError::NoSuchBuiltin(name.to_string()))
        }
    }
}

pub struct HelpCommand;

impl BuiltinCommand for HelpCommand {
    fn name(&self) -> &'static str {
        "help"
    }
    fn run(&self, _args: &[String], _env: &mut Environment) -> i32 {
        println!("Available built-in commands:");
        println!("  cd [DIR]   : Change directory");
        println!("  exit       : Exit shell");
        println!("  help       : Show this help");
        0
    }
}

pub struct CdCommand;

impl BuiltinCommand for CdCommand {
    fn name(&self) -> &'static str {
        "cd"
    }
    fn run(&self, args: &[String], _env: &mut Environment) -> i32 {
        let target = args.get(0).map(|s| s.as_str()).unwrap_or("/");
        match std::env::set_current_dir(target) {
            Ok(_) => 0,
            Err(e) => {
                eprintln!("cd: {}: {}", target, e);
                1
            }
        }
    }
}

pub struct ExitCommand;

impl BuiltinCommand for ExitCommand {
    fn name(&self) -> &'static str {
        "exit"
    }
    fn run(&self, args: &[String], _env: &mut Environment) -> i32 {
        let code = args.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
        std::process::exit(code);
    }
}

pub struct ExportCommand;

impl BuiltinCommand for ExportCommand {
    fn name(&self) -> &'static str {
        "export"
    }
    fn run(&self, _args: &[String], _env: &mut Environment) -> i32 {
        // for arg in args {
        //     if let Some((k, v)) = arg.split_once('=') {
        //         env.envs.insert(k.to_string(), v.to_string());
        //     }
        // }
        0
    }
}

