//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: boot_stub
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Obsługa klawiatury (PS/2) dla boot stuba.

pub mod ps2;

pub fn handle_interrupt() {
    ps2::handle_interrupt();
}

pub fn read_char() -> Option<u8> {
    ps2::read_char()
}
