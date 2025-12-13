//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: boot_stub
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Moduł timerów (PIT) dla boot stuba.

pub mod pit;

static mut TICKS: u64 = 0;

pub fn init(hz: u32) {
    pit::init(hz);
}

pub fn handle_interrupt() {
    unsafe {
        TICKS += 1;
    }
}

pub fn get_ticks() -> u64 {
    unsafe { TICKS }
}
