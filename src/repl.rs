use std::io::{self, Write};
use crate::parser;

pub fn start() {
    loop {
        // Display prompt
        print!("tinysh> ");
        io::stdout().flush().unwrap();

        // Read input from stdin
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                // End with EOF (e.g. Ctrl+D)
                println!();
                break;
            }
            Ok(_) => {
                let line = input.trim();
                if line.is_empty() {
                    continue;
                }

                let pipeline = parser::split_pipeline(line);
                if pipeline.is_empty() {
                    continue;
                }

                if let Err(e) = crate::executor::execute_pipeline(pipeline) {
                    eprintln!("Error: {}", e);
                }
            }
            Err(err) => {
                eprintln!("Error reading input: {}", err);
            }
        }
    }
}

