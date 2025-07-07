pub struct PathResolver;

impl PathResolver {
    pub fn resolve(&self, command: &str) -> Option<std::path::PathBuf> {
        use std::env;
        use std::fs;
        use std::path::Path;

        if command.contains('/') {
            let path = Path::new(command);
            if path.exists() && path.is_file() {
                return Some(std::path::PathBuf::from(command));
            } else {
                return None;
            }
        }

        if let Ok(paths) = env::var("PATH") {
            for dir in env::split_paths(&paths) {
                let full_path = dir.join(command);
                if full_path.exists() && fs::metadata(&full_path).map(|m| m.is_file()).unwrap_or(false) {
                    return Some(full_path);
                }
            }
        }

        None
    }
}

