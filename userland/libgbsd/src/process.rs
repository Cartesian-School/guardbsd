// userland/libgbsd/src/process.rs
// Process management syscalls
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::syscall::{syscall0, syscall1, syscall2, syscall3};
use crate::{Error, Result};
use shared::syscall_numbers::*;

// Wait options
pub const WNOHANG: i32 = 1;

// Standard file descriptors
pub const STDIN_FILENO: i32 = 0;
pub const STDOUT_FILENO: i32 = 1;
pub const STDERR_FILENO: i32 = 2;

// Signal numbers
pub const SIGINT: i32 = 2;
pub const SIGTSTP: i32 = 20;
pub const SIGCHLD: i32 = 17;
pub const SIGCONT: i32 = 18;

/// Fork current process
pub fn fork() -> Result<i32> {
    let ret = unsafe { syscall0(SYS_FORK as u64) };
    // Check if high bit is set (negative as i64)
    if (ret as i64) < 0 {
        Err(Error::from_code(-(ret as i64) as i32))
    } else {
        Ok(ret as i32)
    }
}

/// Execute a new program
pub fn exec(path: *const u8, argv: *const *const u8) -> Result<()> {
    let ret = unsafe { syscall2(SYS_EXEC as u64, path as u64, argv as u64) };
    // Check if high bit is set (negative as i64)
    if (ret as i64) < 0 {
        Err(Error::from_code(-(ret as i64) as i32))
    } else {
        Ok(())
    }
}

/// Wait for child process
pub fn wait(status: *mut i32) -> Result<i32> {
    let ret = unsafe { syscall1(SYS_WAIT as u64, status as u64) };
    // Check if high bit is set (negative as i64)
    if (ret as i64) < 0 {
        Err(Error::from_code(-(ret as i64) as i32))
    } else {
        Ok(ret as i32)
    }
}

/// Wait for specific child process with options
/// Returns (pid, status) or None if WNOHANG and no child ready
pub fn waitpid(pid: i32, options: i32) -> Result<Option<(i32, i32)>> {
    let mut status: i32 = 0;
    let ret = unsafe {
        syscall3(
            SYS_WAITPID as u64,
            pid as i64 as u64,
            &mut status as *mut i32 as u64,
            options as u64,
        )
    };

    let ret_i = ret as i64;
    if ret_i < 0 {
        Err(Error::from_code((-ret_i) as i32))
    } else if ret_i == 0 {
        // WNOHANG and no child ready
        Ok(None)
    } else {
        Ok(Some((ret_i as i32, status)))
    }
}

/// Get current process ID
pub fn getpid() -> i32 {
    unsafe { syscall0(SYS_GETPID as u64) as i32 }
}

/// Exit current process
pub fn exit(status: i32) -> ! {
    unsafe {
        syscall1(SYS_EXIT as u64, status as u64);
    }
    // SAFETY: exit syscall never returns, but compiler doesn't know that
    #[allow(clippy::empty_loop)]
    loop {
        core::hint::spin_loop();
    }
}

/// Yield CPU to other processes
pub fn yield_cpu() {
    unsafe {
        syscall0(SYS_YIELD as u64);
    }
}

/// Set process group ID
pub fn setpgid(pid: i32, pgid: i32) -> Result<()> {
    let ret = unsafe { syscall2(SYS_SETPGID as u64, pid as u64, pgid as u64) };
    let ret_i = ret as i64;
    if ret_i < 0 {
        Err(Error::from_code((-ret_i) as i32))
    } else {
        Ok(())
    }
}

/// Get process group ID
pub fn getpgid(pid: i32) -> Result<i32> {
    let ret = unsafe { syscall1(SYS_GETPGID as u64, pid as u64) };
    let ret_i = ret as i64;
    if ret_i < 0 {
        Err(Error::from_code((-ret_i) as i32))
    } else {
        Ok(ret_i as i32)
    }
}

/// Send signal to process or process group
pub fn kill(pid: i32, sig: i32) -> Result<()> {
    let ret = unsafe { syscall2(SYS_KILL as u64, pid as i64 as u64, sig as u64) };
    let ret_i = ret as i64;
    if ret_i < 0 {
        Err(Error::from_code((-ret_i) as i32))
    } else {
        Ok(())
    }
}

/// Set foreground process group for TTY
pub fn tcsetpgrp(fd: i32, pgid: i32) -> Result<()> {
    let ret = unsafe { syscall2(SYS_TCSETPGRP as u64, fd as u64, pgid as u64) };
    let ret_i = ret as i64;
    if ret_i < 0 {
        Err(Error::from_code((-ret_i) as i32))
    } else {
        Ok(())
    }
}

/// Get foreground process group for TTY
pub fn tcgetpgrp(fd: i32) -> Result<i32> {
    let ret = unsafe { syscall1(SYS_TCGETPGRP as u64, fd as u64) };
    let ret_i = ret as i64;
    if ret_i < 0 {
        Err(Error::from_code((-ret_i) as i32))
    } else {
        Ok(ret_i as i32)
    }
}
