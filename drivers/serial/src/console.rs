//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: serial
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Operacje I/O konsoli oparte na UART.

use crate::uart::Uart;

const RX_BUFFER_SIZE: usize = 256;
const ISR_MAX_ITERATIONS: usize = 16; // Limit iterations in ISR

pub struct Console {
    uart: Uart,
    rx_buf: [u8; RX_BUFFER_SIZE],
    rx_head: usize,
    rx_tail: usize,
    overflow_count: usize,
}

impl Console {
    pub const fn new(uart: Uart) -> Self {
        Self {
            uart,
            rx_buf: [0; RX_BUFFER_SIZE],
            rx_head: 0,
            rx_tail: 0,
            overflow_count: 0,
        }
    }

    pub fn init(&mut self) {
        self.uart.init();
    }

    /// Write data to console. Returns number of bytes successfully written.
    pub fn write(&mut self, data: &[u8]) -> usize {
        self.uart.write(data)
    }

    /// Non-blocking read from console buffer.
    /// Returns number of bytes read (may be 0 if no data available).
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        if buf.is_empty() {
            return 0;
        }

        let mut count = 0;

        // Read from circular buffer first
        while count < buf.len() && self.rx_head != self.rx_tail {
            buf[count] = self.rx_buf[self.rx_tail];
            self.rx_tail = (self.rx_tail + 1) % RX_BUFFER_SIZE;
            count += 1;
        }

        // Try to read directly from UART if buffer was empty
        while count < buf.len() {
            if let Some(byte) = self.uart.read_byte() {
                buf[count] = byte;
                count += 1;
            } else {
                break;
            }
        }

        count
    }

    /// Handle UART interrupt - should be called from interrupt handler.
    /// Reads available bytes into circular buffer.
    /// Limited to ISR_MAX_ITERATIONS to prevent long ISR execution.
    pub fn handle_interrupt(&mut self) {
        let mut iterations = 0;

        while let Some(byte) = self.uart.read_byte() {
            iterations += 1;
            if iterations >= ISR_MAX_ITERATIONS {
                break; // Prevent ISR from running too long
            }

            let next_head = (self.rx_head + 1) % RX_BUFFER_SIZE;

            if next_head != self.rx_tail {
                // Buffer has space
                self.rx_buf[self.rx_head] = byte;
                self.rx_head = next_head;
            } else {
                // Buffer overflow - data lost
                self.overflow_count = self.overflow_count.saturating_add(1);
                break; // Stop reading to prevent losing more data
            }
        }
    }

    /// Returns number of bytes available to read.
    pub fn available(&self) -> usize {
        if self.rx_head >= self.rx_tail {
            self.rx_head - self.rx_tail
        } else {
            RX_BUFFER_SIZE - self.rx_tail + self.rx_head
        }
    }

    /// Returns number of buffer overflow events (data loss).
    pub fn overflow_count(&self) -> usize {
        self.overflow_count
    }

    /// Clears the receive buffer and resets overflow counter.
    pub fn clear(&mut self) {
        self.rx_head = 0;
        self.rx_tail = 0;
    }

    /// Resets the overflow counter.
    pub fn reset_overflow_count(&mut self) {
        self.overflow_count = 0;
    }

    /// Check if receive buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.rx_head == self.rx_tail
    }

    /// Check if receive buffer is full.
    pub fn is_full(&self) -> bool {
        (self.rx_head + 1) % RX_BUFFER_SIZE == self.rx_tail
    }

    /// Get the maximum capacity of the receive buffer.
    pub fn capacity(&self) -> usize {
        RX_BUFFER_SIZE - 1 // One slot reserved for full/empty distinction
    }
}
