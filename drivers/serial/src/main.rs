// drivers/serial/src/main.rs
// GuardBSD Serial Driver
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#![no_std]
#![no_main]

use gbsd::*;

mod console;
mod uart;

use console::Console;
use uart::Uart;

const UART_BASE: u16 = 0x3F8; // COM1
const SERIAL_MAJOR: u16 = 10;
const SERIAL_MINOR: u16 = 0;

static mut CONSOLE: Console = Console::new(Uart::new(UART_BASE));

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_main()
}

fn serial_main() -> ! {
    unsafe {
        CONSOLE.init();
    }

    // Register as character device
    let dev_id = match dev_register(DEV_CHAR, SERIAL_MAJOR, SERIAL_MINOR) {
        Ok(id) => id,
        Err(_) => exit(1),
    };

    let port = match port_create() {
        Ok(p) => p,
        Err(_) => exit(2),
    };

    let mut req_buf = [0u8; 512];
    let mut resp_buf = [0u8; 512];

    loop {
        // Handle interrupts
        unsafe {
            CONSOLE.handle_interrupt();
        }

        // Handle IPC requests
        if port_receive(port, req_buf.as_mut_ptr(), req_buf.len()).is_ok() {
            if req_buf.len() >= 4 {
                let op = u32::from_le_bytes([req_buf[0], req_buf[1], req_buf[2], req_buf[3]]);
                let result = handle_request(op, &req_buf[4..], &mut resp_buf[8..]);

                resp_buf[0..8].copy_from_slice(&result.to_le_bytes());
                let _ = port_send(port, resp_buf.as_ptr(), resp_buf.len());
            }
        }

        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("pause", options(nomem, nostack));
        }

        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("yield", options(nomem, nostack));
        }
    }
}

fn handle_request(op: u32, data: &[u8], resp_data: &mut [u8]) -> i64 {
    unsafe {
        match op {
            1 => {
                // Write
                if data.len() >= 4 {
                    let len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
                    let write_data = &data[4..4 + len.min(data.len() - 4)];
                    CONSOLE.write(write_data) as i64
                } else {
                    -22 // EINVAL
                }
            }
            2 => {
                // Read
                if data.len() >= 4 {
                    let len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
                    let read_len = len.min(resp_data.len());
                    CONSOLE.read(&mut resp_data[..read_len]) as i64
                } else {
                    -22 // EINVAL
                }
            }
            _ => -38, // ENOSYS
        }
    }
}
