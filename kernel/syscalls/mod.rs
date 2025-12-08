// Syscall Implementations Module
// BSD 3-Clause License

#![no_std]

pub mod process;
pub mod signal;
pub mod sched;
pub mod fs;

// Re-export main syscall functions
pub use process::{sys_exit, sys_getpid, sys_fork, sys_exec, sys_wait};
pub use fs::{sys_open, sys_read, sys_write, sys_close};

