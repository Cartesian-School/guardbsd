//! kernel/boot_stub/src/interrupt/mod.rs
//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: boot_stub
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Interrupt subsystem (IDT + optional exception stubs) for boot stub.

#[cfg(all(target_arch = "x86_64", feature = "idt_exceptions"))]
core::arch::global_asm!(include_str!("exception_stubs.S"));

pub mod idt;

