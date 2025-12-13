//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_fs
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Moduł systemu plików w jądrze (ISO9660).

#![no_std]

pub mod iso9660;

pub fn init() {
    // ISO is loaded at 0x10000000 (256MB) by bootloader
    iso9660::init(0x10000000);
}
