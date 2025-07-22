use std::collections::HashMap;
use std::{ io, fmt };
use std::io::{ BufRead, BufReader };
use std::fs::File;

#[derive(Debug, Clone)]
pub struct Config {
    pub prompt: String,
    pub history_file: String,
    pub history_max: usize,
    pub executor_type: ExecutorType,
    pub aliases: HashMap<String, String>,
    pub env_vars: HashMap<String, String>,
}

pub struct ConfigLoader;

impl ConfigLoader {
    pub fn default_config() -> Config {
        Config {
            prompt: "$ ".to_string(),
            history_file: "~/.tiny_shell_history".to_string(),
            history_max: 500,
            executor_type: ExecutorType::Flatten,
            aliases: HashMap::new(),
            env_vars: HashMap::new(),
        }
    }

    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Config, ConfigError> {
        let file = File::open(path).map_err(ConfigError::Io)?;
        let mut src = String::new();
        for line in BufReader::new(file).lines() {
            let line = line.map_err(ConfigError::Io)?;
            src.push_str(&line);
            src.push('\n');
        }
        Self::load_from_str(&src)
    }

    pub fn load_from_str(src: &str) -> Result<Config, ConfigError> {
        let mut prompt = None;
        let mut history_file = None;
        let mut history_max = None;
        let mut executor_type = None;
        let mut aliases = HashMap::new();
        let mut env_vars = HashMap::new();

        for (lineno, line) in src.lines().enumerate() {
            let line = line;
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                return Err(ConfigError::Parse(format!("Line {}: No '=' found: {}", lineno+1, line)));
            };
            let key = key.trim();
            let value = if let Some(idx) = line.find('=') {
                &line[idx + 1..]
            } else {
                value.trim()
            };

            match key {
                "prompt" => prompt = Some(value.to_string()),
                "history_file" => history_file = Some(value.to_string()),
                "history_max" => match value.parse::<usize>() {
                    Ok(n) => history_max = Some(n),
                    Err(_) => return Err(ConfigError::Parse(format!("Line {}: Invalid usize: {}", lineno+1, line))),
                },
                "executor_type" => {
                    executor_type = match value {
                        "recursive" => Some(ExecutorType::Recursive),
                        _ => Some(ExecutorType::Flatten),
                    };
                }
                k if k.starts_with("alias.") => {
                    let alias = k.trim_start_matches("alias.").to_string();
                    aliases.insert(alias, value.to_string());
                }
                k if k.starts_with("env.") => {
                    let var = k.trim_start_matches("env.").to_string();
                    env_vars.insert(var, value.to_string());
                }
                _ => return Err(ConfigError::Parse(format!("Line {}: Unknown key: {}", lineno+1, key))),
            }
        }

        let default = ConfigLoader::default_config();
        Ok(Config {
            prompt: prompt.unwrap_or(default.prompt),
            history_file: history_file.unwrap_or(default.history_file),
            history_max: history_max.unwrap_or(default.history_max),
            executor_type: executor_type.unwrap_or(default.executor_type),
            aliases,
            env_vars,
        })
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Parse(String),
    // Validation
}
impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "IO error: {}", e),
            ConfigError::Parse(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutorType {
    Flatten,
    Recursive,
}

