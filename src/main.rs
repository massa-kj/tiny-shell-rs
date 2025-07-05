fn main() {
    use tiny_shell_rs::lexer::{Lexer};
    use tiny_shell_rs::parser::{Parser, default::DefaultParser};
    use tiny_shell_rs::expander;
    use tiny_shell_rs::environment::Environment;
    use tiny_shell_rs::prompt::ShellPrompt;
    use tiny_shell_rs::executor::{Executor, DefaultExecutor};
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
            Some(l) => Lexer::tokenize(l),
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

        let mut executor = DefaultExecutor;
        match executor.exec(&expanded, &mut env) {
            Ok(_) => continue,
            Err(e) => eprintln!("execution error: {}", e),
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

