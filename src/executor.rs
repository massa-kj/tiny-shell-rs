use std::env;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::parser::ParsedCommand;

pub fn execute_parsed(cmd: ParsedCommand) -> Result<(), io::Error> {
	use crate::builtins::{is_builtin_command, run_builtin_command, BuiltinStatus};

	// Built-in command handling
	if is_builtin_command(cmd.command) {
		match run_builtin_command(cmd.command, &cmd.args) {
			Ok(BuiltinStatus::Continue) => return Ok(()),
			Ok(BuiltinStatus::Exit) => std::process::exit(0),
			Err(err) => {
				eprintln!("{}", err);
				return Ok(());
			}
		}
	}

	let path = super::executor::resolve_command_path(cmd.command).ok_or_else(|| {
		io::Error::new(io::ErrorKind::NotFound, format!("Command not found: {}", cmd.command))
	})?;

	let mut command = Command::new(path);
	command.args(cmd.args);

	// stdin
	if let Some(input_file) = cmd.stdin {
		let f = File::open(input_file)?;
		command.stdin(Stdio::from(f));
	}

	// stdout
	if let Some((output_file, append)) = cmd.stdout {
		let f = if append {
			OpenOptions::new().create(true).append(true).open(output_file)?
		} else {
			OpenOptions::new().create(true).truncate(true).write(true).open(output_file)?
		};
		command.stdout(Stdio::from(f));
	}

	let status = command.spawn()?.wait()?;

	if !status.success() {
		eprintln!("Command exited with status: {:?}", status.code());
	}

	Ok(())
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

