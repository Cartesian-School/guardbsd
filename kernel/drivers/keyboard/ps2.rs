//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_drivers
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Minimalny sterownik klawiatury PS/2.

const PS2_DATA: u16 = 0x60;
const PS2_STATUS: u16 = 0x64;

static mut INPUT_BUFFER: [u8; 256] = [0; 256];
static mut BUF_READ: usize = 0;
static mut BUF_WRITE: usize = 0;

// US QWERTY scancode to ASCII map (set 1)
static SCANCODE_MAP: [u8; 128] = [
    0, 27, b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0', b'-', b'=', 8, b'\t',
    b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i', b'o', b'p', b'[', b']', b'\n', 0,
    b'a', b's', b'd', b'f', b'g', b'h', b'j', b'k', b'l', b';', b'\'', b'`', 0, b'\\',
    b'z', b'x', b'c', b'v', b'b', b'n', b'm', b',', b'.', b'/', 0, b'*', 0, b' ',
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    b'-', 0, 0, 0, b'+', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub fn init() {
    // Keyboard already initialized by BIOS
}

pub fn handle_interrupt() {
    unsafe {
        let scancode = inb(PS2_DATA);
        
        // Ignore key release (high bit set)
        if scancode & 0x80 != 0 {
            return;
        }
        
        // Translate scancode to ASCII
        let ascii = SCANCODE_MAP[scancode as usize];
        if ascii != 0 {
            // Add to ring buffer (legacy)
            INPUT_BUFFER[BUF_WRITE] = ascii;
            BUF_WRITE = (BUF_WRITE + 1) % 256;
            
            // NEW: Feed to console TTY with line discipline
            crate::drivers::console::handle_input_char(ascii);
        }
    }
}

pub fn read_char() -> Option<u8> {
    unsafe {
        if BUF_READ != BUF_WRITE {
            let ch = INPUT_BUFFER[BUF_READ];
            BUF_READ = (BUF_READ + 1) % 256;
            Some(ch)
        } else {
            None
        }
    }
}

pub fn has_input() -> bool {
    unsafe { BUF_READ != BUF_WRITE }
}

unsafe fn inb(port: u16) -> u8 {
    let ret: u8;
    core::arch::asm!("in al, dx", out("al") ret, in("dx") port);
    ret
}
