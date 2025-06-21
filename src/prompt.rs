use std::io::{self, Write};

pub struct ShellPrompt {}

impl ShellPrompt {
    pub fn new() -> Self {
        ShellPrompt {}
    }
    pub fn show_prompt(&self) {
        print!("$ ");
        io::stdout().flush().unwrap();
    }
    pub fn read_line(&self) -> io::Result<Option<String>> {
        let mut buf = String::new();
        let bytes_read = io::stdin().read_line(&mut buf)?;
        if bytes_read == 0 {
            // EOF (e.g., Ctrl-D)
            println!();
            return Ok(None);
        }
        Ok(Some(buf.trim_end().to_string()))
    }
}

