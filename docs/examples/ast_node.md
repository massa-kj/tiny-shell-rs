# AST node examples

- `echo "Hello, World!"`

```rust
AstNode::Command(CommandNode {
    name: "echo".to_string(),
    args: vec!["Hello, World!".to_string()],
    kind: CommandKind::External,
});
```

- `ls -l > ls.txt; cat < ls.txt | grep "txt" | wc > output.txt`

```rust
AstNode::Sequence(
    Box::new(AstNode::Redirect {
        node: Box::new(AstNode::Command(CommandNode {
            name: "ls".to_string(),
            args: vec!["-l".to_string()],
            kind: CommandKind::External,
        })),
        kind: RedirectKind::Out,
        file: "ls.txt".to_string(),
    }),
    Box::new(AstNode::Pipeline(
        Box::new(AstNode::Redirect {
            node: Box::new(AstNode::Command(CommandNode {
                name: "cat".to_string(),
                args: vec!["ls.txt".to_string()],
                kind: CommandKind::External,
            })),
            kind: RedirectKind::In,
            file: "".to_string(), // Input redirection does not require a file
        }),
        Box::new(AstNode::Pipeline(
            Box::new(AstNode::Command(CommandNode {
                name: "grep".to_string(),
                args: vec!["txt".to_string()],
                kind: CommandKind::External,
            })),
            Box::new(AstNode::Redirect {
                node: Box::new(AstNode::Command(CommandNode {
                    name: "wc".to_string(),
                    args: vec![],
                    kind: CommandKind::External,
                })),
                kind: RedirectKind::Out,
                file: "output.txt".to_string(),
            }),
        )),
    )),
);
```

- `(cd /tmp && ls) || echo "Failed"`

```rust
AstNode::Or(
    Box::new(AstNode::Subshell(Box::new(AstNode::Sequence(
        Box::new(AstNode::Command(CommandNode {
            name: "cd".to_string(),
            args: vec!["/tmp".to_string()],
            kind: CommandKind::External,
        })),
        Box::new(AstNode::Command(CommandNode {
            name: "ls".to_string(),
            args: vec![],
            kind: CommandKind::External,
        })),
    )))),
    Box::new(AstNode::Command(CommandNode {
        name: "echo".to_string(),
        args: vec!["Failed".to_string()],
        kind: CommandKind::Builtin,
    })),
);
```

- `if [ -f file.txt ]; then echo "File exists"; fi`

```rust
AstNode::If(
    Box::new(AstNode::Command(CommandNode {
        name: "test".to_string(),
        args: vec!("-f".to_string(), "file.txt".to_string()),
        kind: CommandKind::Builtin,
    })),
    Box::new(AstNode::Command(CommandNode {
        name: "echo".to_string(),
        args: vec!["File exists".to_string()],
        kind: CommandKind::Builtin,
    })),
);
```

