use std::fs::{File, OpenOptions};
use std::os::unix::io::{AsRawFd};
use std::io;
use crate::ast::{AstNode, RedirectKind};
use crate::executor::{ExecStatus, ExecError, Executor};

pub struct RedirectHandler;

impl RedirectHandler {
    // pub fn new() -> Self {
    //     RedirectHandler
    // }
    //
    pub fn handle_redirect(
        // &self,
        node: &AstNode,
        kind: &RedirectKind,
        file: &str,
        executor: &mut dyn Executor,
        env: &mut crate::environment::Environment,
    ) -> ExecStatus {
        // 1. Save the file descriptor (so it can be restored later)
        // 2. Open the file and replace the appropriate FD with dup2
        // 3. Execute the node (recursively call executor.exec)
        // 4. Restore the FD

        use RedirectKind::*;
        let result = match kind {
            In => {
                let f = File::open(file);
                match f {
                    Ok(f) => {
                        let fd = f.as_raw_fd();
                        let saved = unsafe { libc::dup(0) };
                        if unsafe { libc::dup2(fd, 0) } == -1 {
                            return Err(ExecError::Io(io::Error::last_os_error()));
                        }
                        // Explicitly forget the File so the fd is not closed
                        std::mem::forget(f);
                        let res = executor.exec(node, env);
                        unsafe { libc::dup2(saved, 0); libc::close(saved); }
                        res
                    }
                    Err(e) => Err(ExecError::Io(e)),
                }
            }
            Out => {
                let f = File::create(file);
                match f {
                    Ok(f) => {
                        let fd = f.as_raw_fd();
                        let saved = unsafe { libc::dup(1) };
                        if unsafe { libc::dup2(fd, 1) } == -1 {
                            return Err(ExecError::Io(io::Error::last_os_error()));
                        }
                        std::mem::forget(f);
                        let res = executor.exec(node, env);
                        unsafe { libc::dup2(saved, 1); libc::close(saved); }
                        res
                    }
                    Err(e) => Err(ExecError::Io(e)),
                }
            }
            Append => {
                let f = OpenOptions::new().write(true).append(true).create(true).open(file);
                match f {
                    Ok(f) => {
                        let fd = f.as_raw_fd();
                        let saved = unsafe { libc::dup(1) };
                        if unsafe { libc::dup2(fd, 1) } == -1 {
                            return Err(ExecError::Io(io::Error::last_os_error()));
                        }
                        std::mem::forget(f);
                        let res = executor.exec(node, env);
                        unsafe { libc::dup2(saved, 1); libc::close(saved); }
                        res
                    }
                    Err(e) => Err(ExecError::Io(e)),
                }
            }
        };
        result
    }

    pub fn handle_pipeline(
        // &self,
        left: &AstNode,
        right: &AstNode,
        executor: &mut dyn Executor,
        env: &mut crate::environment::Environment,
    ) -> ExecStatus {
        // 1. create pipe
        let mut fds = [0; 2];
        if unsafe { libc::pipe(fds.as_mut_ptr()) } == -1 {
            return Err(ExecError::Io(io::Error::last_os_error()));
        }
        let (read_fd, write_fd) = (fds[0], fds[1]);

        // 2. Fork the child process
        let pid = unsafe { libc::fork() };
        if pid < 0 {
            return Err(ExecError::Io(io::Error::last_os_error()));
        }
        if pid == 0 {
            // --- Child process: left-side command (output to pipe) ---
            unsafe {
                libc::close(read_fd); // The read end is not needed
                libc::dup2(write_fd, 1); // Redirect stdout to the write end of the pipe
                libc::close(write_fd);
            }
            // Create a new executor and execute the left node
            // Note: Be careful with Rust's drop/RAII here
            // Normally, commands are executed/terminated with execve, etc.
            std::process::exit(
                executor.exec(left, env).unwrap_or_else(|_| 1)
            );
        } else {
            // --- Parent process: right-side command (input from pipe) ---
            unsafe {
                libc::close(write_fd);
                let saved = libc::dup(0);
                libc::dup2(read_fd, 0);
                libc::close(read_fd);
                let status = executor.exec(right, env);
                libc::dup2(saved, 0);
                libc::close(saved);
                status
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{File, read_to_string, remove_file};
    use std::io::Write;
    use super::*;
    use crate::ast::{AstNode, CommandNode, CommandKind, RedirectKind};
    use crate::environment::Environment;

    #[test]
    fn test_redirect_out_creates_file() {
        // 1. Test output file name
        let file_name = "test_redirect_out.txt";
        let _ = remove_file(file_name); // Delete if it was left over from last time

        // 2. Prepare the command node (for mock Executor)
        let cmd = CommandNode {
            name: "echo".to_string(),
            args: vec!["hello".to_string()],
            kind: CommandKind::Simple,
        };
        let cmd_node = AstNode::Command(cmd.clone());

        // 3. Prepare the redirect node
        let _redirect_node = AstNode::Redirect {
            node: Box::new(cmd_node.clone()),
            kind: RedirectKind::Out,
            file: file_name.to_string(),
        };

        // 4. Prepare mock Executor, Environment, and RedirectHandler
        let mut mock_executor = crate::executor::tests::MockExecutor::new();
        let mut env = Environment::new();
        // let handler = RedirectHandler::new();


        // 5. execute by handle_redirect
        let res = RedirectHandler::handle_redirect(
            &AstNode::Command(cmd.clone()),
            &RedirectKind::Out,
            file_name,
            &mut mock_executor,
            &mut env,
        );
        assert!(res.is_ok());

        // 6. Confirm that the file has been created
        let content = read_to_string(file_name).unwrap_or_default();
        // * Since this is a mock Executor, there is no real output, but the file itself should be generated
        assert!(std::path::Path::new(file_name).exists());

        // 7. Cleanup
        let _ = remove_file(file_name);
    }

    #[test]
    fn test_redirect_in_reads_file() {
        let file_name = "test_redirect_in.txt";
        let mut f = File::create(file_name).unwrap();
        writeln!(f, "hello world").unwrap();

        let cmd = CommandNode {
            name: "cat".to_string(),
            args: vec![],
            kind: CommandKind::Simple,
        };
        let cmd_node = AstNode::Command(cmd.clone());

        let _redirect_node = AstNode::Redirect {
            node: Box::new(cmd_node.clone()),
            kind: RedirectKind::In,
            file: file_name.to_string(),
        };

        let mut mock_executor = crate::executor::tests::MockExecutor::new();
        let mut env = Environment::new();
        // let handler = RedirectHandler::new();

        let res = RedirectHandler::handle_redirect(
            &AstNode::Command(cmd.clone()),
            &RedirectKind::In,
            file_name,
            &mut mock_executor,
            &mut env,
        );
        assert!(res.is_ok());

        let _ = remove_file(file_name);
    }
}

