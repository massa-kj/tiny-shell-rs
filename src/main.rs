mod lexer;
mod parser;
mod ast;
mod expander;
mod executor;
mod env;
mod prompt;
mod builtins;

use env::Environment;
use prompt::ShellPrompt;
// use executor::execute;
use parser::Parser;

fn main() {
    let mut env = Environment::new();
    let prompt = ShellPrompt::new();

    loop {
        prompt.show_prompt();
        let line = match prompt.read_line() {
            Ok(l) => l,
            Err(_) => break,
        };
        let tokens = match &line {
            Some(l) if l.trim().is_empty() => continue,
            Some(l) => lexer::tokenize(l),
            None => {
                // End with EOF (e.g. Ctrl+D)
                break;
            }
        };
        let mut parser = Parser::new(&tokens);
        let ast = parser.parse();
        let expanded = match ast {
            Ok(ast) => expander::expand(&ast, &env),
            Err(e) => {
                eprintln!("Parse error: {}", e);
                continue;
            }
        };

        let status = executor::execute(&expanded, &mut env);
        if status == 0 {
            // by exit status
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

