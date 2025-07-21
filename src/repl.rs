use std::cell::RefCell;
use std::rc::Rc;
use crate::lexer::Lexer;
use crate::parser::{ Parser, DefaultParser };
use crate::expander;
use crate::environment::Environment;
use crate::io::InputHandler;
use crate::executor::{Executor, ExecOutcome, RecursiveExecutor, FlattenExecutor, BuiltinManager, HistoryCommand};
use crate::history::HistoryManager;

pub struct Repl;

impl Repl {
    pub fn run() {
        let mut env = Environment::new();
        let history_mgr = Rc::new(RefCell::new(
            HistoryManager::load("./.my_shell_history", 20).unwrap()
        ));
        let mut builtin_mgr = BuiltinManager::new();
        builtin_mgr.register(Box::new(HistoryCommand { history: Rc::clone(&history_mgr) }));

        loop {
            let line = match InputHandler::read_line("$ ") {
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

            let expanded = match ast {
                Ok(ast) => expander::expand(&ast, &env),
                Err(e) => {
                    eprintln!("{}", e);
                    continue;
                }
            };

            // let mut executor = RecursiveExecutor::new(&builtin_mgr);
            let mut executor = FlattenExecutor::new(&builtin_mgr);
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
