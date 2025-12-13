//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: shared
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Wspólne definicje GuardBSD (syscall numbers, typy itp.).

#![no_std]

pub mod syscall_numbers {
    include!("syscall_numbers.rs");
}

pub use syscall_numbers::*;
