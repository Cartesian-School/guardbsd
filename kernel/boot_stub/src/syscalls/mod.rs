//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: boot_stub
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Minimalne stuby wywołań systemowych potrzebne do zbudowania boot stuba.

pub mod process {
    pub fn sys_exit(_code: i32) -> isize {
        loop {}
    }
    pub fn sys_getpid() -> isize {
        1
    }
    pub fn sys_fork() -> isize {
        -38
    }
    pub fn sys_exec(_path: *const u8, _argv: *const *const u8) -> isize {
        -38
    }
    pub fn sys_wait(_status: *mut i32) -> isize {
        -38
    }
    pub fn sys_waitpid(_pid: isize, _status: *mut i32, _opts: i32) -> isize {
        -38
    }
}

pub mod signal {
    pub fn sys_kill(_pid: usize, _sig: i32) -> isize {
        -38
    }
    pub fn sys_signal(_sig: i32, _handler: u64) -> isize {
        -38
    }
    pub fn sys_sigaction(
        _signum: i32,
        _act: *const core::ffi::c_void,
        _oldact: *mut core::ffi::c_void,
    ) -> isize {
        -38
    }
    pub fn sys_sigreturn() -> isize {
        -38
    }
}

pub mod fs {
    pub fn sys_write(_fd: u32, _buf: *const u8, _len: usize) -> isize {
        -38
    }
    pub fn sys_read(_fd: u32, _buf: *mut u8, _len: usize) -> isize {
        -38
    }
    pub fn sys_open(_path: *const u8, _flags: u32) -> isize {
        -38
    }
    pub fn sys_close(_fd: u32) -> isize {
        -38
    }
    pub fn sys_dup(_fd: usize) -> isize {
        -38
    }
    pub fn sys_dup2(_old: usize, _new: usize) -> isize {
        -38
    }
    pub fn sys_stat(_path: *const u8, _buf: *mut u8) -> isize {
        -38
    }
    pub fn sys_console_read(_buf: *mut u8, _len: usize) -> isize {
        -38
    }
    pub fn sys_tcsetpgrp(_fd: u32, _pgid: usize) -> isize {
        -38
    }
    pub fn sys_tcgetpgrp(_fd: u32) -> isize {
        -38
    }
}

pub mod log {
    pub fn sys_log_read(_buf: *mut u8, _len: usize) -> isize {
        -38
    }
}

pub mod process_jobctl {
    pub fn sys_kill(_pid: isize, _sig: i32) -> isize {
        -38
    }
    pub fn sys_waitpid(_pid: isize, _status: *mut i32, _opts: u32) -> isize {
        -38
    }
    pub fn sys_setpgid(_pid: usize, _pgid: usize) -> isize {
        -38
    }
    pub fn sys_getpgid(_pid: usize) -> isize {
        -38
    }
    pub fn find_process_by_pid(_pid: usize) -> Option<usize> {
        None
    }
    pub fn find_process_mut_for_signal(_pid: usize) -> Option<usize> {
        None
    }
}
