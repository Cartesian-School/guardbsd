//! kernel/arch/riscv64/clint.rs
//! Minimal CLINT access for QEMU virt (clint@0x0200_0000).
//!
//! We only need MTIME as a time source. The compare is done via stimecmp (Sstc)
//! so that the interrupt is delivered to S-mode.

#![allow(dead_code)]

use core::ptr::read_volatile;

pub const CLINT_BASE_QEMU_VIRT: usize = 0x0200_0000;

// SiFive/QEMU CLINT layout:
// - mtimecmp: base + 0x4000 + 8*hart (not used for S-mode timer here)
// - mtime:    base + 0xBFF8
const MTIME_OFFSET: usize = 0xBFF8;

#[inline(always)]
pub fn read_mtime(clint_base: usize) -> u64 {
    let p = (clint_base + MTIME_OFFSET) as *const u64;
    unsafe { read_volatile(p) }
}
