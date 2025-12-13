//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_process
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Moduł procesów (typy, loader ELF, zarządzanie procesami).

#![no_std]

pub mod types;
pub mod elf_loader;
pub mod process;

// Re-export canonical types
pub use types::{
    Process, Pid, ProcessState, FileDescriptor, SignalHandler,
    MAX_CHILDREN, MAX_FD_PER_PROCESS, MAX_SIGNALS,
    SIG_DFL, SIG_IGN,
    SIGHUP, SIGINT, SIGQUIT, SIGILL, SIGTRAP, SIGABRT, SIGBUS, SIGFPE,
    SIGKILL, SIGUSR1, SIGSEGV, SIGUSR2, SIGPIPE, SIGALRM, SIGTERM,
    SIGCHLD, SIGCONT, SIGSTOP, SIGTSTP, SIGTTIN, SIGTTOU,
};

// Re-export process management functions
pub use process::{create_process, exec, schedule, switch_to, get_current, allocate_pid};
