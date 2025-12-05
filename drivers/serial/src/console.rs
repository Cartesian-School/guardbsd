// drivers/serial/src/console.rs
// Console I/O operations
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::uart::Uart;

pub struct Console {
    uart: Uart,
    rx_buf: [u8; 256],
    rx_head: usize,
    rx_tail: usize,
}

impl Console {
    pub const fn new(uart: Uart) -> Self {
        Self {
            uart,
            rx_buf: [0; 256],
            rx_head: 0,
            rx_tail: 0,
        }
    }

    pub fn init(&self) {
        self.uart.init();
    }

    pub fn write(&self, data: &[u8]) -> usize {
        self.uart.write(data);
        data.len()
    }

    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let mut count = 0;

        // Read from buffer
        while count < buf.len() && self.rx_head != self.rx_tail {
            buf[count] = self.rx_buf[self.rx_tail];
            self.rx_tail = (self.rx_tail + 1) % 256;
            count += 1;
        }

        // Try to read from UART
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

    pub fn handle_interrupt(&mut self) {
        // Read available bytes into buffer
        while let Some(byte) = self.uart.read_byte() {
            let next_head = (self.rx_head + 1) % 256;
            if next_head != self.rx_tail {
                self.rx_buf[self.rx_head] = byte;
                self.rx_head = next_head;
            }
        }
    }
}
