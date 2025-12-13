//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_proc
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Minimalny model PID.

use core::sync::atomic::{AtomicU32, Ordering};

/// The current process ID. Starts at 1 (the kernel/init process).
static CURRENT_PID: AtomicU32 = AtomicU32::new(1);

/// Initialize the boot task (PID 1).
pub fn init_boot_task() {
    CURRENT_PID.store(1, Ordering::Relaxed);
}

/// Return the current process ID.
pub fn current_pid() -> u32 {
    CURRENT_PID.load(Ordering::Relaxed)
}
