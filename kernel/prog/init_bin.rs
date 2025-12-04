#![no_std]

// Temporary placeholder until flat init.bin exists.
// Use a single dummy byte so compilation always succeeds.
pub static INIT_BIN: &[u8] = &[0xCC]; // INT3 for debug

// User-mode entry address for flat binary (hardcoded)
pub const INIT_ENTRY: u64 = 0x400000;
