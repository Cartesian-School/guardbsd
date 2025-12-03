// Filesystem Module
// BSD 3-Clause License

#![no_std]

pub mod iso9660;

pub fn init() {
    // ISO is loaded at 0x10000000 (256MB) by bootloader
    iso9660::init(0x10000000);
}
