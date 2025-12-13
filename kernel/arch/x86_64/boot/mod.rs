//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_arch_x86_64
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Deklaracje wejścia long-mode dla x86_64.

#![cfg(target_arch = "x86_64")]

extern "C" {
    /// Assembly entry that transitions from 32-bit protected mode into 64-bit long mode.
    pub fn long_mode_entry();
}
