//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_drivers
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Moduł timera (PIT) w jądrze.

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
