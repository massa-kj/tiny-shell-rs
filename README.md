# tiny-shell-rs

A minimal UNIX-like shell implemented in Rust.  
This project is both a learning journey and an attempt to design a practical, extensible shell from scratch.

## Goals

tiny-shell-rs emphasizes the following principles:

- **Extensibility**: Modular architecture for easy function addition and replacement
- **Testability**: Core logic designed with unit and integration testing in mind
- **Consistency**: Uniform conventions for code structure, CLI behavior, and error reporting

## Architecture Overview

- See [Design Document](./docs/design.md) for architecture and module-level details.

## Getting Started

1. Clone the repository
2. (Optional) Rename `.tinyshrc.sample` to `.tinyshrc` and place it in your home directory  
3. (Optional) Adjust configuration values as needed (or skip — defaults will apply)

```sh
cargo run
```

## Features

### Implemented Features

- Basic command execution (e.g., `ls`, `echo`)
- Built-in commands (e.g., `cd`, `exit`, `help`)
- Path resolution (`command` vs `./command`)
- Piping (`command1 | command2 | command3`)
- Redirection (`command > file`, `command < file`)
- Command history
- Configuration file loading (`.tinyshrc` — ini-like format)
- Environment variable management (`export`, `unset`)
- Environment variable expansion (`$HOME`, `${VAR}`)
- Tilde expansion (`~/path`)

### Work in Progress / Planned

These features are currently being designed or prototyped:

- Wildcard/glob expansion (`**/*.txt`)
- Command substitution (`$(command)`)
- Background execution (`command &`)
- Job control (`jobs`, `fg`, `bg`)
- Signal handling (Ctrl+C interrupt, SIGTSTP, etc.)
- Auto-completion and suggestions

