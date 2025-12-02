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

pub use error::{Error, Result};
pub use syscall::*;
pub use ipc::*;
pub use fs::*;
pub use device::*;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    syscall::exit(1);
}


