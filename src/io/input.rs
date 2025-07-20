use std::io::{Write};

pub struct InputHandler;

impl InputHandler {
    pub fn read_line(prompt: &str) -> std::io::Result<Option<String>> {
        print!("{}", prompt);
        std::io::stdout().flush().unwrap();

        let mut buf = String::new();
        let bytes_read = std::io::stdin().read_line(&mut buf)?;
        if bytes_read == 0 {
            // EOF (e.g., Ctrl-D)
            println!();
            return Ok(None);
        }
        Ok(Some(buf.trim_end().to_string()))
    }
}

