//! kernel/arch/riscv64/uart_irq.rs
//! Minimal NS16550A/8250 RX interrupt enable + drain for QEMU virt.
//!
//! NOTE: We use byte-wide registers at base 0x1000_0000 for QEMU virt UART0.

#![allow(dead_code)]

use core::ptr::{read_volatile, write_volatile};

pub const UART0_BASE: usize = 0x1000_0000;

// Register offsets (byte)
const RBR_THR: usize = 0x00; // RBR read / THR write
const IER: usize = 0x01;     // Interrupt Enable
const IIR_FCR: usize = 0x02; // IIR read / FCR write
const LSR: usize = 0x05;     // Line Status

#[inline(always)]
fn mmio_read8(addr: usize) -> u8 {
    unsafe { read_volatile(addr as *const u8) }
}

#[inline(always)]
fn mmio_write8(addr: usize, v: u8) {
    unsafe { write_volatile(addr as *mut u8, v) }
}

/// Initialize UART interrupts for RX:
/// - enable FIFO + clear FIFO (FCR)
/// - enable Received Data Available IRQ (IER bit0)
pub fn init_rx_irq_mode() {
    // Enable FIFO (bit0) + clear RX/TX FIFOs (bit1+bit2)
    mmio_write8(UART0_BASE + IIR_FCR, 0x07);

    // Enable RX available interrupt
    mmio_write8(UART0_BASE + IER, 0x01);

    // Best-effort: drain any pending data
    let _ = drain_rx();
}

/// Drain RX bytes to clear the interrupt source.
/// Returns drained byte count.
pub fn drain_rx() -> usize {
    let mut n = 0usize;

    // LSR bit0 = Data Ready
    while (mmio_read8(UART0_BASE + LSR) & 0x01) != 0 {
        let _b = mmio_read8(UART0_BASE + RBR_THR);
        n += 1;
    }

    n
}

/// Small sanity readback for debugging (optional).
pub fn read_ier() -> u8 {
    mmio_read8(UART0_BASE + IER)
}
