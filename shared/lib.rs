// shared/lib.rs
// Shared GuardBSD definitions
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#![no_std]

pub mod syscall_numbers {
    // Core Process Management Syscalls
    pub const SYS_EXIT: usize = 0;
    pub const SYS_FORK: usize = 1;
    pub const SYS_EXEC: usize = 2;
    pub const SYS_WAIT: usize = 3;
    pub const SYS_GETPID: usize = 4;
    pub const SYS_YIELD: usize = 5;
    pub const SYS_SLEEP: usize = 6;

    // File System Syscalls
    pub const SYS_OPEN: usize = 10;
    pub const SYS_CLOSE: usize = 11;
    pub const SYS_READ: usize = 12;
    pub const SYS_WRITE: usize = 13;
    pub const SYS_STAT: usize = 14;
    pub const SYS_MKDIR: usize = 15;
    pub const SYS_UNLINK: usize = 16;
    pub const SYS_RENAME: usize = 17;
    pub const SYS_SYNC: usize = 18;
    pub const SYS_CHDIR: usize = 19;
    pub const SYS_GETCWD: usize = 20;
    pub const SYS_MOUNT: usize = 21;
    pub const SYS_UMOUNT: usize = 22;

    // IPC Syscalls
    pub const SYS_IPC_PORT_CREATE: usize = 30;
    pub const SYS_IPC_SEND: usize = 31;
    pub const SYS_IPC_RECV: usize = 32;

    // Logging Syscalls
    pub const SYS_LOG_READ: usize = 40;
    pub const SYS_LOG_ACK: usize = 41;
    pub const SYS_LOG_REGISTER_DAEMON: usize = 42;

    // Signal Syscalls
    pub const SYS_KILL: usize = 50;
    pub const SYS_SIGNAL: usize = 51;
    pub const SYS_SIGACTION: usize = 52;
    pub const SYS_SIGPROCMASK: usize = 53;
    pub const SYS_SIGRETURN: usize = 54;
}
