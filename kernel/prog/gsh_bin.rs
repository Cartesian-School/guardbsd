#![no_std]

// Placeholder flat binary for /bin/gsh.
// It only needs to be non-empty so the loader works.
// Later this can be replaced with a real userland shell image.
pub static GSH_BIN: &[u8] = &[0xCC]; // single INT3 byte, just a stub
