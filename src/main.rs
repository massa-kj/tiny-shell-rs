use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
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

				let mut parts = line.split_whitespace();
				let cmd = match parts.next() {
					Some(c) => c,
					None => continue,
				};
				let args: Vec<&str> = parts.collect();

				// Built-in command handling
				match cmd {
					"cd" => {
						let target = args.get(0)
							.map(|s| s.to_string())
							.unwrap_or_else(|| env::var("HOME").unwrap_or_else(|_| "/".to_string()));
						if let Err(err) = env::set_current_dir(&target) {
							eprintln!("cd: {}: {}", target, err);
						}
						continue;
					}
					"help" => {
						println!("tinysh built-in commands:");
						println!("  cd [DIR]   : Change directory");
						println!("  exit       : Exit shell");
						println!("  help       : Show this help");
						continue;
					}
					"exit" => break,
					_ => {}
				}

				let cmd_path = resolve_command_path(cmd);
				match cmd_path {
					Some(_path) => {
						let status = Command::new(cmd)
							.args(&args)
							.stdin(Stdio::inherit())
							.stdout(Stdio::inherit())
							.stderr(Stdio::inherit())
							.status();

						match status {
							Ok(status) if !status.success() => {
								eprintln!("Command '{}' exited with status: {}", cmd, status);
							}
							Err(err) => {
								eprintln!("Failed to execute command '{}': {}", cmd, err);
							}
							_ => {}
						}
					}
					None => {
						eprintln!("Command '{}' not found", cmd);
					}
				}
            }
            Err(err) => {
                eprintln!("Error reading input: {}", err);
            }
        }
    }
}

fn resolve_command_path(cmd: &str) -> Option<String> {
	if cmd.contains('/') {
		let path = Path::new(cmd);
		if path.exists() && path.is_file() {
			return Some(cmd.to_string());
		} else {
			return None;
		}
	}

	// Otherwise, search in PATH
	if let Ok(paths) = env::var("PATH") {
		for dir in env::split_paths(&paths) {
			let full_path = dir.join(cmd);
			if full_path.exists() && fs::metadata(&full_path).map(|m| m.is_file()).unwrap_or(false) {
				return Some(full_path.to_string_lossy().to_string());
			}
		}
	}

	None
}

