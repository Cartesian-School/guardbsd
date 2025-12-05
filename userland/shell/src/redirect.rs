// userland/shell/src/redirect.rs
// Pipes & Redirection (bash/zsh-inspired)
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use gbsd::*;

#[derive(Copy, Clone, PartialEq)]
pub enum RedirectType {
    None,
    Output,        // >
    Append,        // >>
    Input,         // <
}

pub struct Redirect {
    pub rtype: RedirectType,
    pub filename: [u8; 256],
    pub filename_len: usize,
}

impl Redirect {
    pub const fn new() -> Self {
        Self {
            rtype: RedirectType::None,
            filename: [0; 256],
            filename_len: 0,
        }
    }
    
    pub fn parse(input: &[u8]) -> (Option<Self>, &[u8]) {
        let mut redirect = None;
        let mut cmd_end = input.len();
        
        // Find redirection operators
        for i in 0..input.len() {
            if i + 1 < input.len() && input[i] == b'>' && input[i + 1] == b'>' {
                // >> (append)
                redirect = Some(Self::parse_redirect(RedirectType::Append, &input[i + 2..]));
                cmd_end = i;
                break;
            } else if input[i] == b'>' {
                // > (output)
                redirect = Some(Self::parse_redirect(RedirectType::Output, &input[i + 1..]));
                cmd_end = i;
                break;
            } else if input[i] == b'<' {
                // < (input)
                redirect = Some(Self::parse_redirect(RedirectType::Input, &input[i + 1..]));
                cmd_end = i;
                break;
            }
        }
        
        (redirect, &input[..cmd_end])
    }
    
    fn parse_redirect(rtype: RedirectType, rest: &[u8]) -> Self {
        let mut redir = Self::new();
        redir.rtype = rtype;
        
        // Skip whitespace
        let mut start = 0;
        while start < rest.len() && (rest[start] == b' ' || rest[start] == b'\t') {
            start += 1;
        }
        
        // Extract filename (until whitespace or end)
        let mut end = start;
        while end < rest.len() && rest[end] != b' ' && rest[end] != b'\t' && rest[end] != 0 {
            end += 1;
        }
        
        let len = (end - start).min(255);
        redir.filename[..len].copy_from_slice(&rest[start..start + len]);
        redir.filename_len = len;
        
        redir
    }
    
    pub fn apply(&self, saved_stdout: &mut Option<i32>, saved_stdin: &mut Option<i32>) -> Result<()> {
        match self.rtype {
            RedirectType::Output => {
                // Open file for writing (create/truncate)
                let fd = gbsd::fs::open(&self.filename[..self.filename_len], 0o644)?;
                
                // Save current stdout and redirect
                *saved_stdout = Some(1); // TODO: dup(1)
                // TODO: dup2(fd, 1)
                gbsd::fs::close(fd)?;
                Ok(())
            }
            RedirectType::Append => {
                // Open file for appending
                let fd = gbsd::fs::open(&self.filename[..self.filename_len], 0o644)?;
                
                // TODO: Seek to end
                *saved_stdout = Some(1); // TODO: dup(1)
                // TODO: dup2(fd, 1)
                gbsd::fs::close(fd)?;
                Ok(())
            }
            RedirectType::Input => {
                // Open file for reading
                let fd = gbsd::fs::open(&self.filename[..self.filename_len], 0o444)?;
                
                *saved_stdin = Some(0); // TODO: dup(0)
                // TODO: dup2(fd, 0)
                gbsd::fs::close(fd)?;
                Ok(())
            }
            RedirectType::None => Ok(()),
        }
    }
    
    pub fn restore(saved_stdout: Option<i32>, saved_stdin: Option<i32>) -> Result<()> {
        if let Some(_old_stdout) = saved_stdout {
            // TODO: dup2(old_stdout, 1); close(old_stdout)
        }
        if let Some(_old_stdin) = saved_stdin {
            // TODO: dup2(old_stdin, 0); close(old_stdin)
        }
        Ok(())
    }
}

pub struct Pipeline {
    pub commands: [Option<[u8; 256]>; 8],
    pub command_count: usize,
}

impl Pipeline {
    pub const fn new() -> Self {
        Self {
            commands: [None; 8],
            command_count: 0,
        }
    }
    
    /// Parse input for pipes
    pub fn parse(input: &[u8]) -> Self {
        let mut pipeline = Self::new();
        let mut cmd_start = 0;
        
        for i in 0..input.len() {
            if input[i] == b'|' {
                // Found pipe
                if cmd_start < i && pipeline.command_count < 8 {
                    let mut cmd = [0u8; 256];
                    let len = (i - cmd_start).min(255);
                    cmd[..len].copy_from_slice(&input[cmd_start..cmd_start + len]);
                    pipeline.commands[pipeline.command_count] = Some(cmd);
                    pipeline.command_count += 1;
                }
                cmd_start = i + 1;
            }
        }
        
        // Add last command
        if cmd_start < input.len() && pipeline.command_count < 8 {
            let mut cmd = [0u8; 256];
            let len = (input.len() - cmd_start).min(255);
            cmd[..len].copy_from_slice(&input[cmd_start..cmd_start + len]);
            pipeline.commands[pipeline.command_count] = Some(cmd);
            pipeline.command_count += 1;
        }
        
        pipeline
    }
    
    pub fn has_pipe(&self) -> bool {
        self.command_count > 1
    }
    
    pub fn execute(&self) -> Result<()> {
        if !self.has_pipe() {
            return Ok(());
        }
        
        // TODO: Implement pipe execution with fork/exec
        // For each command except last:
        //   1. Create pipe with pipe()
        //   2. Fork child process
        //   3. In child: dup2 pipe write end to stdout
        //   4. In parent: dup2 pipe read end to stdin
        //   5. Execute command
        
        Err(Error::Unsupported)
    }
}

