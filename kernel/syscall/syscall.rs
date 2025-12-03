// Syscall Interface - Minimal Implementation
// BSD 3-Clause License

#![no_std]

pub const SYS_EXIT: usize = 0;
pub const SYS_WRITE: usize = 1;
pub const SYS_READ: usize = 2;
pub const SYS_OPEN: usize = 3;
pub const SYS_EXEC: usize = 4;

pub fn syscall_handler(syscall_num: usize, arg1: usize, arg2: usize, arg3: usize) -> isize {
    match syscall_num {
        SYS_EXIT => sys_exit(arg1 as i32),
        SYS_WRITE => sys_write(arg1, arg2 as *const u8, arg3),
        SYS_READ => sys_read(arg1, arg2 as *mut u8, arg3),
        _ => -1,
    }
}

fn sys_exit(status: i32) -> isize {
    // Terminate process
    loop {}
}

fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    if fd == 1 || fd == 2 {
        // stdout/stderr - write to serial
        unsafe {
            let slice = core::slice::from_raw_parts(buf, len);
            for &byte in slice {
                serial_putc(byte);
            }
        }
        len as isize
    } else {
        -1
    }
}

fn sys_read(_fd: usize, _buf: *mut u8, _len: usize) -> isize {
    -1 // Not implemented
}

const COM1: u16 = 0x3F8;

unsafe fn serial_putc(c: u8) {
    while (inb(COM1 + 5) & 0x20) == 0 {}
    outb(COM1, c);
}

unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") val);
}

unsafe fn inb(port: u16) -> u8 {
    let ret: u8;
    core::arch::asm!("in al, dx", out("al") ret, in("dx") port);
    ret
}
