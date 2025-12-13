//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_arch_aarch64
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Cienki wrapper na kanoniczną ramkę pułapki AArch64 w przestrzeni arch.

#![cfg(target_arch = "aarch64")]

// Re-export the canonical definition from kernel/trapframe.rs to avoid divergent layouts.
pub use crate::trapframe::TrapFrameAArch64;
