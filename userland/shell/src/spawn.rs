// userland/shell/src/spawn.rs
// Process Spawning (fork/exec)
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::env::Environment;
use crate::parser::Command;
use gbsd::*;

pub struct ProcessSpawner {
    path_dirs: [[u8; 64]; 16],
    path_count: usize,
}

impl ProcessSpawner {
    pub fn new(env: &Environment) -> Self {
        let mut spawner = Self {
            path_dirs: [[0; 64]; 16],
            path_count: 0,
        };

        spawner.init_path(env);
        spawner
    }

    fn init_path(&mut self, env: &Environment) {
        if let Some(path_value) = env.get("PATH") {
            // Split PATH by ':'
            let mut start = 0;
            for i in 0..path_value.len() {
                if path_value[i] == b':' || path_value[i] == 0 {
                    if self.path_count < 16 && i > start {
                        let len = (i - start).min(63);
                        self.path_dirs[self.path_count][..len]
                            .copy_from_slice(&path_value[start..start + len]);
                        self.path_count += 1;
                    }
                    start = i + 1;
                }
                if path_value[i] == 0 {
                    break;
                }
            }

            // Add last directory
            if start < path_value.len() && self.path_count < 16 {
                let len = (path_value.len() - start).min(63);
                self.path_dirs[self.path_count][..len]
                    .copy_from_slice(&path_value[start..start + len]);
                self.path_count += 1;
            }
        }

        // Default PATH
        if self.path_count == 0 {
            self.path_dirs[0] =
                *b"/bin\0                                                          ";
            self.path_dirs[1] =
                *b"/usr/bin\0                                                      ";
            self.path_count = 2;
        }
    }

    /// Find executable in PATH
    pub fn find_executable(&self, name: &[u8]) -> Option<[u8; 128]> {
        // If name contains '/', use it directly
        for &ch in name {
            if ch == b'/' {
                let mut path = [0u8; 128];
                let len = name.len().min(127);
                path[..len].copy_from_slice(&name[..len]);
                return Some(path);
            }
        }

        // Search in PATH directories
        for i in 0..self.path_count {
            let mut path = [0u8; 128];
            let mut pos = 0;

            // Copy directory
            for j in 0..64 {
                if self.path_dirs[i][j] == 0 {
                    break;
                }
                path[pos] = self.path_dirs[i][j];
                pos += 1;
            }

            // Add '/'
            if pos > 0 && path[pos - 1] != b'/' {
                path[pos] = b'/';
                pos += 1;
            }

            // Add command name
            for &ch in name {
                if ch == 0 {
                    break;
                }
                if pos < 127 {
                    path[pos] = ch;
                    pos += 1;
                }
            }

            // Check if file exists (try to stat it)
            if gbsd::fs::stat(&path[..pos]).is_ok() {
                return Some(path);
            }
        }

        None
    }

    /// Spawn external command using fork/exec
    pub fn spawn(&self, cmd: &Command, env: &Environment) -> Result<i32> {
        // Find executable
        let exec_path = self.find_executable(cmd.name).ok_or(Error::NotFound)?;

        // Fork process
        let pid = gbsd::process::fork()?;

        if pid == 0 {
            // Child process - execute command
            self.exec_child(&exec_path, cmd, env);
            // If exec fails, exit child
            gbsd::process::exit(127);
        } else {
            // Parent process - return child PID
            Ok(pid)
        }
    }

    fn exec_child(&self, path: &[u8], cmd: &Command, _env: &Environment) {
        // Build argv array
        let mut argv_ptrs: [*const u8; 16] = [core::ptr::null(); 16];
        let mut argv_count = 0;

        // argv[0] = command name
        argv_ptrs[0] = cmd.name.as_ptr();
        argv_count += 1;

        // argv[1..] = arguments
        for i in 0..cmd.arg_count {
            if argv_count < 15 {
                if let Some(arg) = cmd.args[i] {
                    argv_ptrs[argv_count] = arg.as_ptr();
                    argv_count += 1;
                }
            }
        }

        // argv[last] = NULL
        argv_ptrs[argv_count] = core::ptr::null();

        // Execute
        let _ = gbsd::process::exec(path, argv_ptrs.as_ptr());

        // If we get here, exec failed
        let _ = crate::io::println(b"exec: command not found");
    }

    /// Wait for child process to complete
    pub fn wait(&self, pid: i32) -> Result<i32> {
        let mut status = 0;
        gbsd::process::waitpid(pid, &mut status)?;
        Ok(status)
    }
}
