// userland/libgbsd/src/lib.rs
// GuardBSD System Library
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#![no_std]

pub mod syscall;
pub mod ipc;
pub mod error;
pub mod fs;
pub mod device;
pub mod log;
pub mod process;

pub use error::{Error, Result};
pub use syscall::*;
pub use ipc::*;
pub use fs::*;
pub use device::*;
pub use log::*;

// Re-export kernel logging macros for userland servers
pub use kernel_log::{klog_trace, klog_debug, klog_info, klog_warn, klog_error};

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    syscall::exit(1);
}

