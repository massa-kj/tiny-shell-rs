use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, ExitStatus};
use std::os::unix::process::ExitStatusExt;
use crate::builtins::{is_builtin_command, run_builtin_command, BuiltinStatus};

pub fn execute(line: &str) -> Result<ExitStatus, std::io::Error> {
	let mut parts = line.split_whitespace();
	let cmd = match parts.next() {
		Some(c) => c,
		None => return Ok(ExitStatusExt::from_raw(0)),
	};
	let args: Vec<&str> = parts.collect();

	// Built-in command handling
	if is_builtin_command(cmd) {
		match run_builtin_command(cmd, &args) {
			Ok(BuiltinStatus::Continue) => return Ok(ExitStatusExt::from_raw(0)),
			Ok(BuiltinStatus::Exit) => std::process::exit(0),
			Err(err) => {
				eprintln!("Error executing built-in command '{}': {}", cmd, err);
				return Ok(ExitStatusExt::from_raw(1));
			}
		}
	}

	let cmd_path = resolve_command_path(cmd);
	match cmd_path {
		Some(_path) => {
			Command::new(cmd)
				.args(&args)
				.spawn()?
				.wait()
		}
		None => {
			eprintln!("Command '{}' not found", cmd);
			Ok(ExitStatusExt::from_raw(127)) // Command not found
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

