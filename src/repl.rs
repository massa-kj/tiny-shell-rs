use std::io::{self, Write};

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

				match crate::executor::execute(line) {
					Ok(status) => {
						if !status.success() {
							eprintln!("Command exited with status: {:?}", status.code());
						}
					}
					Err(e) => {
						eprintln!("Error: {}", e);
					}
				}
            }
            Err(err) => {
                eprintln!("Error reading input: {}", err);
            }
        }
    }
}

