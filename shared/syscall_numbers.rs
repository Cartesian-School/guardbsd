// Project: GuardBSD Winter Saga version 1.0.0
// Package: shared
// Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
// License: BSD-3-Clause
//
// Kanoniczne numery wywołań systemowych GuardBSD (źródło prawdy dla kernela i userlandu).


// Process Management Syscalls
pub const SYS_EXIT: usize = 0;
pub const SYS_FORK: usize = 1;
pub const SYS_EXEC: usize = 2;
pub const SYS_WAIT: usize = 3;
pub const SYS_GETPID: usize = 4;
pub const SYS_KILL: usize = 5;
pub const SYS_YIELD: usize = 6;
pub const SYS_WAITPID: usize = 7;
pub const SYS_SETPGID: usize = 8;
pub const SYS_GETPGID: usize = 9;

// I/O Syscalls
pub const SYS_READ: usize = 10;
pub const SYS_WRITE: usize = 11;
pub const SYS_OPEN: usize = 12;
pub const SYS_CLOSE: usize = 13;
pub const SYS_DUP: usize = 14;
pub const SYS_DUP2: usize = 15;

// Filesystem Syscalls
pub const SYS_STAT: usize = 20;
pub const SYS_MKDIR: usize = 21;
pub const SYS_UNLINK: usize = 22;
pub const SYS_RENAME: usize = 23;
pub const SYS_CHDIR: usize = 24;
pub const SYS_GETCWD: usize = 25;
pub const SYS_MOUNT: usize = 26;
pub const SYS_UMOUNT: usize = 27;
pub const SYS_SYNC: usize = 28;

// IPC Syscalls
pub const SYS_IPC_PORT_CREATE: usize = 30;
pub const SYS_IPC_SEND: usize = 31;
pub const SYS_IPC_RECV: usize = 32;

// Console/TTY Syscalls
pub const SYS_CONSOLE_READ: usize = 40;
pub const SYS_TCSETPGRP: usize = 41;
pub const SYS_TCGETPGRP: usize = 42;

// Signal Syscalls
pub const SYS_SIGNAL: usize = 40;
pub const SYS_SIGACTION: usize = 41;
pub const SYS_SIGPROCMASK: usize = 42;
pub const SYS_SIGRETURN: usize = 43;
pub const SYS_SIGNAL_REGISTER: usize = 44; // alias for signal(int, handler)

// Logging Syscalls
pub const SYS_LOG_READ: usize = 50;
pub const SYS_LOG_ACK: usize = 51;
pub const SYS_LOG_REGISTER_DAEMON: usize = 52;

// Service Registry
pub const SYS_SERVICE_REGISTER: usize = 60;

// Error code for unimplemented syscalls
pub const ENOSYS: isize = -38;
