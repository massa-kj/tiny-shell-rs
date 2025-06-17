use std::env;

pub enum BuiltinStatus {
	Continue,
	Exit,
}

pub fn is_builtin_command(cmd: &str) -> bool {
	match cmd {
		"cd" | "help" | "exit" => true,
		_ => false,
	}
}

pub fn run_builtin_command(cmd: &str, args: &[&str]) -> Result<BuiltinStatus, String> {
	match cmd {
		"cd" => {
			let target = args.get(0)
				.map(|s| s.to_string())
				.unwrap_or_else(|| env::var("HOME").unwrap_or_else(|_| "/".to_string()));
			if let Err(err) = env::set_current_dir(&target) {
				eprintln!("cd: {}: {}", target, err);
			}
			Ok(BuiltinStatus::Continue)
		}
		"help" => {
			println!("Available built-in commands:");
			println!("  cd [DIR]   : Change directory");
			println!("  exit       : Exit shell");
			println!("  help       : Show this help");
			Ok(BuiltinStatus::Continue)
		}
		"exit" => Ok(BuiltinStatus::Exit),
		_ => Err(format!("Unknown builtin command: {}", cmd)),
	}
}

