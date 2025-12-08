// Console TTY Driver with Line Discipline
// BSD 3-Clause License
// Provides canonical line editing, echo, and buffering for /dev/console

use core::sync::atomic::{AtomicBool, Ordering};

const CONSOLE_LINE_BUF_SIZE: usize = 1024;

static mut LINE_BUFFER: [u8; CONSOLE_LINE_BUF_SIZE] = [0; CONSOLE_LINE_BUF_SIZE];
static mut LINE_READ: usize = 0;
static mut LINE_WRITE: usize = 0;
static mut LINE_EDITING: [u8; 256] = [0; 256]; // Current line being edited
static mut LINE_EDITING_LEN: usize = 0;
static ECHO_ENABLED: AtomicBool = AtomicBool::new(true);

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
pub fn handle_input_char(ch: u8) {
    unsafe {
        match ch {
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

