use std::io::{self, Write};

fn main() {
    loop {
        // Display prompt
        print!("tinysh> ");
        io::stdout().flush().unwrap();

        // Read input from stdin
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                // End with EOF (e.g. Ctrl+D)
                println!("\nexit");
                break;
            }
            Ok(_) => {
                let line = input.trim();
                if line.is_empty() {
                    continue;
                }

                if line == "exit" {
                    break;
                }

                // TODO: Temporary Implementation
                println!("Entered: '{}'", line);
            }
            Err(err) => {
                eprintln!("Error reading input: {}", err);
            }
        }
    }
}

