use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    vars: HashMap<String, String>,
    // pub envs: HashMap<String, String>,
    // pub cwd: String,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            // Import system environment variables as initial values
            vars: std::env::vars().collect(),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.vars.get(key).cloned()
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.vars.insert(key.into(), value.into());
    }

    pub fn unset(&mut self, key: &str) {
        self.vars.remove(key);
    }

    pub fn all(&self) -> Vec<(String, String)> {
        self.vars
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get_unset() {
        let mut env = Environment::new();

        env.set("FOO", "123");
        assert_eq!(env.get("FOO"), Some("123".to_string()));

        env.unset("FOO");
        assert_eq!(env.get("FOO"), None);
    }

    #[test]
    fn test_all() {
        let mut env = Environment::new();
        env.set("X", "abc");
        let all = env.all();
        assert!(all.iter().any(|(k, v)| k == "X" && v == "abc"));
    }
}

