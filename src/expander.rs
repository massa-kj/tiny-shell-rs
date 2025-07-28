use std::{env, fmt};
use std::path::PathBuf;
use crate::ast::{AstNode, CommandNode};
use crate::environment::Environment;

pub struct Expander<'a> {
    env: &'a Environment,
    cwd: std::path::PathBuf, // Required for wildcard expansion
}

impl<'a> Expander<'a> {
    pub fn new(env: &'a Environment, cwd: impl Into<std::path::PathBuf>) -> Self {
        Self {
            env,
            cwd: cwd.into(),
        }
    }

    // Recursively expands the AST (such as command substitution, variable expansion, wildcard expansion, etc.)
    pub fn expand(&self, node: AstNode) -> Result<AstNode, ExpandError> {
        match node {
            AstNode::Command(cmd) => {
                let expanded = self.expand_command(cmd)?;
                Ok(AstNode::Command(expanded))
            }
            AstNode::Subshell(inner) => {
                let expanded = self.expand(*inner)?;
                Ok(AstNode::Subshell(Box::new(expanded)))
            }
            AstNode::Redirect { node, kind, file } => {
                let expanded_node = self.expand(*node)?;
                let file = self.expand_single_arg(&file)?;
                Ok(AstNode::Redirect {
                    node: Box::new(expanded_node),
                    kind,
                    file,
                })
            }
            AstNode::Pipeline(nodes) => {
                let expanded_nodes = nodes
                    .into_iter()
                    .map(|node| self.expand(node))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(AstNode::Pipeline(expanded_nodes))
            }
            AstNode::And(left, right) => {
                Ok(AstNode::And(
                    Box::new(self.expand(*left)?),
                    Box::new(self.expand(*right)?),
                ))
            }
            AstNode::Or(left, right) => {
                Ok(AstNode::Or(
                    Box::new(self.expand(*left)?),
                    Box::new(self.expand(*right)?),
                ))
            }
            AstNode::Sequence(nodes) => {
                let expanded_nodes = nodes
                    .into_iter()
                    .map(|node| self.expand(node))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(AstNode::Sequence(expanded_nodes))
            }
            _ => Err(ExpandError::Unsupported("AST node not yet handled".into())),
        }
    }

    fn expand_command(&self, cmd: CommandNode) -> Result<CommandNode, ExpandError> {
        let name_parts = self.expand_arg(&cmd.name)?;
        let args_parts = cmd
            .args
            .iter()
            .flat_map(|arg| self.expand_arg(arg).unwrap_or_else(|_| vec![arg.clone()]))
            .collect::<Vec<_>>();

        Ok(CommandNode {
            name: name_parts.get(0).cloned().unwrap_or_default(),
            args: args_parts,
            kind: cmd.kind,
        })
    }

    // Argument expansion (variable, command, wildcard, quote processing)
    pub fn expand_arg(&self, arg: &str) -> Result<Vec<String>, ExpandError> {
        // Temporary implementation: actually should tokenize → expand → split
        let s = self.expand_tilde(arg)?;
        let s = self.substitute_vars(&s)?;
        let s = self.command_substitute(&s)?;
        let parts = self.glob_expand(&s)?;
        Ok(parts)
    }

    // Expansion of quoted heredoc
    pub fn expand_heredoc(&self, content: &str, quoted: bool) -> Result<String, ExpandError> {
        if quoted {
            Ok(content.to_string()) // No expansion
        } else {
            let s = self.substitute_vars(content)?;
            let s = self.command_substitute(&s)?;
            Ok(s)
        }
    }

    fn substitute_vars(&self, input: &str) -> Result<String, ExpandError> {
        // Example: Replace $VAR, ${VAR} with environment variables
        let mut result = String::new();
        let mut chars = input.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\\' {
                // Escaped character → Add as is
                if let Some(next) = chars.next() {
                    result.push(next);
                }
            } else if ch == '$' {
                match chars.peek() {
                    Some('{') => {
                        chars.next(); // skip '{'
                        let mut var_name = String::new();
                        while let Some(&c) = chars.peek() {
                            if c == '}' {
                                chars.next(); // skip '}'
                                break;
                            }
                            var_name.push(c);
                            chars.next();
                        }
                        let value = self.env.get(&var_name).unwrap_or("").to_string();
                        result.push_str(&value);
                    }
                    Some(c) if is_var_start_char(*c) => {
                        let mut var_name = String::new();
                        while let Some(&c) = chars.peek() {
                            if is_var_char(c) {
                                var_name.push(c);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        let value = self.env.get(&var_name).unwrap_or("").to_string();
                        result.push_str(&value);
                    }
                    _ => {
                        // No variable name follows $ → Add $ as is
                        result.push('$');
                    }
                }
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }

    fn command_substitute(&self, input: &str) -> Result<String, ExpandError> {
        // Example: Replace $(echo foo) by executing and substituting output
        // Not implemented yet
        Ok(input.to_string()) // 仮
    }

    fn glob_expand(&self, pattern: &str) -> Result<Vec<String>, ExpandError> {
        // Example: *.rs → ["main.rs", "lib.rs"], etc.
        // Not implemented yet
        Ok(vec![pattern.to_string()]) // 仮
    }

    fn expand_single_arg(&self, s: &str) -> Result<String, ExpandError> {
        self.expand_arg(s).map(|mut v| v.remove(0))
    }

    fn expand_tilde(&self, arg: &str) -> Result<String, ExpandError> {
        if let Some(rest) = arg.strip_prefix('~') {
            let path = rest;
            let home = env::var("HOME").map(PathBuf::from)
                .map_err(|e| ExpandError::TildeExpandFailed(e.to_string()))?;
            Ok(format!("{}{}", home.display(), path))
        } else {
            Ok(arg.to_string())
        }
    }
}

fn is_var_start_char(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_var_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

#[derive(Debug)]
pub enum ExpandError {
    InvalidVariableSyntax,
    CommandSubstitutionFailed(String),
    GlobPatternError(String),
    TildeExpandFailed(String),
    IoError(std::io::Error),
    Unsupported(String),
}
impl fmt::Display for ExpandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExpandError::InvalidVariableSyntax => write!(f, "Invalid variable syntax"),
            ExpandError::CommandSubstitutionFailed(cmd) => write!(f, "Command substitution failed: {}", cmd),
            ExpandError::GlobPatternError(pattern) => write!(f, "Glob pattern error: {}", pattern),
            ExpandError::TildeExpandFailed(user) => write!(f, "Tilde expansion failed for user: {}", user),
            ExpandError::IoError(e) => write!(f, "IO error: {}", e),
            ExpandError::Unsupported(msg) => write!(f, "Unsupported operation: {}", msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::expander::{Expander, ExpandError};
    use crate::environment::Environment;
    use std::path::PathBuf;

    fn setup_env() -> Environment {
        let mut env = Environment::new();
        env.set("USER", "user");
        env.set("EMPTY", "");
        env
    }

    fn with_expander<F: FnOnce(&Expander)>(test: F) {
        let env = setup_env();
        let expander = Expander::new(&env, ".");
        test(&expander);
    }

    #[test]
    fn test_variable_substitution_simple() {
        with_expander(|expander| {
            let result = expander.expand_arg("Hello $USER").unwrap();
            assert_eq!(result, vec!["Hello user"]);
        });
    }

    #[test]
    fn test_variable_substitution_braced() {
        with_expander(|expander| {
            let result = expander.expand_arg("Home: ${USER}land").unwrap();
            assert_eq!(result, vec!["Home: userland"]);
        });
    }

    #[test]
    fn test_variable_substitution_missing() {
        with_expander(|expander| {
            let result = expander.expand_arg("Unset: $NOTFOUND").unwrap();
            assert_eq!(result, vec!["Unset: "]);
        });
    }

    #[test]
    fn test_command_substitution_basic() {
        with_expander(|expander| {
            let result = expander.expand_arg("Today is $(echo Friday)").unwrap();
            assert_eq!(result, vec!["Today is Friday"]);
        });
    }

    #[test]
    fn test_command_substitution_backtick() {
        with_expander(|expander| {
            let result = expander.expand_arg("Now: `echo 42`").unwrap();
            assert_eq!(result, vec!["Now: 42"]);
        });
    }

    #[test]
    fn test_glob_expansion() {
        with_expander(|expander| {
            let result = expander.expand_arg("src/*.rs").unwrap();
            assert!(result.iter().any(|s| s.ends_with(".rs")));
        });
    }

    #[test]
    fn test_quoted_string_expansion() {
        with_expander(|expander| {
            let result = expander.expand_arg("\"Hello $USER\"").unwrap();
            assert_eq!(result, vec!["Hello user"]);
        });
    }

    #[test]
    fn test_escaped_characters() {
        with_expander(|expander| {
            let result = expander.expand_arg("Line\\nBreak\\$USER").unwrap();
            assert_eq!(result, vec!["Line\\nBreak$USER"]);
        });
    }

    #[test]
    fn test_heredoc_unquoted() {
        with_expander(|expander| {
            let input = "Path: $USER\nToday: $(echo Sun)";
            let output = expander.expand_heredoc(input, false).unwrap();
            assert_eq!(output.trim(), "Path: user\nToday: Sun");
        });
    }

    #[test]
    fn test_heredoc_quoted_no_expansion() {
        with_expander(|expander| {
            let input = "Path: $USER\nToday: $(echo Sun)";
            let output = expander.expand_heredoc(input, true).unwrap();
            assert_eq!(output.trim(), input);
        });
    }

    #[test]
    fn test_glob_no_match_returns_literal() {
        with_expander(|expander| {
            let result = expander.expand_arg("no_such_file_*.xyz").unwrap();
            assert_eq!(result, vec!["no_such_file_*.xyz"]);
        });
    }

    #[test]
    fn test_empty_variable() {
        with_expander(|expander| {
            let result = expander.expand_arg("[$EMPTY]").unwrap();
            assert_eq!(result, vec!["[]"]);
        });
    }

    #[test]
    fn test_tilde_expand_home() {
        with_expander(|expander| {
            let home = std::env::var("HOME").unwrap();
            let result = expander.expand_tilde("~").unwrap();
            assert_eq!(result, home);

            let result = expander.expand_tilde("~/foo/bar").unwrap();
            assert_eq!(result, format!("{}/foo/bar", home));
        });
    }
}
