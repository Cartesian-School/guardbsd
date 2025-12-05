// shared/syscall_numbers.rs
// Canonical Syscall Number Definitions for GuardBSD
// BSD 3-Clause License
//
// This is the SINGLE source of truth for all syscall numbers.
// All kernel and userland code MUST use these constants.


// Process Management Syscalls
pub const SYS_EXIT: usize = 0;
pub const SYS_FORK: usize = 1;
pub const SYS_EXEC: usize = 2;
pub const SYS_WAIT: usize = 3;
pub const SYS_GETPID: usize = 4;
pub const SYS_KILL: usize = 5;
pub const SYS_YIELD: usize = 6;

// I/O Syscalls
pub const SYS_READ: usize = 10;
pub const SYS_WRITE: usize = 11;
pub const SYS_OPEN: usize = 12;
pub const SYS_CLOSE: usize = 13;

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

// Signal Syscalls
pub const SYS_SIGNAL: usize = 40;
pub const SYS_SIGACTION: usize = 41;
pub const SYS_SIGPROCMASK: usize = 42;
pub const SYS_SIGRETURN: usize = 43;

// Logging Syscalls
pub const SYS_LOG_READ: usize = 50;
pub const SYS_LOG_ACK: usize = 51;
pub const SYS_LOG_REGISTER_DAEMON: usize = 52;

// Error code for unimplemented syscalls
pub const ENOSYS: isize = -38;

