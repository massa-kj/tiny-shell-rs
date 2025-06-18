#[derive(Debug)]
pub struct ParsedCommand<'a> {
	pub command: &'a str,
	pub args: Vec<&'a str>,
	pub stdin: Option<&'a str>,
	pub stdout: Option<(&'a str, bool)>, // (file, append)
}

pub fn parse_line(line: &str) -> ParsedCommand {
	let mut parts = line.split_whitespace().peekable();

	let mut command = "";
	let mut args = Vec::new();
	let mut stdin = None;
	let mut stdout = None;

	while let Some(&token) = parts.peek() {
		match token {
			">" | ">>" => {
				parts.next(); // consume '>' or '>>'
				if let Some(output_file) = parts.next() {
					let append = token == ">>";
					stdout = Some((output_file, append));
				} else {
					eprintln!("Error: No output file specified after '{}'", token);
				}
			}
			"<" => {
				parts.next(); // consume '<'
				if let Some(input_file) = parts.next() {
					stdin = Some(input_file);
				} else {
					eprintln!("Error: No input file specified after '<'");
				}
			}
			_ => {
				if command.is_empty() {
					command = parts.next().unwrap();
				} else {
					args.push(parts.next().unwrap());
				}
			}
		}
	}

	ParsedCommand {
		command,
		args,
		stdin,
		stdout,
	}
}

