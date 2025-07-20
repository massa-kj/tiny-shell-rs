# tiny-shell-rs

## Purpose / Policy

- For study purpose
- Scalability, Testability, Consistency

## Architecture Overview

[design](./docs/design.md)

This is a simple shell implementation in Rust, designed to be educational and modular. The architecture is structured to allow for easy expansion and understanding of how a shell operates.
The shell consists of several components that work together in a pipeline fashion:

## Features

### Implemented

- Simple execution of commands (e.g., `ls`, `echo`)
- Built-in commands (e.g., `cd`, `exit`, `help`)
- Path resolution (e.g., `command` vs `./command`)
- Pipe support (e.g., `command1 | command2 | command3 ...`)
- Redirection support (e.g., `command > file`, `command < file`)

### Planned

- Background execution (e.g., `command &`)
- Job control (e.g., `jobs`, `fg`, `bg`)
- Environment variable expansion (e.g., `$HOME`, `${VAR}`)
- Command substitution (e.g., `$(command)`)
- Tilde expansion (e.g., `~/path`)
- Signal handling (e.g., `Ctrl+C` to interrupt)
- History management
- Complementary Features
- Loading the configuration file

