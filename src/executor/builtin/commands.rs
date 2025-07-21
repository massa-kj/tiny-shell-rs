use std::cell::RefCell;
use std::rc::Rc;
use crate::history::HistoryManager;
use crate::executor::{ ExecStatus, ExecOutcome, ExecError };
use crate::environment::Environment;
use crate::executor::builtin::manager::BuiltinCommand;

pub struct HelpCommand;

impl BuiltinCommand for HelpCommand {
    fn name(&self) -> &'static str {
        "help"
    }
    fn run(&self, _args: &[String], _env: &mut Environment) -> ExecStatus {
        println!("Available built-in commands:");
        println!("  cd [DIR]   : Change directory");
        println!("  exit       : Exit shell");
        println!("  help       : Show this help");
        Ok(ExecOutcome::Code(0))
    }
}

pub struct CdCommand;

impl BuiltinCommand for CdCommand {
    fn name(&self) -> &'static str {
        "cd"
    }
    fn run(&self, args: &[String], _env: &mut Environment) -> ExecStatus {
        let target = args.get(0).map(|s| s.as_str()).unwrap_or("/");
        match std::env::set_current_dir(target) {
            Ok(_) => Ok(ExecOutcome::Code(0)),
            Err(e) => {
                eprintln!("cd: {}: {}", target, e);
                Ok(ExecOutcome::Code(1))
            }
        }
    }
}

pub struct ExitCommand;

impl BuiltinCommand for ExitCommand {
    fn name(&self) -> &'static str {
        "exit"
    }
    fn run(&self, args: &[String], _env: &mut Environment) -> ExecStatus {
        let code = args.get(0)
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0);
        Ok(ExecOutcome::Exit(code))
    }
}

pub struct ExportCommand;

impl BuiltinCommand for ExportCommand {
    fn name(&self) -> &'static str {
        "export"
    }
    fn run(&self, _args: &[String], _env: &mut Environment) -> ExecStatus {
        // for arg in args {
        //     if let Some((k, v)) = arg.split_once('=') {
        //         env.envs.insert(k.to_string(), v.to_string());
        //     }
        // }
        Ok(ExecOutcome::Code(0))
    }
}

pub struct HistoryCommand {
    pub history: Rc<RefCell<HistoryManager>>,
}

impl BuiltinCommand for HistoryCommand {
    fn name(&self) -> &'static str {
        "history"
    }
    fn run(&self, args: &[String], _env: &mut Environment) -> ExecStatus {
        let mut n: Option<usize> = None;
        let mut clear = false;

        // Parse arguments
        let mut idx = 0;
        while idx < args.len() {
            match args[idx].as_str() {
                "-c" | "--clear" => {
                    clear = true;
                    idx += 1;
                }
                s if s.chars().all(|c| c.is_ascii_digit()) => {
                    n = s.parse().ok();
                    idx += 1;
                }
                _ => {
                    return Err(ExecError::Custom(format!("history: unknown option '{}'", args[idx])));
                }
            }
        }

        let mut history = self.history.borrow_mut();

        if clear {
            history.clear();
            println!("history cleared.");
            return Ok(ExecOutcome::Code(0));
        }

        let entries = history.list();
        let total = entries.len();
        let start = if let Some(limit) = n {
            if limit > total {
                0
            } else {
                total - limit
            }
        } else {
            0
        };

        for (i, cmd) in entries.iter().enumerate().skip(start) {
            println!("{:>4}  {}", i + 1, cmd);
        }
        Ok(ExecOutcome::Code(0))
    }
}

