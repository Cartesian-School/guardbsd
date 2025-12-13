//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: libgbsd
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Biblioteka systemowa GuardBSD (no_std).

#![no_std]

pub mod device;
pub mod error;
pub mod fs;
pub mod ipc;
pub mod log;
pub mod process;
pub mod syscall;

pub use device::*;
pub use error::{Error, Result};
pub use fs::*;
pub use ipc::*;
pub use log::*;
pub use syscall::*;

// Re-export kernel logging macros for userland servers
pub use kernel_log::{klog_debug, klog_error, klog_info, klog_trace, klog_warn};

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    syscall::exit(1);
}
