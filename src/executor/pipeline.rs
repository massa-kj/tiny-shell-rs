use super::super::executor::{ ExecStatus, ExecError };

pub struct PipelineHandler;

impl PipelineHandler {
    pub fn exec_pipeline_generic<T, F>(
        nodes: &[T],
        mut exec_fn: F,
    ) -> ExecStatus
    where
        F: FnMut(&T) -> Result<i32, ExecError>,
    {
        if nodes.len() < 2 {
            return Err(ExecError::Custom("Pipeline must have at least two commands".into()));
        }

        let mut prev_read_fd: Option<i32> = None;
        let mut child_pids = Vec::new();

        for (i, node) in nodes.iter().enumerate() {
            let is_last = i == nodes.len() - 1;
            let mut pipefds = [0; 2];

            if !is_last {
                if unsafe { libc::pipe(pipefds.as_mut_ptr()) } == -1 {
                    return Err(ExecError::Io(std::io::Error::last_os_error()));
                }
            }

            let pid = unsafe { libc::fork() };
            if pid < 0 {
                return Err(ExecError::Io(std::io::Error::last_os_error()));
            }

            if pid == 0 {
                // Child process
                if let Some(read_fd) = prev_read_fd {
                    unsafe {
                        libc::dup2(read_fd, 0);
                        libc::close(read_fd);
                    }
                }
                if !is_last {
                    unsafe {
                        libc::close(pipefds[0]);
                        libc::dup2(pipefds[1], 1);
                        libc::close(pipefds[1]);
                    }
                }
                std::process::exit(exec_fn(node).unwrap_or(1));
            } else {
                // Parent process
                if let Some(read_fd) = prev_read_fd {
                    unsafe { libc::close(read_fd); }
                }
                if !is_last {
                    unsafe { libc::close(pipefds[1]); }
                    prev_read_fd = Some(pipefds[0]);
                } else {
                    prev_read_fd = None;
                }
                child_pids.push(pid);
            }
        }

        for pid in child_pids {
            let mut status_code = 0;
            unsafe { libc::waitpid(pid, &mut status_code, 0); }
        }
        Ok(0)
    }
}

