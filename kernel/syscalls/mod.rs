// Syscall Implementations Module
// BSD 3-Clause License

#![no_std]

pub mod process;
pub mod process_jobctl;
pub mod signal;
pub mod sched;
pub mod fs;
pub mod log;

// Re-export main syscall functions
pub use process::{sys_exit, sys_getpid, sys_fork, sys_exec, sys_wait};
pub use process_jobctl::{sys_setpgid, sys_getpgid, sys_kill, sys_waitpid, check_pending_signals};
pub use fs::{sys_open, sys_read, sys_write, sys_close, sys_dup, sys_dup2, sys_stat, sys_mkdir, sys_unlink, sys_rename, sys_sync, sys_chdir, sys_getcwd, sys_mount, sys_umount, sys_console_read, sys_tcsetpgrp, sys_tcgetpgrp};
