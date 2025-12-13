//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: serial
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Sterownik portu szeregowego GuardBSD.

#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};
use gbsd::*;

mod console;
mod uart;

use console::Console;
use uart::Uart;

const UART_BASE: u16 = 0x3F8; // COM1
const SERIAL_MAJOR: u16 = 10;
const SERIAL_MINOR: u16 = 0;

const MAX_REQUEST_SIZE: usize = 4096;
const MAX_RESPONSE_SIZE: usize = 4096;

// Operation codes
const OP_WRITE: u32 = 1;
const OP_READ: u32 = 2;
const OP_AVAILABLE: u32 = 3;
const OP_FLUSH: u32 = 4;

// Use a mutex or lock instead of static mut
// For now, using static mut with careful access patterns
static mut CONSOLE: Option<Console> = None;
static INITIALIZED: AtomicBool = AtomicBool::new(false);

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_main()
}

fn serial_main() -> ! {
    // Initialize console
    unsafe {
        CONSOLE = Some(Console::new(Uart::new(UART_BASE)));
        if let Some(ref mut console) = CONSOLE {
            console.init();
        }
    }
    INITIALIZED.store(true, Ordering::Release);

    // Register as character device
    let dev_id = match dev_register(DEV_CHAR, SERIAL_MAJOR, SERIAL_MINOR) {
        Ok(id) => id,
        Err(e) => {
            // Log error if possible
            exit(1);
        }
    };

    let port = match port_create() {
        Ok(p) => p,
        Err(e) => {
            exit(2);
        }
    };

    let mut req_buf = [0u8; MAX_REQUEST_SIZE];
    let mut resp_buf = [0u8; MAX_RESPONSE_SIZE];

    loop {
        // Blocking receive - no busy-wait
        match port_receive(port, req_buf.as_mut_ptr(), req_buf.len()) {
            Ok(received_len) => {
                if received_len < 4 {
                    // Invalid request - too small
                    let error = (-22i64).to_le_bytes(); // EINVAL
                    resp_buf[0..8].copy_from_slice(&error);
                    let _ = port_send(port, resp_buf.as_ptr(), 8);
                    continue;
                }

                let op = u32::from_le_bytes([req_buf[0], req_buf[1], req_buf[2], req_buf[3]]);

                let (result, data_len) =
                    handle_request(op, &req_buf[4..received_len], &mut resp_buf[8..]);

                // Write result code (8 bytes) + data
                resp_buf[0..8].copy_from_slice(&result.to_le_bytes());
                let total_len = 8 + data_len;
                let _ = port_send(port, resp_buf.as_ptr(), total_len);
            }
            Err(_) => {
                // Port error - continue or handle appropriately
                continue;
            }
        }
    }
}

/// Handle a serial port request.
/// Returns (result_code, response_data_length)
fn handle_request(op: u32, data: &[u8], resp_data: &mut [u8]) -> (i64, usize) {
    if !INITIALIZED.load(Ordering::Acquire) {
        return (-5, 0); // EIO - not initialized
    }

    unsafe {
        if let Some(ref mut console) = CONSOLE {
            match op {
                OP_WRITE => {
                    // Write: [4 bytes len][data...]
                    if data.len() < 4 {
                        return (-22, 0); // EINVAL
                    }

                    let len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;

                    // Validate length
                    if len > data.len() - 4 || len > MAX_REQUEST_SIZE - 4 {
                        return (-22, 0); // EINVAL
                    }

                    let write_data = &data[4..4 + len];
                    let written = console.write(write_data);
                    (written as i64, 0)
                }

                OP_READ => {
                    // Read: [4 bytes max_len]
                    if data.len() < 4 {
                        return (-22, 0); // EINVAL
                    }

                    let max_len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;

                    // Validate and limit read length
                    let read_len = max_len.min(resp_data.len()).min(MAX_RESPONSE_SIZE - 8);
                    if read_len == 0 {
                        return (0, 0);
                    }

                    let bytes_read = console.read(&mut resp_data[..read_len]);
                    (bytes_read as i64, bytes_read)
                }

                OP_AVAILABLE => {
                    // Get number of bytes available
                    let available = console.available();
                    resp_data[0..4].copy_from_slice(&(available as u32).to_le_bytes());
                    (0, 4)
                }

                OP_FLUSH => {
                    // Clear receive buffer
                    console.clear();
                    (0, 0)
                }

                _ => (-38, 0), // ENOSYS - not implemented
            }
        } else {
            (-5, 0) // EIO
        }
    }
}

/// Interrupt handler - should be called from actual ISR
#[no_mangle]
pub extern "C" fn serial_interrupt_handler() {
    if !INITIALIZED.load(Ordering::Acquire) {
        return;
    }

    unsafe {
        if let Some(ref mut console) = CONSOLE {
            console.handle_interrupt();
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack));
        }

        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("wfi", options(nomem, nostack));
        }
    }
}
