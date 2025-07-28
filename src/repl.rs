use std::cell::RefCell;
use std::rc::Rc;
use crate::lexer::Lexer;
use crate::parser::{ Parser, DefaultParser };
use crate::expander::Expander;
use crate::environment::Environment;
use crate::io::InputHandler;
use crate::executor::{
    Executor,
    ExecOutcome,
    RecursiveExecutor,
    FlattenExecutor,
};
use crate::executor::builtin::{
    BuiltinManager,
    HistoryCommand,
};
use crate::history::HistoryManager;
use crate::config::{ ConfigLoader, ExecutorType };

pub struct Repl;

impl Repl {
    pub fn run() {
        let config = match ConfigLoader::load_from_file("./.tinyshrc") {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Failed to load config: {}", e);
                ConfigLoader::default_config()
            }
        };

        let mut env = Environment::new();
        let history_mgr = Rc::new(RefCell::new(
            HistoryManager::load(config.history_file.as_str(), config.history_max).unwrap()
        ));
        let mut builtin_mgr = BuiltinManager::new();
        builtin_mgr.register(Box::new(HistoryCommand { history: Rc::clone(&history_mgr) }));

        loop {
            let line = match InputHandler::read_line(config.prompt.as_str()) {
                Ok(l) => l,
                Err(_) => break,
            };

            {
                let mut history = history_mgr.borrow_mut();
                history.add(line.as_deref().unwrap_or(""));
            }

            let tokens = match &line {
                Some(l) if l.trim().is_empty() => continue,
                Some(l) => {
                    let mut lexer = Lexer::new(l);
                    lexer.tokenize_all()
                }
                None => {
                    // End with EOF (e.g. Ctrl+D)
                    break;
                }
            };

            let mut parser = match tokens {
                Ok(ref toks) => DefaultParser::new(toks),
                Err(ref e) => {
                    eprintln!("{}", e);
                    continue;
                }
            };
            let ast = parser.parse();

            let cwd = std::env::current_dir().unwrap();
            let expander = Expander::new(&env, cwd);
            let expanded = match ast {
                Ok(ast) => match expander.expand(ast) {
                    Ok(expanded_ast) => expanded_ast,
                    Err(e) => {
                        eprintln!("{}", e);
                        continue;
                    }
                }
                Err(e) => {
                    eprintln!("{}", e);
                    continue;
                }
            };

            let mut executor: Box<dyn Executor> = match config.executor_type {
                ExecutorType::Recursive => Box::new(RecursiveExecutor::new(&builtin_mgr)),
                _ => Box::new(FlattenExecutor::new(&builtin_mgr)),
            };
            match executor.exec(&expanded, &mut env) {
                Ok(ExecOutcome::Code(_)) => continue,
                Ok(ExecOutcome::Exit(_)) => break,
                Err(e) => {
                    eprintln!("execution error: {}", e);
                    continue;
                }
            }
        }

        Repl::cleanup(&history_mgr);
    }

    fn cleanup(history_mgr: &Rc<RefCell<HistoryManager>>) {
        println!("Exiting shell...");
        let history = history_mgr.borrow();
        if let Err(e) = history.save() {
            eprintln!("Failed to save history: {}", e);
        }
    }
}

// fn read_logical_line(prompt: &ShellPrompt) -> std::io::Result<String> {
//     let mut lines = String::new();
//
//     loop {
//         prompt.show_prompt(); // change to `> `
//         let mut line = prompt.read_line()?;
//         if line.trim_end().ends_with('\\') {
//             // Remove `\` before newline and concatenate
//             line = line.trim_end().trim_end_matches('\\').to_string();
//             lines.push_str(&line);
//             // Add a space or a line break to the end
//         } else {
//             lines.push_str(&line);
//             break;
//         }
//     }
//     Ok(lines)
// }
