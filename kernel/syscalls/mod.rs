// Syscall Implementations Module
// BSD 3-Clause License

#![no_std]

pub mod process;
pub mod sched;

// Re-export main syscall functions
pub use process::{sys_exit, sys_getpid, sys_fork, sys_exec, sys_wait};

