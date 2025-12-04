// Program images for ETAP 3.2 (placeholders)
// In a future step, these will be populated with real user binaries.

#![no_std]

// Hardcoded entry addresses for placeholder blobs
pub const INIT_ENTRY: u64 = 0x400000;
pub const GSH_ENTRY: u64 = 0x500000;

// Placeholder program images (empty); replace with real include_bytes! when available
pub static INIT_BIN: &[u8] = &[];
pub static GSH_BIN: &[u8] = &[];

pub enum ProgKind {
    Init,
    Gsh,
}

pub fn lookup(path: &[u8]) -> Option<(ProgKind, u64, &'static [u8])> {
    if path == b"/bin/init" {
        Some((ProgKind::Init, INIT_ENTRY, INIT_BIN))
    } else if path == b"/bin/gsh" {
        Some((ProgKind::Gsh, GSH_ENTRY, GSH_BIN))
    } else {
        None
    }
}
