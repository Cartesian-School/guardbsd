//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: serial
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Abstrakcja sprzętowa UART (16550).

// 16550 UART register offsets
const REG_DATA: u16 = 0;        // Data register (DLAB=0)
const REG_IER: u16 = 1;         // Interrupt Enable Register (DLAB=0)
const REG_FCR: u16 = 2;         // FIFO Control Register
const REG_LCR: u16 = 3;         // Line Control Register
const REG_MCR: u16 = 4;         // Modem Control Register
const REG_LSR: u16 = 5;         // Line Status Register
const REG_DLL: u16 = 0;         // Divisor Latch Low (DLAB=1)
const REG_DLH: u16 = 1;         // Divisor Latch High (DLAB=1)

// Line Status Register bits
const LSR_DATA_READY: u8 = 0x01;
const LSR_OVERRUN_ERROR: u8 = 0x02;
const LSR_PARITY_ERROR: u8 = 0x04;
const LSR_FRAMING_ERROR: u8 = 0x08;
const LSR_BREAK_INTERRUPT: u8 = 0x10;
const LSR_THR_EMPTY: u8 = 0x20;
const LSR_TRANSMITTER_EMPTY: u8 = 0x40;

// Line Control Register bits
const LCR_DLAB: u8 = 0x80;
const LCR_8N1: u8 = 0x03;  // 8 bits, no parity, 1 stop bit

// FIFO Control Register bits
const FCR_ENABLE_FIFO: u8 = 0x01;
const FCR_CLEAR_RX: u8 = 0x02;
const FCR_CLEAR_TX: u8 = 0x04;
const FCR_TRIGGER_14: u8 = 0xC0;

// Standard UART base addresses
pub const COM1_BASE: u16 = 0x3F8;
pub const COM2_BASE: u16 = 0x2F8;
pub const COM3_BASE: u16 = 0x3E8;
pub const COM4_BASE: u16 = 0x2E8;

// Baud rate divisors for 115200 base clock
const BAUD_115200: u16 = 1;
const BAUD_57600: u16 = 2;
const BAUD_38400: u16 = 3;
const BAUD_9600: u16 = 12;

const TIMEOUT_ITERATIONS: u32 = 100_000;

pub struct Uart {
    base: u16,
}

impl Uart {
    /// Create new UART instance with the given base address.
    /// Common values: COM1_BASE (0x3F8), COM2_BASE (0x2F8)
    pub const fn new(base: u16) -> Self {
        Self { base }
    }

    /// Initialize UART with 38400 baud, 8N1, FIFO enabled
    pub fn init(&mut self) {
        unsafe {
            // Disable all interrupts
            outb(self.base + REG_IER, 0x00);

            // Enable DLAB to set baud rate
            outb(self.base + REG_LCR, LCR_DLAB);

            // Set divisor to 3 (38400 baud)
            outb(self.base + REG_DLL, BAUD_38400 as u8);
            outb(self.base + REG_DLH, (BAUD_38400 >> 8) as u8);

            // Disable DLAB, set 8N1
            outb(self.base + REG_LCR, LCR_8N1);

            // Enable and clear FIFO (14-byte threshold)
            outb(
                self.base + REG_FCR,
                FCR_ENABLE_FIFO | FCR_CLEAR_RX | FCR_CLEAR_TX | FCR_TRIGGER_14,
            );

            // Enable RTS/DTR
            outb(self.base + REG_MCR, 0x03);

            // Enable receive interrupts only
            outb(self.base + REG_IER, 0x01);
        }
    }

    /// Write a single byte. Returns false if timeout occurred.
    pub fn write_byte(&self, byte: u8) -> bool {
        unsafe {
            // Wait for transmit buffer empty with timeout
            let mut timeout = TIMEOUT_ITERATIONS;
            while (inb(self.base + REG_LSR) & LSR_THR_EMPTY) == 0 {
                timeout -= 1;
                if timeout == 0 {
                    return false; // Timeout
                }
            }
            outb(self.base + REG_DATA, byte);
            true
        }
    }

    /// Read a byte if available. Returns None if no data or error occurred.
    pub fn read_byte(&self) -> Option<u8> {
        unsafe {
            let lsr = inb(self.base + REG_LSR);

            // Check for errors
            if (lsr & (LSR_OVERRUN_ERROR | LSR_PARITY_ERROR | LSR_FRAMING_ERROR)) != 0 {
                // Clear error by reading data register
                let _ = inb(self.base + REG_DATA);
                return None;
            }

            // Check if data available
            if (lsr & LSR_DATA_READY) != 0 {
                Some(inb(self.base + REG_DATA))
            } else {
                None
            }
        }
    }

    /// Write multiple bytes. Returns number of bytes successfully written.
    pub fn write(&self, data: &[u8]) -> usize {
        let mut count = 0;
        for &byte in data {
            if !self.write_byte(byte) {
                break; // Stop on timeout
            }
            count += 1;
        }
        count
    }

    /// Check if transmitter is idle (all data sent)
    pub fn is_transmit_idle(&self) -> bool {
        unsafe { (inb(self.base + REG_LSR) & LSR_TRANSMITTER_EMPTY) != 0 }
    }

    /// Check if data is available to read
    pub fn is_data_available(&self) -> bool {
        unsafe { (inb(self.base + REG_LSR) & LSR_DATA_READY) != 0 }
    }
}

#[cfg(target_arch = "x86_64")]
unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") val,
        options(nomem, nostack, preserves_flags)
    );
}

#[cfg(target_arch = "x86_64")]
unsafe fn inb(port: u16) -> u8 {
    let ret: u8;
    core::arch::asm!(
        "in al, dx",
        in("dx") port,
        out("al") ret,
        options(nomem, nostack, preserves_flags)
    );
    ret
}

#[cfg(not(target_arch = "x86_64"))]
unsafe fn outb(_port: u16, _val: u8) {
    // Not implemented for non-x86 architectures
    // Use memory-mapped I/O for ARM/RISC-V
    unimplemented!("UART port I/O not available on this architecture");
}

#[cfg(not(target_arch = "x86_64"))]
unsafe fn inb(_port: u16) -> u8 {
    // Not implemented for non-x86 architectures
    unimplemented!("UART port I/O not available on this architecture");
}