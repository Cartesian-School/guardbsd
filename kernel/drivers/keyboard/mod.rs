//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_drivers
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Moduł sterownika klawiatury.

pub mod ps2;

pub fn init() {
    ps2::init();
}

pub fn handle_interrupt() {
    ps2::handle_interrupt();
}

pub fn read_char() -> Option<u8> {
    ps2::read_char()
}

pub fn has_input() -> bool {
    ps2::has_input()
}
