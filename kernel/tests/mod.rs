//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_tests
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Testowe wątki kernelowe do sprawdzania preempcji.

#[cfg(target_arch = "x86_64")]
pub mod preempt_threads;

#[cfg(target_arch = "aarch64")]
pub mod preempt_threads_aarch64;
