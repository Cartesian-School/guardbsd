// drivers/serial/src/uart.rs
// UART hardware abstraction
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

// 16550 UART registers
const UART_BASE: u16 = 0x3F8; // COM1

const UART_DATA: u16 = UART_BASE;
const UART_IER: u16 = UART_BASE + 1;
const UART_LCR: u16 = UART_BASE + 3;
const UART_LSR: u16 = UART_BASE + 5;

pub struct Uart {
    base: u16,
}

impl Uart {
    pub const fn new(base: u16) -> Self {
        Self { base }
    }

    pub fn init(&self) {
        unsafe {
            // Disable interrupts
            outb(self.base + 1, 0x00);

            // Enable DLAB (set baud rate divisor)
            outb(self.base + 3, 0x80);

            // Set divisor to 3 (38400 baud)
            outb(self.base, 0x03);
            outb(self.base + 1, 0x00);

            // 8 bits, no parity, one stop bit
            outb(self.base + 3, 0x03);

            // Enable FIFO
            outb(self.base + 2, 0xC7);

            // Enable interrupts
            outb(self.base + 1, 0x01);
        }
    }

    pub fn write_byte(&self, byte: u8) {
        unsafe {
            // Wait for transmit buffer empty
            while (inb(self.base + 5) & 0x20) == 0 {}
            outb(self.base, byte);
        }
    }

    pub fn read_byte(&self) -> Option<u8> {
        unsafe {
            // Check if data available
            if (inb(self.base + 5) & 0x01) != 0 {
                Some(inb(self.base))
            } else {
                None
            }
        }
    }

    pub fn write(&self, data: &[u8]) {
        for &byte in data {
            self.write_byte(byte);
        }
    }
}

#[cfg(target_arch = "x86_64")]
unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") val,
        options(nomem, nostack)
    );
}

#[cfg(target_arch = "x86_64")]
unsafe fn inb(port: u16) -> u8 {
    let ret: u8;
    core::arch::asm!(
        "in al, dx",
        in("dx") port,
        out("al") ret,
        options(nomem, nostack)
    );
    ret
}

#[cfg(target_arch = "aarch64")]
unsafe fn outb(_port: u16, _val: u8) {
    // ARM uses memory-mapped I/O
}

#[cfg(target_arch = "aarch64")]
unsafe fn inb(_port: u16) -> u8 {
    // ARM uses memory-mapped I/O
    0
}
