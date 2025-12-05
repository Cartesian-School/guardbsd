// userland/libgbsd/src/process.rs
// Process management syscalls
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::{Error, Result};
use crate::syscall::{syscall0, syscall1, syscall2};
use shared::syscall_numbers::*;

/// Fork current process
pub fn fork() -> Result<i32> {
    let ret = unsafe { syscall0(SYS_FORK as u64) };
    if ret < 0 {
        Err(Error::from_code(-(ret as i32)))
    } else {
        Ok(ret as i32)
    }
}

/// Execute a new program
pub fn exec(path: *const u8, argv: *const *const u8) -> Result<()> {
    let ret = unsafe { syscall2(SYS_EXEC as u64, path as u64, argv as u64) };
    if ret < 0 {
        Err(Error::from_code(-(ret as i32)))
    } else {
        Ok(())
    }
}

/// Wait for child process
pub fn wait(status: *mut i32) -> Result<i32> {
    let ret = unsafe { syscall1(SYS_WAIT as u64, status as u64) };
    if ret < 0 {
        Err(Error::from_code(-(ret as i32)))
    } else {
        Ok(ret as i32)
    }
}

/// Wait for specific child process
pub fn waitpid(_pid: i32, status: *mut i32) -> Result<i32> {
    // For now, just use wait() and ignore PID
    // TODO: Add SYS_WAITPID syscall
    wait(status)
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
    loop {}
}

/// Yield CPU to other processes
pub fn yield_cpu() {
    unsafe {
        syscall0(SYS_YIELD as u64);
    }
}

