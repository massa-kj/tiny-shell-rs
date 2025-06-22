# tiny-shell-rs

## Architecture Overview

This is a simple shell implementation in Rust, designed to be educational and modular. The architecture is structured to allow for easy expansion and understanding of how a shell operates.
The shell consists of several components that work together in a pipeline fashion:

- **Prompt**: Displays the shell prompt and reads user input.
- **Lexer**: Tokenizes the input string into meaningful components (commands, arguments, operators).
- **Parser**: Constructs an Abstract Syntax Tree (AST) from the sequence of tokens.
- **Expander**: Scans the AST and performs variable expansion, command substitution, tilde expansion, etc.
- **Executor**: Traverses the AST nodes recursively and executes commands, pipes, redirects, and subshells.
- **Environment Management**: Manages environment variables and provides access to them.

- Lexer/Parser/Expander/Executor have a sequential (pipeline) relationship.
- The executor selects the appropriate executor (command, redirect, pipe, etc.) depending on the node type, and also accesses environment management and built-ins.

## Features

### Implemented

- Simple execution of commands (e.g., `ls`, `echo`)

### Planned

- Path resolution (e.g., `command` vs `./command`)
- Pipe support (e.g., `ls | grep txt`)
- Redirection support (e.g., `command > file`, `command < file`)
- Background execution (e.g., `command &`)
- Built-in commands (e.g., `cd`, `exit`, `help`)
- Job control (e.g., `jobs`, `fg`, `bg`)
- Environment variable expansion (e.g., `$HOME`, `${VAR}`)
- Command substitution (e.g., `$(command)`)
- Tilde expansion (e.g., `~/path`)
- Signal handling (e.g., `Ctrl+C` to interrupt)

