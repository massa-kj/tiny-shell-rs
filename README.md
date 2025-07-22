# tiny-shell-rs

## Architecture Overview

- [design](./docs/design.md)

tiny-shell-rs is a UNIX-like shell implemented in Rust.  
This project emphasizes the following three points.  
- Extensibility: Easy addition of functions and replacement of modules
- Testability: Designed to facilitate automated testing of core logic
- Consistency: Consistent code, UI, and error handling policy

## How to Run

Rename `.tinyshrc.sample` to `.tinyshrc` and place it in your home directory.  
Change the configuration values as needed.  
If you do not place `.tinyshrc`, all default values will be applied.  

```sh
cargo run
```

## Features

### Implemented

- Simple execution of commands (e.g., `ls`, `echo`)
- Built-in commands (e.g., `cd`, `exit`, `help`)
- Path resolution (e.g., `command` vs `./command`)
- Pipe support (e.g., `command1 | command2 | command3 ...`)
- Redirection support (e.g., `command > file`, `command < file`)
- History management
- Loading the configuration file (`.tinyshrc` is similar to an ini file)

### Planned

- Complementary Features
- Environment variable expansion (e.g., `$HOME`, `${VAR}`)
- Tilde expansion (e.g., `~/path`)
- Background execution (e.g., `command &`)
- Job control (e.g., `jobs`, `fg`, `bg`)
- Signal handling (e.g., `Ctrl+C` to interrupt)
- Command substitution (e.g., `$(command)`)

