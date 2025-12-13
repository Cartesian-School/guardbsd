//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_arch_aarch64
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Glue architektury AArch64: wektory wyjątków, timer i integracja schedulera.

#![cfg(target_arch = "aarch64")]
#![allow(dead_code)]

pub mod interrupts;
pub mod trap_frame;

/// Main entry point for AArch64 kernel bring-up.
#[no_mangle]
pub extern "C" fn kernel_main_aarch64() -> ! {
    interrupts::init_exceptions_and_timer();
    {
        use crate::sched::{spawn_kernel_thread_aarch64, start_first_thread};
        use crate::tests::preempt_threads_aarch64::{thread_a, thread_b, thread_c, thread_d};

        spawn_kernel_thread_aarch64(thread_a);
        spawn_kernel_thread_aarch64(thread_b);
        spawn_kernel_thread_aarch64(thread_c);
        spawn_kernel_thread_aarch64(thread_d);

        start_first_thread();
    }
    loop {
        unsafe { core::arch::asm!("wfe", options(nomem, nostack, preserves_flags)) };
    }
}
