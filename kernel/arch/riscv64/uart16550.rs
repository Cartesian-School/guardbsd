//! kernel/arch/riscv64/uart16550.rs
//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_arch_riscv64
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//! Minimal 16550 UART driver for QEMU virt.
//! QEMU virt maps UART0 (NS16550A) at 0x1000_0000.

use core::ptr::{read_volatile, write_volatile};

const UART0_BASE: usize = 0x1000_0000;

// 16550 registers (offsets)
const RHR_THR: usize = 0x00; // Receive Holding / Transmit Holding
const LSR: usize = 0x05;     // Line Status Register

// LSR bits
const LSR_THRE: u8 = 1 << 5; // Transmit Holding Register Empty

pub struct Uart16550 {
    base: usize,
}

impl Uart16550 {
    pub const fn new(base: usize) -> Self {
        Self { base }
    }

    #[inline(always)]
    fn reg8(&self, off: usize) -> *mut u8 {
        (self.base + off) as *mut u8
    }

    pub fn putc(&self, ch: u8) {
        // Wait for THR empty
        while unsafe { read_volatile(self.reg8(LSR)) } & LSR_THRE == 0 {}
        unsafe { write_volatile(self.reg8(RHR_THR), ch) };
    }

    pub fn puts(&self, s: &str) {
        for &b in s.as_bytes() {
            if b == b'\n' {
                self.putc(b'\r');
            }
            self.putc(b);
        }
    }
}

pub fn uart0() -> Uart16550 {
    Uart16550::new(UART0_BASE)
}
