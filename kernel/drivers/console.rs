//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_drivers
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Sterownik TTY konsoli z line discipline (echo, edycja, buforowanie).

use core::sync::atomic::{AtomicBool, Ordering};

const CONSOLE_LINE_BUF_SIZE: usize = 1024;

static mut LINE_BUFFER: [u8; CONSOLE_LINE_BUF_SIZE] = [0; CONSOLE_LINE_BUF_SIZE];
static mut LINE_READ: usize = 0;
static mut LINE_WRITE: usize = 0;
static mut LINE_EDITING: [u8; 256] = [0; 256]; // Current line being edited
static mut LINE_EDITING_LEN: usize = 0;
static ECHO_ENABLED: AtomicBool = AtomicBool::new(true);

// TTY control: foreground process group
static mut FOREGROUND_PGID: usize = 1; // Default to init's pgid

// Serial port for echo output
const COM1: u16 = 0x3F8;

/// Initialize console TTY
pub fn init() {
    ECHO_ENABLED.store(true, Ordering::Relaxed);
}

/// Called from keyboard IRQ handler with a character
/// Implements canonical mode line discipline:
/// - Buffers input until newline
/// - Echoes characters
/// - Handles backspace
/// - Handles Ctrl-C and Ctrl-Z for job control
pub fn handle_input_char(ch: u8) {
    unsafe {
        match ch {
            3 => {
                // Ctrl-C (ASCII 0x03): send SIGINT to foreground process group
                serial_putc(b'^');
                serial_putc(b'C');
                serial_putc(b'\r');
                serial_putc(b'\n');
                
                // Send SIGINT to foreground process group
                send_signal_to_pgid(FOREGROUND_PGID, 2); // SIGINT = 2
                
                // Clear current line
                LINE_EDITING_LEN = 0;
            }
            26 => {
                // Ctrl-Z (ASCII 0x1A): send SIGTSTP to foreground process group
                serial_putc(b'^');
                serial_putc(b'Z');
                serial_putc(b'\r');
                serial_putc(b'\n');
                
                // Send SIGTSTP to foreground process group
                send_signal_to_pgid(FOREGROUND_PGID, 20); // SIGTSTP = 20
                
                // Clear current line
                LINE_EDITING_LEN = 0;
            }
            b'\n' | b'\r' => {
                // End of line - commit the line to the ring buffer
                if LINE_EDITING_LEN > 0 {
                    for i in 0..LINE_EDITING_LEN {
                        push_to_ring_buffer(LINE_EDITING[i]);
                    }
                }
                // Add newline
                push_to_ring_buffer(b'\n');
                
                // Echo newline
                if ECHO_ENABLED.load(Ordering::Relaxed) {
                    serial_putc(b'\r');
                    serial_putc(b'\n');
                }
                
                // Reset editing buffer
                LINE_EDITING_LEN = 0;
            }
            8 | 127 => {
                // Backspace (8 = ^H, 127 = DEL)
                if LINE_EDITING_LEN > 0 {
                    LINE_EDITING_LEN -= 1;
                    
                    // Echo backspace sequence: \b \b (back, space, back)
                    if ECHO_ENABLED.load(Ordering::Relaxed) {
                        serial_putc(8);    // Move cursor back
                        serial_putc(b' '); // Overwrite with space
                        serial_putc(8);    // Move cursor back again
                    }
                }
            }
            _ => {
                // Regular character
                if LINE_EDITING_LEN < 255 {
                    LINE_EDITING[LINE_EDITING_LEN] = ch;
                    LINE_EDITING_LEN += 1;
                    
                    // Echo character
                    if ECHO_ENABLED.load(Ordering::Relaxed) {
                        serial_putc(ch);
                    }
                }
            }
        }
    }
}

/// Push character to ring buffer
unsafe fn push_to_ring_buffer(ch: u8) {
    LINE_BUFFER[LINE_WRITE] = ch;
    LINE_WRITE = (LINE_WRITE + 1) % CONSOLE_LINE_BUF_SIZE;
    
    // Handle overflow (drop oldest data)
    if LINE_WRITE == LINE_READ {
        LINE_READ = (LINE_READ + 1) % CONSOLE_LINE_BUF_SIZE;
    }
}

/// Read available bytes from console input buffer
/// Returns number of bytes read
/// This is called from the SYS_CONSOLE_READ syscall
pub fn read(buf: &mut [u8]) -> usize {
    unsafe {
        let mut count = 0;
        while count < buf.len() && LINE_READ != LINE_WRITE {
            buf[count] = LINE_BUFFER[LINE_READ];
            LINE_READ = (LINE_READ + 1) % CONSOLE_LINE_BUF_SIZE;
            count += 1;
            
            // In canonical mode, stop at newline
            if buf[count - 1] == b'\n' {
                break;
            }
        }
        count
    }
}

/// Check if input is available
pub fn has_input() -> bool {
    unsafe { LINE_READ != LINE_WRITE }
}

/// Get number of bytes available
pub fn available_bytes() -> usize {
    unsafe {
        if LINE_WRITE >= LINE_READ {
            LINE_WRITE - LINE_READ
        } else {
            CONSOLE_LINE_BUF_SIZE - LINE_READ + LINE_WRITE
        }
    }
}

/// Serial output helper
unsafe fn serial_putc(c: u8) {
    // Wait for transmit buffer to be empty
    while (inb(COM1 + 5) & 0x20) == 0 {}
    outb(COM1, c);
}

unsafe fn outb(port: u16, val: u8) {
    #[cfg(target_arch = "x86_64")]
    core::arch::asm!("out dx, al", in("dx") port, in("al") val);
}

unsafe fn inb(port: u16) -> u8 {
    let ret: u8;
    #[cfg(target_arch = "x86_64")]
    core::arch::asm!("in al, dx", out("al") ret, in("dx") port);
    ret
}

/// Set foreground process group for console TTY
pub fn set_foreground_pgid(pgid: usize) {
    unsafe {
        FOREGROUND_PGID = pgid;
    }
}

/// Get foreground process group for console TTY
pub fn get_foreground_pgid() -> usize {
    unsafe { FOREGROUND_PGID }
}

/// Send signal to all processes in a process group
/// Called from console interrupt handler (Ctrl-C, Ctrl-Z)
fn send_signal_to_pgid(pgid: usize, sig: i32) {
    // Access process table and send signal
    extern "C" {
        static mut PROCESS_TABLE: [Option<crate::process::types::Process>; 64];
    }
    
    unsafe {
        for slot in PROCESS_TABLE.iter_mut() {
            if let Some(proc) = slot {
                if proc.pgid == pgid && pgid != 1 {
                    // Don't send signals to init's process group
                    proc.pending_signals |= 1u64 << sig;
                }
            }
        }
    }
}
