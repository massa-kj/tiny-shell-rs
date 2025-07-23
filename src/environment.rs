use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
struct Variable {
    value: String,
    exported: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    vars: HashMap<String, Variable>,
}

impl Environment {
    pub fn new() -> Self {
        let mut env = Environment {
            vars: HashMap::new(),
        };

        // Import all OS environment variables when starting the process (default value)
        for (k, v) in std::env::vars() {
            env.vars.insert(
                k,
                Variable {
                    value: v,
                    exported: true,
                },
            );
        }

        env
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.vars.get(key).map(|v| v.value.as_str())
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.vars
            .entry(key.to_string())
            .and_modify(|var| var.value = value.to_string())
            .or_insert(Variable {
                value: value.to_string(),
                exported: false,
            });
    }

    pub fn unset(&mut self, key: &str) {
        self.vars.remove(key);
    }

    pub fn export(&mut self, key: &str) {
        if let Some(var) = self.vars.get_mut(key) {
            var.exported = true;
        }
    }

    pub fn all(&self) -> Vec<(String, String)> {
        self.vars
            .iter()
            .map(|(k, v)| (k.clone(), v.value.clone()))
            .collect()
    }

    pub fn exported_vars(&self) -> Vec<(String, String)> {
        self.vars
            .iter()
            .filter(|(_, v)| v.exported)
            .map(|(k, v)| (k.clone(), v.value.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_includes_os_env() {
        let env = Environment::new();
        // At least one OS env var should exist
        assert!(!env.vars.is_empty());
    }

    #[test]
    fn test_set_and_get() {
        let mut env = Environment::new();
        env.set("FOO", "bar");
        assert_eq!(env.get("FOO"), Some("bar"));
    }

    #[test]
    fn test_unset() {
        let mut env = Environment::new();
        env.set("FOO", "bar");
        env.unset("FOO");
        assert_eq!(env.get("FOO"), None);
    }

    #[test]
    fn test_export() {
        let mut env = Environment::new();
        env.set("FOO", "bar");
        env.export("FOO");
        let exported = env.exported_vars();
        assert!(exported.iter().any(|(k, v)| k == "FOO" && v == "bar"));
    }

    #[test]
    fn test_all_and_exported_vars() {
        let mut env = Environment::new();
        env.set("FOO", "bar");
        env.export("FOO");
        env.set("BAZ", "qux");
        let all = env.all();
        assert!(all.iter().any(|(k, v)| k == "FOO" && v == "bar"));
        assert!(all.iter().any(|(k, v)| k == "BAZ" && v == "qux"));
        let exported = env.exported_vars();
        assert!(exported.iter().any(|(k, v)| k == "FOO" && v == "bar"));
        assert!(!exported.iter().any(|(k, _)| k == "BAZ"));
    }
}
