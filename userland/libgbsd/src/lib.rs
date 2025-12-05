// userland/libgbsd/src/lib.rs
// GuardBSD System Library
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

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
