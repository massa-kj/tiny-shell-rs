use std::io::{self, Write};
use std::process::{Command, Stdio};

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

				let mut parts = line.split_whitespace();
				let cmd = match parts.next() {
					Some(c) => c,
					None => continue,
				};
				let args: Vec<&str> = parts.collect();

				let status = Command::new(cmd)
					.args(&args)
					.stdin(Stdio::inherit())
					.stdout(Stdio::inherit())
					.stderr(Stdio::inherit())
					.status();

				match status {
					Ok(status) => {
						if !status.success() {
							eprintln!("Command '{}' exited with status: {}", cmd, status);
						}
					}
					Err(err) => {
						eprintln!("Failed to execute command '{}': {}", cmd, err);
					}
				}
            }
            Err(err) => {
                eprintln!("Error reading input: {}", err);
            }
        }
    }
}

