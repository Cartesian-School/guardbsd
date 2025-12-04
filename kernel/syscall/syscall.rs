// Syscall Interface - Minimal Implementation
// BSD 3-Clause License

#![no_std]
use crate::proc;

// Canonical syscall table for ETAP 3.2
// Implemented: exit (0), write (1)
// To be implemented soon: exec (4), getpid (7), yield (6)
// Reserved/ENOSYS: read (2) and all others
pub const SYS_EXIT: usize = 0;
pub const SYS_WRITE: usize = 1;
pub const SYS_READ: usize = 2;   // ENOSYS for now
pub const SYS_EXEC: usize = 4;   // ENOSYS placeholder
pub const SYS_YIELD: usize = 6;  // ENOSYS placeholder
pub const SYS_GETPID: usize = 7; // ENOSYS placeholder

// Reserved/ENOSYS (keep numbering stable)
pub const SYS_FORK: usize = 3;
pub const SYS_WAIT: usize = 5;
pub const SYS_OPEN: usize = 8;
pub const SYS_CLOSE: usize = 9;
pub const SYS_MKDIR: usize = 10;
pub const SYS_STAT: usize = 11;
pub const SYS_RENAME: usize = 12;
pub const SYS_UNLINK: usize = 13;
pub const SYS_SYNC: usize = 14;
pub const SYS_LOG_READ: usize = 20;
pub const SYS_LOG_ACK: usize = 21;
pub const SYS_LOG_REGISTER_DAEMON: usize = 22;
pub const SYS_IPC_PORT_CREATE: usize = 30;
pub const SYS_IPC_SEND: usize = 31;
pub const SYS_IPC_RECV: usize = 32;

const ENOSYS: isize = -38;
const ENOENT: isize = -2;

pub fn syscall_handler(syscall_num: usize, arg1: usize, arg2: usize, arg3: usize) -> isize {
    match syscall_num {
        SYS_EXIT => sys_exit(arg1 as i32),
        SYS_WRITE => sys_write(arg1, arg2 as *const u8, arg3),
        SYS_READ => ENOSYS,
        SYS_EXEC => sys_exec(arg1 as u64),
        SYS_YIELD => ENOSYS,
        SYS_GETPID => sys_getpid(),
        SYS_FORK => ENOSYS,
        SYS_WAIT => ENOSYS,
        SYS_OPEN => ENOSYS,
        SYS_CLOSE => ENOSYS,
        SYS_MKDIR => ENOSYS,
        SYS_STAT => ENOSYS,
        SYS_RENAME => ENOSYS,
        SYS_UNLINK => ENOSYS,
        SYS_SYNC => ENOSYS,
        SYS_LOG_READ => ENOSYS,
        SYS_LOG_ACK => ENOSYS,
        SYS_LOG_REGISTER_DAEMON => ENOSYS,
        SYS_IPC_PORT_CREATE => ENOSYS,
        SYS_IPC_SEND => ENOSYS,
        SYS_IPC_RECV => ENOSYS,
        _ => ENOSYS,
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
    ENOSYS
}

fn sys_exec(path_ptr: u64) -> isize {
    // TODO: replace with real exec once task model and user entry are wired.
    const MAX_PATH_LEN: usize = 128;
    let mut buf = [0u8; MAX_PATH_LEN];
    let mut i = 0usize;

    while i < MAX_PATH_LEN {
        let c = unsafe { *(path_ptr as *const u8).add(i) };
        buf[i] = c;
        if c == 0 {
            break;
        }
        i += 1;
    }
    if i == MAX_PATH_LEN {
        buf[MAX_PATH_LEN - 1] = 0;
    }

    const INIT_PATH: &[u8] = b"/bin/init\0";
    const GSH_PATH: &[u8] = b"/bin/gsh\0";

    if buf.starts_with(INIT_PATH) || buf.starts_with(GSH_PATH) {
        0
    } else {
        ENOENT
    }
}

fn sys_getpid() -> isize {
    // TODO: replace with real task model once wired
    1
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
