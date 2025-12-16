//! kernel/arch/riscv64/plic.rs
//! Minimal PLIC for QEMU virt (hart0 S-mode context = 1).
//!
//! QEMU virt wiring commonly uses:
//! - UART0 external interrupt source: IRQ 10
//! - VirtIO MMIO sources: IRQ 1..8
//! (See QEMU virt wiring references.) :contentReference[oaicite:2]{index=2}

#![allow(dead_code)]

use core::ptr::{read_volatile, write_volatile};

pub const PLIC_BASE_QEMU_VIRT: usize = 0x0c00_0000;

// PLIC layout (SiFive / QEMU virt)
const PRIORITY_BASE: usize = 0x0000; // u32 priority per irq
const ENABLE_BASE: usize = 0x2000;   // enable bits per context
const ENABLE_STRIDE: usize = 0x80;   // bytes per context enable bank
const CONTEXT_BASE: usize = 0x200000;
const CONTEXT_STRIDE: usize = 0x1000;

// QEMU virt: context 1 == hart0 S-mode (odd contexts are S-mode) :contentReference[oaicite:3]{index=3}
const CTX_SMODE_HART0: usize = 1;

#[inline(always)]
fn mmio_read32(addr: usize) -> u32 {
    unsafe { read_volatile(addr as *const u32) }
}

#[inline(always)]
fn mmio_write32(addr: usize, v: u32) {
    unsafe { write_volatile(addr as *mut u32, v) }
}

#[inline(always)]
fn priority_addr(irq: u32) -> usize {
    PLIC_BASE_QEMU_VIRT + PRIORITY_BASE + (irq as usize) * 4
}

#[inline(always)]
fn enable_word_addr(context: usize, word_index: usize) -> usize {
    // Each word covers 32 IRQs
    PLIC_BASE_QEMU_VIRT + ENABLE_BASE + context * ENABLE_STRIDE + word_index * 4
}

#[inline(always)]
fn threshold_addr(context: usize) -> usize {
    PLIC_BASE_QEMU_VIRT + CONTEXT_BASE + context * CONTEXT_STRIDE + 0x0
}

#[inline(always)]
fn claim_complete_addr(context: usize) -> usize {
    PLIC_BASE_QEMU_VIRT + CONTEXT_BASE + context * CONTEXT_STRIDE + 0x4
}

/// Enable a single IRQ for hart0 S-mode:
/// - set priority(irq)=1
/// - set enable bit in S-mode context
/// - set threshold=0
pub fn init_smode_hart0_enable_irq(irq: u32) {
    // Priority: 0 disables, >0 enables
    mmio_write32(priority_addr(irq), 1);

    // Enable bit
    let word_index = (irq / 32) as usize;
    let bit = irq % 32;

    let addr = enable_word_addr(CTX_SMODE_HART0, word_index);
    let prev = mmio_read32(addr);
    mmio_write32(addr, prev | (1u32 << bit));

    // Accept all priorities > 0
    mmio_write32(threshold_addr(CTX_SMODE_HART0), 0);
}

/// Claim next pending IRQ (0 means none).
#[inline(always)]
pub fn claim() -> u32 {
    mmio_read32(claim_complete_addr(CTX_SMODE_HART0))
}

/// Complete IRQ.
#[inline(always)]
pub fn complete(irq: u32) {
    mmio_write32(claim_complete_addr(CTX_SMODE_HART0), irq);
}
