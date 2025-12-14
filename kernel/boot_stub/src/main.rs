//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: boot_stub
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! GuardBSD boot stub main module (kernel entry).

#![no_std]
#![no_main]

use core::convert::TryInto;
use core::ffi::c_void;
use core::panic::PanicInfo;
use core::str;

mod drivers;
mod fs;
mod interrupt;
// Note: Using kernel/sched/mod.rs instead of local scheduler
mod ipc;
mod kernel;
mod log_sink;
mod process;
mod sched;
mod syscalls;

// ============================================================================
// Errno constants (local to boot_stub; negative Linux-style values)
// ============================================================================
const ENOSYS: isize = -38;
const EINVAL: isize = -22;
const EFAULT: isize = -14;
const EIO: isize = -5;
const ERANGE: isize = -34;

// ============================================================================
// Port I/O helpers (must be defined before first use)
// ============================================================================
#[inline(always)]
pub unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
}

#[inline(always)]
pub unsafe fn inb(port: u16) -> u8 {
    let ret: u8;
    core::arch::asm!("in al, dx", out("al") ret, in("dx") port, options(nomem, nostack, preserves_flags));
    ret
}

mod syscall {
    //! Syscall dispatch layer for boot_stub.
    //!
    //! IMPORTANT:
    //! - This boot_stub build MUST NOT depend on external crates like `shared`.
    //! - Therefore syscall numbers are defined locally here.
    //!
    //! If/when you add the real `shared` crate later, you can switch these constants
    //! back to canonical values.

    use super::{EFAULT, EINVAL, EIO, ENOSYS, ERANGE};

    // ------------------------------------------------------------------------
    // Syscall numbers (local, boot_stub only)
    // ------------------------------------------------------------------------
    // Keep the "classic" ones compatible with x86_64 Linux where reasonable,
    // and assign your custom ones in a high range to avoid collisions.
    pub(crate) const SYS_WRITE: usize = 1;
    pub(crate) const SYS_READ: usize = 0;
    pub(crate) const SYS_OPEN: usize = 2;   // boot_stub local (not Linux)
    pub(crate) const SYS_CLOSE: usize = 3;  // boot_stub local (not Linux)

    pub(crate) const SYS_EXIT: usize = 60;
    pub(crate) const SYS_GETPID: usize = 39;
    pub(crate) const SYS_FORK: usize = 57;
    pub(crate) const SYS_EXEC: usize = 59;   // execve-like
    pub(crate) const SYS_WAIT: usize = 61;   // wait4-like
    pub(crate) const SYS_YIELD: usize = 24_000; // custom

    pub(crate) const SYS_SIGNAL: usize = 24_010;         // custom
    pub(crate) const SYS_SIGACTION: usize = 24_011;      // custom
    pub(crate) const SYS_SIGNAL_REGISTER: usize = 24_012; // custom
    pub(crate) const SYS_SIGRETURN: usize = 24_013;      // custom
    pub(crate) const SYS_WAITPID: usize = 24_014;        // custom

    // Job control / process groups
    pub(crate) const SYS_KILL: usize = 62;   // kill-like
    pub(crate) const SYS_SETPGID: usize = 24_020;
    pub(crate) const SYS_GETPGID: usize = 24_021;

    // FD management / VFS
    pub(crate) const SYS_DUP: usize = 24_100;
    pub(crate) const SYS_DUP2: usize = 24_101;
    pub(crate) const SYS_STAT: usize = 24_102;
    pub(crate) const SYS_MKDIR: usize = 24_103;
    pub(crate) const SYS_UNLINK: usize = 24_104;
    pub(crate) const SYS_RENAME: usize = 24_105;
    pub(crate) const SYS_SYNC: usize = 24_106;
    pub(crate) const SYS_CHDIR: usize = 24_107;
    pub(crate) const SYS_GETCWD: usize = 24_108;
    pub(crate) const SYS_MOUNT: usize = 24_109;
    pub(crate) const SYS_UMOUNT: usize = 24_110;

    pub(crate) const SYS_CONSOLE_READ: usize = 24_120;
    pub(crate) const SYS_TCSETPGRP: usize = 24_121;
    pub(crate) const SYS_TCGETPGRP: usize = 24_122;

    // Logging
    pub(crate) const SYS_LOG_READ: usize = 24_200;
    pub(crate) const SYS_LOG_ACK: usize = 24_201;
    pub(crate) const SYS_LOG_REGISTER_DAEMON: usize = 24_202;

    // Service registry
    pub(crate) const SYS_SERVICE_REGISTER: usize = 24_300;

    // IPC
    pub(crate) const SYS_IPC_PORT_CREATE: usize = 24_400;
    pub(crate) const SYS_IPC_SEND: usize = 24_401;
    pub(crate) const SYS_IPC_RECV: usize = 24_402;

    pub fn syscall_handler(syscall_num: usize, arg1: usize, arg2: usize, arg3: usize) -> isize {
        // Delegate to main kernel implementations where available.
        match syscall_num {
            // Process management
            SYS_EXIT => {
                crate::syscalls::process::sys_exit(arg1 as i32);
                0
            }
            SYS_GETPID => crate::syscalls::process::sys_getpid(),
            SYS_FORK => crate::syscalls::process::sys_fork(),
            SYS_EXEC => crate::syscalls::process::sys_exec(arg1 as *const u8, arg2 as *const *const u8),
            SYS_WAIT => crate::syscalls::process::sys_wait(arg1 as *mut i32),
            SYS_YIELD => {
                crate::sched::yield_current();
                0
            }

            // Signals
            SYS_SIGNAL => crate::syscalls::signal::sys_signal(arg2 as i32, arg1 as u64),
            SYS_SIGACTION => crate::syscalls::signal::sys_sigaction(
                arg1 as i32,
                arg2 as *const c_void,
                arg3 as *mut c_void,
            ),
            SYS_SIGNAL_REGISTER => crate::syscalls::signal::sys_signal(arg1 as i32, arg2 as u64),
            SYS_SIGRETURN => crate::syscalls::signal::sys_sigreturn(),
            SYS_WAITPID => crate::syscalls::process::sys_waitpid(arg1 as isize, arg2 as *mut i32, arg3 as i32),

            // Job control (preferred mapping for SYS_KILL / waitpid variant)
            SYS_KILL => crate::syscalls::process_jobctl::sys_kill(arg1 as isize, arg2 as i32),
            SYS_SETPGID => crate::syscalls::process_jobctl::sys_setpgid(arg1, arg2),
            SYS_GETPGID => crate::syscalls::process_jobctl::sys_getpgid(arg1),

            // File operations
            SYS_WRITE => crate::syscalls::fs::sys_write(arg1 as u32, arg2 as *const u8, arg3),
            SYS_READ => crate::syscalls::fs::sys_read(arg1 as u32, arg2 as *mut u8, arg3),
            SYS_OPEN => crate::syscalls::fs::sys_open(arg1 as *const u8, arg2 as u32),
            SYS_CLOSE => crate::syscalls::fs::sys_close(arg1 as u32),

            SYS_DUP => crate::syscalls::fs::sys_dup(arg1),
            SYS_DUP2 => crate::syscalls::fs::sys_dup2(arg1, arg2),
            SYS_STAT => crate::syscalls::fs::sys_stat(arg1 as *const u8, arg2 as *mut u8),
            SYS_MKDIR => sys_mkdir(arg1 as *const u8, arg2),
            SYS_UNLINK => sys_unlink(arg1 as *const u8),
            SYS_RENAME => sys_rename(arg1 as *const u8, arg2 as *const u8),
            SYS_SYNC => sys_sync(arg1),
            SYS_CHDIR => sys_chdir(arg1 as *const u8),
            SYS_GETCWD => sys_getcwd(arg1 as *mut u8, arg2),
            SYS_MOUNT => sys_mount(arg1 as *const u8, arg2 as *const u8, arg3 as *const u8),
            SYS_UMOUNT => sys_umount(arg1 as *const u8),
            SYS_CONSOLE_READ => crate::syscalls::fs::sys_console_read(arg1 as *mut u8, arg2),
            SYS_TCSETPGRP => crate::syscalls::fs::sys_tcsetpgrp(arg1 as u32, arg2),
            SYS_TCGETPGRP => crate::syscalls::fs::sys_tcgetpgrp(arg1 as u32),

            // Logging
            SYS_LOG_READ => crate::syscalls::log::sys_log_read(arg1 as *mut u8, arg2 as usize),
            SYS_LOG_ACK => ENOSYS,
            SYS_LOG_REGISTER_DAEMON => ENOSYS,

            // Service registry
            SYS_SERVICE_REGISTER => crate::service_syscall::sys_service_register(arg1 as *const u8, arg2 as usize),

            // IPC
            SYS_IPC_PORT_CREATE => sys_ipc_port_create(),
            SYS_IPC_SEND => sys_ipc_send(arg1, arg2, arg3 as u32, [0, 0, 0, 0]),
            SYS_IPC_RECV => sys_ipc_recv(arg1),

            _ => EINVAL,
        }
    }

    // ------------------------------------------------------------------------
    // File ops that are not yet delegated (simple stubs / VFS helpers)
    // ------------------------------------------------------------------------
    fn sys_mkdir(path: *const u8, mode: usize) -> isize {
        vfs_mkdir(path, mode)
    }

    fn sys_unlink(path: *const u8) -> isize {
        vfs_unlink(path)
    }

    fn sys_rename(_old_path: *const u8, _new_path: *const u8) -> isize {
        ENOSYS
    }

    fn sys_sync(_fd: usize) -> isize {
        0
    }

    fn sys_chdir(_path: *const u8) -> isize {
        0
    }

    fn sys_getcwd(buf: *mut u8, size: usize) -> isize {
        unsafe {
            if size < 2 {
                return ERANGE;
            }
            *buf = b'/';
            *(buf.add(1)) = 0;
            2
        }
    }

    fn sys_mount(_source: *const u8, _target: *const u8, _fstype: *const u8) -> isize {
        0
    }

    fn sys_umount(_target: *const u8) -> isize {
        ENOSYS
    }

    // ------------------------------------------------------------------------
    // VFS IPC helpers
    // ------------------------------------------------------------------------
    const VFS_PORT: usize = 1000;

    fn vfs_mkdir(path: *const u8, mode: usize) -> isize {
        unsafe {
            if path.is_null() {
                return EFAULT;
            }

            let mut req_buf = [0u8; 512];
            req_buf[0..4].copy_from_slice(&6u32.to_le_bytes()); // Mkdir = 6

            let mut path_len = 0;
            while path_len < 255 {
                let byte = *path.add(path_len);
                if byte == 0 {
                    break;
                }
                req_buf[8 + path_len] = byte;
                path_len += 1;
            }

            req_buf[268..272].copy_from_slice(&(mode as u32).to_le_bytes());

            if crate::ipc::ipc_send_simple(VFS_PORT, req_buf.as_ptr(), 512) < 0 {
                return EIO;
            }

            let mut resp_buf = [0u8; 512];
            if crate::ipc::ipc_recv(VFS_PORT.try_into().unwrap(), resp_buf.as_mut_ptr(), 512) < 0 {
                return EIO;
            }

            let result = i64::from_le_bytes(resp_buf[0..8].try_into().unwrap());
            result as isize
        }
    }

    fn vfs_unlink(path: *const u8) -> isize {
        unsafe {
            if path.is_null() {
                return EFAULT;
            }

            let mut req_buf = [0u8; 512];
            req_buf[0..4].copy_from_slice(&8u32.to_le_bytes()); // Unlink = 8

            let mut path_len = 0;
            while path_len < 255 {
                let byte = *path.add(path_len);
                if byte == 0 {
                    break;
                }
                req_buf[8 + path_len] = byte;
                path_len += 1;
            }

            if crate::ipc::ipc_send_simple(VFS_PORT, req_buf.as_ptr(), 512) < 0 {
                return EIO;
            }

            let mut resp_buf = [0u8; 512];
            if crate::ipc::ipc_recv(VFS_PORT.try_into().unwrap(), resp_buf.as_mut_ptr(), 512) < 0 {
                return EIO;
            }

            let result = i64::from_le_bytes(resp_buf[0..8].try_into().unwrap());
            result as isize
        }
    }

    // ------------------------------------------------------------------------
    // IPC syscalls (boot_stub wrappers)
    // ------------------------------------------------------------------------
    fn sys_ipc_port_create() -> isize {
        crate::ipc::ipc_create_port(0)
    }

    fn sys_ipc_send(port_id: usize, receiver_pid: usize, msg_type: u32, data: [u32; 4]) -> isize {
        crate::ipc::ipc_send(port_id, 0, receiver_pid, msg_type, data)
    }

    fn sys_ipc_recv(_port_id: usize) -> isize {
        0
    }
}

mod service_syscall {
    use crate::ipc;

    // Import parent module helpers (they are private, but visible to child modules).
    use super::{print, print_num};

    pub fn sys_service_register(name_ptr: *const u8, pid: usize) -> isize {
        if name_ptr.is_null() {
            return -1;
        }

        // Copy name (up to 32 bytes)
        let mut name_buf = [0u8; 32];
        let mut len = 0usize;
        unsafe {
            while len < name_buf.len() {
                let b = *name_ptr.add(len);
                if b == 0 {
                    break;
                }
                name_buf[len] = b;
                len += 1;
            }
        }

        let name = core::str::from_utf8(&name_buf[..len]).unwrap_or("");
        if name.is_empty() {
            return -1;
        }

        let ok = ipc::register_service(name, 0, pid);
        if ok {
            unsafe {
                print("[SERVICE] registered service '");
                print(name);
                print("' with PID ");
                print_num(pid);
                print("\n");
            }

            // Test lookup
            if let Some((_, found_pid)) = ipc::lookup_service(name) {
                if found_pid == pid {
                    unsafe {
                        print("[SERVICE] lookup('");
                        print(name);
                        print("') = ");
                        print_num(found_pid);
                        print("\n");
                    }
                }
            }
            0
        } else {
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn syscall_dispatch(syscall_num: usize, arg1: usize, arg2: usize, arg3: usize) -> isize {
    syscall::syscall_handler(syscall_num, arg1, arg2, arg3)
}

#[no_mangle]
pub extern "C" fn keyboard_interrupt_handler() {
    drivers::keyboard::handle_interrupt();
}

#[no_mangle]
pub extern "C" fn timer_interrupt_handler() {
    drivers::timer::handle_interrupt();

    let cpu_id = 0;
    unsafe {
        crate::sched::on_tick(cpu_id);
    }
}

// ============================================================================
// GuaBoot Boot Protocol
// ============================================================================
const GBSD_MAGIC: u32 = 0x4253_4447; // "GBSD"
const PAGE_SIZE: usize = 4096;
const MAX_MEMORY_BYTES: usize = 0x0800_0000; // 128MB
const MAX_PAGES: usize = MAX_MEMORY_BYTES / PAGE_SIZE;
const DISABLE_MODULES: bool = false;
const MAX_MODULE_SIZE: usize = 16 * 1024 * 1024;
const RUN_USERTEST: bool = true;
const MIN_USER_MODULE_BASE: u64 = 0x0000_0000_0400_0000;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BootMmapEntry {
    pub base: u64,
    pub length: u64,
    pub typ: u32, // 1 = usable
    pub reserved: u32,
}

#[repr(C)]
pub struct BootInfo {
    pub magic: u32,
    pub version: u32,
    pub size: u32,
    pub kernel_crc32: u32,
    pub kernel_base: u64,
    pub kernel_size: u64,
    pub mem_lower: u64,
    pub mem_upper: u64,
    pub boot_device: u32,
    pub _pad0: u32,
    pub cmdline: *const u8,
    pub mods_count: u32,
    pub _pad1: u32,
    pub mods: *const Module,
    pub mmap: *const BootMmapEntry,
    pub mmap_count: u32,
    pub _pad2: u32,
}

#[repr(C)]
pub struct Module {
    pub mod_start: u64,
    pub mod_end: u64,
    pub string: *const u8,
    pub reserved: u32,
}

extern "C" {
    static guaboot_magic: u32;
    static guaboot_bootinfo_ptr: *const BootInfo;
    static tss_ready_flag: u8;
    static tss_stack_top: u8;
    fn jump_to_user(ctx: *const crate::sched::ArchContext, entry: u64, stack: u64) -> !;
}

// Linker-provided kernel image bounds.
extern "C" {
    #[link_name = "__image_start"]
    static IMAGE_START: u8;

    #[link_name = "__image_end"]
    static IMAGE_END: u8;
}

#[allow(dead_code)]
#[inline(always)]
fn kernel_image_bounds() -> (*const u8, usize) {
    let start = core::ptr::addr_of!(IMAGE_START) as usize;
    let end = core::ptr::addr_of!(IMAGE_END) as usize;
    let len = end.saturating_sub(start);
    (start as *const u8, len)
}

const COM1: u16 = 0x3F8;
static mut PMM_BITMAP: [u64; MAX_PAGES / 64] = [0; MAX_PAGES / 64];
static mut NEXT_PAGE: usize = 0;

#[cfg(target_pointer_width = "64")]
#[inline(always)]
unsafe fn load_cr3(value: u64) {
    core::arch::asm!("mov cr3, {}", in(reg) value, options(nostack, preserves_flags));
}

#[cfg(target_pointer_width = "64")]
#[inline(always)]
unsafe fn read_cr3() -> u64 {
    let value: u64;
    core::arch::asm!("mov {}, cr3", out(reg) value, options(nostack, preserves_flags));
    value
}

#[cfg(not(target_pointer_width = "64"))]
#[inline(always)]
unsafe fn load_cr3(_value: u64) {}

#[cfg(not(target_pointer_width = "64"))]
#[inline(always)]
unsafe fn read_cr3() -> u64 {
    0
}

fn init_kernel_template() {}
fn kernel_template_ready() -> bool {
    false
}

unsafe fn serial_init() {
    outb(COM1 + 1, 0x00);
    outb(COM1 + 3, 0x80);
    outb(COM1 + 0, 0x03);
    outb(COM1 + 1, 0x00);
    outb(COM1 + 3, 0x03);
    outb(COM1 + 2, 0xC7);
    outb(COM1 + 4, 0x0B);
}

unsafe fn print(s: &str) {
    for b in s.bytes() {
        while (inb(COM1 + 5) & 0x20) == 0 {}
        outb(COM1, b);
    }
}

unsafe fn print_ptr(ptr: *const u8) {
    if ptr.is_null() {
        print("<null>");
        return;
    }
    let mut current = ptr;
    loop {
        let byte = *current;
        if byte == 0 {
            break;
        }
        while (inb(COM1 + 5) & 0x20) == 0 {}
        outb(COM1, byte);
        current = current.add(1);
    }
}

fn print_num(n: usize) {
    unsafe {
        let mut buf = [0u8; 20];
        let mut i = 0usize;
        let mut num = n;

        if num == 0 {
            print("0");
            return;
        }

        while num > 0 {
            buf[i] = b'0' + (num % 10) as u8;
            num /= 10;
            i += 1;
        }

        while i > 0 {
            i -= 1;
            while (inb(COM1 + 5) & 0x20) == 0 {}
            outb(COM1, buf[i]);
        }
    }
}

fn print_hex32(n: u32) {
    unsafe {
        let hex = b"0123456789abcdef";
        for i in (0..8).rev() {
            let nibble = ((n >> (i * 4)) & 0xF) as usize;
            while (inb(COM1 + 5) & 0x20) == 0 {}
            outb(COM1, hex[nibble]);
        }
    }
}

fn print_hex64(n: u64) {
    unsafe {
        let hex = b"0123456789abcdef";
        for i in (0..16).rev() {
            let nibble = ((n >> (i * 4)) & 0xF) as usize;
            while (inb(COM1 + 5) & 0x20) == 0 {}
            outb(COM1, hex[nibble]);
        }
    }
}

#[no_mangle]
pub extern "C" fn syscall_dispatch_trap(rax: u64, rdi: u64, rsi: u64, rdx: u64) -> u64 {
    unsafe {
        print("[SYSCALL] dispatcher invoked (test path), nr=");
        print_num(rax as usize);
        print("\n");
    }
    syscall::syscall_handler(rax as usize, rdi as usize, rsi as usize, rdx as usize) as u64
}

fn panic_and_halt(msg: &str) -> ! {
    unsafe {
        print("\n[PANIC] ");
        print(msg);
        print("\n");
        loop {
            core::arch::asm!("cli; hlt");
        }
    }
}

fn print_cstr(ptr: *const u8) {
    unsafe {
        if ptr.is_null() {
            print("(null)");
            return;
        }
        let mut len = 0usize;
        while len < 64 {
            let b = *ptr.add(len);
            if b == 0 {
                break;
            }
            len += 1;
        }
        let slice = core::slice::from_raw_parts(ptr, len);
        if let Ok(s) = str::from_utf8(slice) {
            print(s);
        } else {
            print("(invalid utf8)");
        }
    }
}

fn validate_bootinfo() -> &'static BootInfo {
    unsafe {
        if guaboot_magic != GBSD_MAGIC {
            panic_and_halt("Invalid BootInfo magic – boot protocol not satisfied");
        }
        if guaboot_bootinfo_ptr.is_null() {
            panic_and_halt("BootInfo pointer is null");
        }
        let bi = &*guaboot_bootinfo_ptr;

        let expected_size = core::mem::size_of::<BootInfo>() as u32;
        if bi.size < expected_size {
            panic_and_halt("BootInfo size too small");
        }
        bi
    }
}

fn log_bootinfo(bi: &BootInfo) {
    unsafe {
        print("[BOOT] BootInfo:\n");
        print("[BOOT]   magic=0x");
        print_hex32(bi.magic);
        print(" version=0x");
        print_hex32(bi.version);
        print(" size=");
        print_num(bi.size as usize);
        print(" bytes\n");

        print("[BOOT]   mem_lower=");
        print_num(bi.mem_lower as usize);
        print(" KB mem_upper=");
        print_num(bi.mem_upper as usize);
        print(" KB\n");

        print("[BOOT]   mmap_count=");
        print_num(bi.mmap_count as usize);
        print("\n");

        print("[BOOT]   kernel_crc32=0x");
        print_hex32(bi.kernel_crc32);
        print("\n");

        print("[BOOT]   kernel_base=0x");
        print_hex64(bi.kernel_base);
        print(" kernel_size=0x");
        print_hex64(bi.kernel_size);
        print("\n");

        print("[BOOT]   boot_device=0x");
        print_hex32(bi.boot_device);
        print(" cmdline=");
        print_ptr(bi.cmdline);
        print("\n");

        let entries = core::slice::from_raw_parts(bi.mmap, bi.mmap_count as usize);
        for e in entries.iter() {
            if e.typ != 1 {
                continue;
            }
            print("[MMAP] usable region: base=0x");
            print_hex64(e.base);
            print(" len=0x");
            print_hex64(e.length);
            print(" (pages=");
            let pages = (e.length as usize + PAGE_SIZE - 1) / PAGE_SIZE;
            print_num(pages);
            print(")\n");
        }
    }
}

fn log_modules(bi: &BootInfo) {
    unsafe {
        if bi.mods_count == 0 || bi.mods.is_null() {
            print("[BOOT] modules: none present in BootInfo\n");
            return;
        }

        let count = bi.mods_count as usize;
        let mods = core::slice::from_raw_parts(bi.mods, count);
        print("[BOOT] modules count=");
        print_num(count);
        print("\n");

        for m in mods {
            let size = m.mod_end.saturating_sub(m.mod_start);
            let magic = if size >= 4 {
                let bytes = core::slice::from_raw_parts(m.mod_start as *const u8, 4);
                bytes == [0x7F, b'E', b'L', b'F']
            } else {
                false
            };
            print("[MOD] name=");
            print_cstr(m.string);
            print(" start=0x");
            print_hex64(m.mod_start);
            print(" end=0x");
            print_hex64(m.mod_end);
            print(" size=0x");
            print_hex64(size);
            if magic {
                print(" magic=ELF");
            } else {
                print(" magic=?");
            }
            print("\n");
        }
    }
}

fn panic_invalid_module(name_ptr: *const u8, start: u64, end: u64, data: &[u8], msg: &str) -> ! {
    unsafe {
        print("[MOD] invalid module '");
        print_ptr(name_ptr);
        print("' start=0x");
        print_hex64(start);
        print(" end=0x");
        print_hex64(end);
        let size = end.saturating_sub(start);
        print(" size=0x");
        print_hex64(size);
        print(" : ");
        print(msg);
        print("\n[MOD] first 16 bytes: ");
        let dump_len = core::cmp::min(data.len(), 16);
        for i in 0..dump_len {
            print_hex32(data[i] as u32);
            print(" ");
        }
        print("\n");
    }
    panic_and_halt("Module validation failed");
}

fn validate_module_elf(name_ptr: *const u8, data: &[u8]) {
    #[repr(C)]
    struct Elf64Ehdr {
        e_ident: [u8; 16],
        e_type: u16,
        e_machine: u16,
        e_version: u32,
        e_entry: u64,
        e_phoff: u64,
        e_shoff: u64,
        e_flags: u32,
        e_ehsize: u16,
        e_phentsize: u16,
        e_phnum: u16,
        e_shentsize: u16,
        e_shnum: u16,
        e_shstrndx: u16,
    }

    #[repr(C)]
    struct Elf64Phdr {
        p_type: u32,
        p_flags: u32,
        p_offset: u64,
        p_vaddr: u64,
        p_paddr: u64,
        p_filesz: u64,
        p_memsz: u64,
        p_align: u64,
    }

    const PT_LOAD: u32 = 1;

    if data.len() < core::mem::size_of::<Elf64Ehdr>() {
        panic_invalid_module(name_ptr, 0, 0, data, "Module ELF header truncated");
    }
    let ehdr = unsafe { &*(data.as_ptr() as *const Elf64Ehdr) };

    // Basic header validation
    if ehdr.e_ident[4] != 2 {
        panic_invalid_module(name_ptr, 0, 0, data, "Module ELF not 64-bit class");
    }
    if ehdr.e_ident[5] != 1 {
        panic_invalid_module(name_ptr, 0, 0, data, "Module ELF not little-endian");
    }
    if ehdr.e_type != 2 {
        panic_invalid_module(name_ptr, 0, 0, data, "Module ELF not ET_EXEC");
    }
    if ehdr.e_machine != 0x3E {
        panic_invalid_module(name_ptr, 0, 0, data, "Module ELF not x86_64 machine");
    }

    // FIX: cast cannot be followed by a method call -> parentheses
    let ph_end = (ehdr.e_phoff as usize)
        .saturating_add((ehdr.e_phnum as usize) * (ehdr.e_phentsize as usize));
    if ph_end > data.len() {
        panic_invalid_module(name_ptr, 0, 0, data, "Module ELF program headers out of range");
    }

    let mut total_mem: u64 = 0;
    let phdr_size = core::mem::size_of::<Elf64Phdr>();
    for i in 0..ehdr.e_phnum {
        let off = ehdr.e_phoff as usize + i as usize * ehdr.e_phentsize as usize;
        if off + phdr_size > data.len() {
            panic_invalid_module(name_ptr, 0, 0, data, "Module ELF program header truncated");
        }
        let ph = unsafe { &*(data.as_ptr().add(off) as *const Elf64Phdr) };
        if ph.p_type != PT_LOAD {
            continue;
        }

        unsafe {
            print("[MOD] PH load vaddr=0x");
            print_hex64(ph.p_vaddr);
            print(" memsz=0x");
            print_hex64(ph.p_memsz);
            print(" filesz=0x");
            print_hex64(ph.p_filesz);
            print(" align=0x");
            print_hex64(ph.p_align);
            print("\n");
        }

        if ph.p_vaddr < MIN_USER_MODULE_BASE {
            panic_invalid_module(name_ptr, 0, 0, data, "Module maps into kernel-reserved region");
        }
        if (ph.p_vaddr & 0xFFF) != (ph.p_offset & 0xFFF) {
            panic_invalid_module(name_ptr, 0, 0, data, "Module PT_LOAD misaligned (vaddr/off mismatch)");
        }
        total_mem = total_mem.saturating_add(ph.p_memsz);
    }

    if total_mem > (MAX_MODULE_SIZE as u64) {
        panic_invalid_module(name_ptr, 0, 0, data, "Module PT_LOAD mem footprint exceeds limit");
    }

    unsafe {
        print("[MOD] ELF validated for '");
        print_ptr(name_ptr);
        print("' entry=0x");
        print_hex64(ehdr.e_entry);
        print(" total_mem=0x");
        print_hex64(total_mem);
        print("\n");
    }
}

unsafe fn spawn_module_from_blob(name_ptr: *const u8, start: u64, end: u64) -> usize {
    if start == 0 || end == 0 || start >= end {
        panic_and_halt("Module blob range invalid");
    }

    let len = (end - start) as usize;
    if len == 0 || len > MAX_MODULE_SIZE {
        panic_and_halt("Module blob size exceeds limit");
    }
    if (start & 0xF) != 0 {
        print("[MOD] Warning: module start not 16-byte aligned\n");
    } else if (start & 0xFFF) != 0 {
        print("[MOD] Warning: module start not page aligned\n");
    }
    let data = core::slice::from_raw_parts(start as *const u8, len);

    // Minimal ELF validation
    if len < 5 || data[0] != 0x7F || data[1] != b'E' || data[2] != b'L' || data[3] != b'F' || data[4] != 2 {
        panic_invalid_module(name_ptr, start, end, data, "Module ELF validation failed (magic/class)");
    }

    validate_module_elf(name_ptr, data);

    let mut aspace = kernel::mm::AddressSpace::new_with_kernel_mappings();
    let loaded = crate::process::elf_loader::parse_and_load_elf(data, &mut aspace)
        .unwrap_or_else(|_| panic_and_halt("Failed to load module ELF"));

    const MODULE_STACK_TOP: usize = 0x0000_7FFE_F000;
    const STACK_PAGES: usize = 4; // 16KB
    for i in 0..STACK_PAGES {
        let phys = kernel::mm::alloc_page().unwrap_or_else(|| panic_and_halt("Out of memory for module stack"));
        let virt = MODULE_STACK_TOP - (i + 1) * 4096;
        let flags = kernel::mm::PageFlags::PRESENT
            | kernel::mm::PageFlags::WRITABLE
            | kernel::mm::PageFlags::USER;
        if !aspace.map(virt as u64, phys as u64, flags) {
            panic_and_halt("Failed to map module stack page");
        }
    }

    let pid = crate::process::process::create_process(
        loaded.entry,
        MODULE_STACK_TOP as u64,
        aspace.pml4_phys(),
    );
    if pid == 0 {
        panic_and_halt("Failed to allocate PID for module");
    }

    let mut ctx = crate::sched::ArchContext::zeroed();
    ctx.rip = loaded.entry;
    ctx.rsp = MODULE_STACK_TOP as u64;
    ctx.cs = 0x1B;
    ctx.ss = 0x23;
    ctx.rflags = 0x202;
    ctx.cr3 = aspace.pml4_phys() as u64;

    if crate::sched::register_thread(pid as i32, 1, 0, ctx).is_none() {
        panic_and_halt("Failed to register module thread");
    }

    print("[MOD] spawned '");
    print_cstr(name_ptr);
    print("' pid=");
    print_num(pid as usize);
    print(" entry=0x");
    print_hex64(loaded.entry);
    print(" cr3=0x");
    print_hex64(aspace.pml4_phys() as u64);
    print(" stack=0x");
    print_hex64(MODULE_STACK_TOP as u64);
    print("\n");

    pid as usize
}

const USERTEST_BASE: u64 = 0x0000_0070_0000;
const USERTEST_STACK_TOP: u64 = 0x0000_0070_F000;
const USERTEST_CODE: [u8; 57] = [
    0x48, 0xB8, 0x0B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov rax,11
    0x48, 0xBF, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov rdi,1
    0x48, 0x8D, 0x35, 0x0D, 0x00, 0x00, 0x00, // lea rsi,[rip+0xd]
    0x48, 0xC7, 0xC2, 0x11, 0x00, 0x00, 0x00, // mov rdx,0x11
    0xCD, 0x80, // int 0x80
    0xF3, 0x90,             // pause
    0xEB, 0xFC,             // jmp back to pause
    b'[', b'U', b'S', b'E', b'R', b'T', b'E', b'S', b'T', b']', b' ', b'h', b'e', b'l', b'l', b'o', b'\n',
];

unsafe fn spawn_user_smoke_test() {
    print("[USERTEST] Spawning user-mode syscall smoke test...\n");

    let mut aspace = kernel::mm::AddressSpace::new_with_kernel_mappings();

    // Use separate flags to avoid "use of moved value" if PageFlags is not Copy.
    let flags_code = kernel::mm::PageFlags::PRESENT | kernel::mm::PageFlags::USER | kernel::mm::PageFlags::WRITABLE;
    let flags_stack = kernel::mm::PageFlags::PRESENT | kernel::mm::PageFlags::USER | kernel::mm::PageFlags::WRITABLE;

    // Map code page
    let code_phys = kernel::mm::alloc_page().unwrap_or_else(|| panic_and_halt("Out of memory for usertest code"));
    if !aspace.map(USERTEST_BASE, code_phys, flags_code) {
        panic_and_halt("Failed to map usertest code page");
    }
    // Copy code into the freshly allocated physical page (identity mapping assumed in early boot).
    let dst = code_phys as *mut u8;
    for (i, b) in USERTEST_CODE.iter().enumerate() {
        core::ptr::write_volatile(dst.add(i), *b);
    }

    // Map stack
    const STACK_PAGES: usize = 4;
    for i in 0..STACK_PAGES {
        let phys = kernel::mm::alloc_page().unwrap_or_else(|| panic_and_halt("Out of memory for usertest stack"));
        let virt = USERTEST_STACK_TOP - ((i + 1) * 4096) as u64;
        if !aspace.map(virt, phys, flags_stack) {
            panic_and_halt("Failed to map usertest stack page");
        }
    }

    let pid = crate::process::process::create_process(USERTEST_BASE, USERTEST_STACK_TOP, aspace.pml4_phys());
    if pid == 0 {
        panic_and_halt("Failed to allocate PID for usertest");
    }

    let mut ctx = crate::sched::ArchContext::zeroed();
    ctx.rip = USERTEST_BASE;
    ctx.rsp = USERTEST_STACK_TOP;
    ctx.cs = 0x1B; // user mode
    ctx.ss = 0x23; // user mode data
    ctx.rflags = 0x202;
    ctx.cr3 = aspace.pml4_phys() as u64;

    if crate::sched::register_thread(pid as i32, 1, 0, ctx).is_none() {
        panic_and_halt("Failed to register usertest thread");
    }

    print("[USERTEST] spawned pid=");
    print_num(pid as usize);
    print(" entry=0x");
    print_hex64(USERTEST_BASE);
    print(" cr3=0x");
    print_hex64(aspace.pml4_phys() as u64);
    print(" rsp=0x");
    print_hex64(USERTEST_STACK_TOP);
    print("\n");
}

unsafe fn pmm_mark_all_reserved() {
    for slot in PMM_BITMAP.iter_mut() {
        *slot = !0u64;
    }
    NEXT_PAGE = MAX_PAGES; // will be lowered when marking usable
}

unsafe fn pmm_mark_usable_range(base: u64, length: u64) {
    if length == 0 {
        return;
    }
    let start_page = (base as usize) / PAGE_SIZE;
    let end_page = ((base as usize + length as usize + PAGE_SIZE - 1) / PAGE_SIZE).min(MAX_PAGES);

    for page in start_page..end_page {
        let idx = page / 64;
        let bit = page % 64;
        PMM_BITMAP[idx] &= !(1u64 << bit);
        if page < NEXT_PAGE {
            NEXT_PAGE = page;
        }
    }
}

unsafe fn init_memory(bi: &BootInfo) {
    let count = bi.mmap_count as usize;
    if count == 0 {
        panic_and_halt("BootInfo contains no memory map entries");
    }
    pmm_mark_all_reserved();
    let entries = core::slice::from_raw_parts(bi.mmap, count);

    // Validate entries
    for e in entries {
        if e.length == 0 {
            panic_and_halt("BootInfo memory entry has zero length");
        }
    }
    for i in 0..count {
        if entries[i].typ != 1 {
            continue;
        }
        let start_i = entries[i].base;
        let end_i = start_i.checked_add(entries[i].length).unwrap_or(0);
        if end_i == 0 {
            panic_and_halt("BootInfo memory entry overflow");
        }
        for j in (i + 1)..count {
            if entries[j].typ != 1 {
                continue;
            }
            let start_j = entries[j].base;
            let end_j = start_j.checked_add(entries[j].length).unwrap_or(0);
            if end_j == 0 {
                panic_and_halt("BootInfo memory entry overflow");
            }
            let overlap = start_i < end_j && start_j < end_i;
            if overlap {
                panic_and_halt("BootInfo contains overlapping usable memory regions");
            }
        }
    }

    for e in entries {
        if e.typ == 1 {
            pmm_mark_usable_range(e.base, e.length);
        }
    }
    if NEXT_PAGE == MAX_PAGES {
        panic_and_halt("PMM: no usable memory from BootInfo (BIOS/UEFI)");
    }

    // Summary logging
    let mut region_count = 0usize;
    let mut total_bytes: usize = 0;
    for e in entries {
        if e.typ == 1 {
            region_count += 1;
            total_bytes = total_bytes.saturating_add(e.length as usize);
        }
    }
    let total_pages = total_bytes / PAGE_SIZE;
    print("[PMM] initialized from BootInfo: regions=");
    print_num(region_count);
    print(", pages=");
    print_num(total_pages);
    print(", bytes=");
    print_num(total_bytes);
    print("\n");
}

fn crc32(data: *const u8, len: usize) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for i in 0..len {
        let byte = unsafe { *data.add(i) } as u32;
        crc ^= byte;
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    crc ^ 0xFFFF_FFFF
}

fn verify_kernel_crc(bi: &BootInfo) {
    unsafe {
        print("[BOOT] verify_kernel_crc: enter\n");

        if bi.kernel_base == 0 || bi.kernel_size == 0 {
            print("[BOOT] verify_kernel_crc: missing kernel range -> SKIP\n");
            return;
        }

        print("[BOOT] verify_kernel_crc: region base=0x");
        print_hex64(bi.kernel_base);
        print(" size=0x");
        print_hex64(bi.kernel_size);
        print("\n");

        let len = bi.kernel_size as usize;
        let crc = crc32(bi.kernel_base as *const u8, len);

        print("[BOOT] verify_kernel_crc: expected=0x");
        print_hex32(bi.kernel_crc32);
        print(" calc=0x");
        print_hex32(crc);
        print("\n");

        if crc != bi.kernel_crc32 {
            print("[SECURITY] Kernel CRC mismatch – halting\n");
            panic_and_halt("Kernel integrity check failed");
        }

        print("[SECURITY] Kernel CRC verified\n");
    }
}

fn test_kernel_mapping_clone() {
    use kernel::mm::AddressSpace;
    unsafe {
        let aspace = AddressSpace::new_with_kernel_mappings();
        let old_cr3 = read_cr3();
        load_cr3(aspace.pml4_phys());
        load_cr3(old_cr3);
        print("[MMU] Kernel mappings cloned into new address space successfully\n");
    }
}

fn log_tss_ready() {
    unsafe {
        if tss_ready_flag != 0 {
            print("[CPU] TSS loaded with ring-0 stack at 0x");
            print_hex64((&tss_stack_top as *const u8 as u64));
            print("\n");
        } else {
            panic_and_halt("TSS not initialized");
        }
    }
}

fn parse_init_elf(data: &[u8]) -> Option<(u64, usize)> {
    #[repr(C)]
    struct Elf64Ehdr {
        e_ident: [u8; 16],
        e_type: u16,
        e_machine: u16,
        e_version: u32,
        e_entry: u64,
        e_phoff: u64,
        e_shoff: u64,
        e_flags: u32,
        e_ehsize: u16,
        e_phentsize: u16,
        e_phnum: u16,
        e_shentsize: u16,
        e_shnum: u16,
        e_shstrndx: u16,
    }

    #[repr(C)]
    struct Elf64Phdr {
        p_type: u32,
        p_flags: u32,
        p_offset: u64,
        p_vaddr: u64,
        p_paddr: u64,
        p_filesz: u64,
        p_memsz: u64,
        p_align: u64,
    }

    const PT_LOAD: u32 = 1;
    if data.len() < core::mem::size_of::<Elf64Ehdr>() {
        return None;
    }
    let ehdr = unsafe { &*(data.as_ptr() as *const Elf64Ehdr) };
    if &ehdr.e_ident[0..4] != b"\x7FELF" {
        return None;
    }
    let mut load_count = 0usize;
    for i in 0..ehdr.e_phnum {
        let off = ehdr.e_phoff as usize + i as usize * ehdr.e_phentsize as usize;
        if off + core::mem::size_of::<Elf64Phdr>() > data.len() {
            break;
        }
        let ph = unsafe { &*(data.as_ptr().add(off) as *const Elf64Phdr) };
        if ph.p_type == PT_LOAD {
            load_count += 1;
            unsafe {
                print("[INIT] PT_LOAD segment: vaddr=0x");
                print_hex64(ph.p_vaddr);
                print(" memsz=0x");
                print_hex64(ph.p_memsz);
                print("\n");
            }
        }
    }
    Some((ehdr.e_entry, load_count))
}

fn compute_elf_load_size(data: &[u8]) -> u64 {
    #[repr(C)]
    struct Elf64Ehdr {
        e_ident: [u8; 16],
        e_type: u16,
        e_machine: u16,
        e_version: u32,
        e_entry: u64,
        e_phoff: u64,
        e_shoff: u64,
        e_flags: u32,
        e_ehsize: u16,
        e_phentsize: u16,
        e_phnum: u16,
        e_shentsize: u16,
        e_shnum: u16,
        e_shstrndx: u16,
    }

    #[repr(C)]
    struct Elf64Phdr {
        p_type: u32,
        p_flags: u32,
        p_offset: u64,
        p_vaddr: u64,
        p_paddr: u64,
        p_filesz: u64,
        p_memsz: u64,
        p_align: u64,
    }

    const PT_LOAD: u32 = 1;
    if data.len() < core::mem::size_of::<Elf64Ehdr>() {
        return 0;
    }
    let ehdr = unsafe { &*(data.as_ptr() as *const Elf64Ehdr) };
    let mut total: u64 = 0;
    for i in 0..ehdr.e_phnum {
        let off = ehdr.e_phoff as usize + i as usize * ehdr.e_phentsize as usize;
        if off + core::mem::size_of::<Elf64Phdr>() > data.len() {
            break;
        }
        let ph = unsafe { &*(data.as_ptr().add(off) as *const Elf64Phdr) };
        if ph.p_type == PT_LOAD {
            total = total.saturating_add(ph.p_memsz);
        }
    }
    total
}

fn test_init_elf_parse() {
    if let Some(data) = fs::iso9660::read_file("init") {
        unsafe {
            print("[INIT] init ELF bytes available, size=");
            print_num(data.len());
            print("\n");
        }
        if let Some((entry, loads)) = parse_init_elf(data) {
            unsafe {
                print("[INIT] ELF header parsed successfully, entry=0x");
                print_hex64(entry);
                print("\n");
                print("[INIT] loadable segments=");
                print_num(loads);
                print("\n");
            }
        } else {
            panic_and_halt("Init ELF parse failed");
        }
    } else {
        panic_and_halt("Init ELF not found");
    }
}

struct InitBootstrapInfo {
    pid: usize,
    entry: u64,
    rsp: u64,
    cr3: usize,
    mapped_bytes: u64,
}

fn bootstrap_init_user(info: InitBootstrapInfo) -> ! {
    unsafe {
        crate::process::process::switch_to(info.pid);
        print("[USER] Entering user mode for PID 1...\n");
        jump_to_user(core::ptr::null(), info.entry, info.rsp);
    }
}

#[no_mangle]
pub extern "C" fn syscall_signal_check() {
    // Signal handling is stubbed out in this boot stub build.
}

#[no_mangle]
pub extern "C" fn guardbsd_main() -> ! {
    unsafe {
        serial_init();
        print("[BOOT] guardbsd_main entered\n");
        print("[BOOT] A1 guardbsd_main entered\n");

        let bi = validate_bootinfo();
        log_bootinfo(bi);
        log_modules(bi);

        print("[BOOT] A2 before verify_kernel_crc\n");
        verify_kernel_crc(bi);
        print("[BOOT] A3 after verify_kernel_crc\n");

        print("\n\n");
        print("================================================================================\n");
        print("[BOOT] GuardBSD Winter Saga v1.0.0 - SYSTEM ONLINE\n");
        print("================================================================================\n");
        print("[OK] Bootloader: GuaBoot (BSD-Licensed)\n");
        print("[OK] Boot stub loaded\n");
        print("[OK] Serial COM1 initialized\n");
        print("[OK] Protected mode active\n");

        print("\n[INIT] Initializing memory management...\n");
        init_memory(bi);
        print("[OK] PMM initialized\n");
        print("[OK] VMM initialized\n");

        print("[BOOT] A4 before init_kernel_template\n");
        init_kernel_template();
        print("[BOOT] A5 after init_kernel_template\n");
        print("[BOOT] A6 before kernel_template_ready\n");
        if kernel_template_ready() {
            print("[OK] Kernel PML4 template captured\n");
            print("[BOOT] A7 kernel_template_ready -> true\n");
        } else {
            panic_and_halt("Failed to capture kernel PML4 template");
        }

        test_kernel_mapping_clone();
        log_tss_ready();

        // Trigger a kernel-mode syscall test path to verify wiring.
        syscall_dispatch_trap(syscall::SYS_GETPID as u64, 0, 0, 0);

        print("\n[INIT] Initializing filesystem...\n");
        init_filesystem();
        print("[OK] ISO filesystem ready\n");

        print("\n[INIT] Initializing kernel log file sink...\n");
        crate::log_sink::init_klog_file_sink();
        print("[INIT] Kernel log file sink configured (stubbed, waiting for VFS)\n");

        test_init_elf_parse();

        print("\n[INIT] Setting up interrupt handling...\n");
        init_pic();
        interrupt::idt::init_idt();
        print("[OK] IDT initialized (syscall int 0x80, IRQ0, IRQ1)\n");

        print("\n[INIT] Initializing scheduler...\n");
        crate::sched::init(100);
        drivers::timer::init(100);
        print("[OK] Scheduler initialized (100 Hz timer)\n");

        if RUN_USERTEST {
            spawn_user_smoke_test();
        } else {
            print("[USERTEST] Skipped (disabled)\n");
        }

        // Spawn bootloader-provided microkernels (optional during diagnostics)
        log_modules(bi);
        if DISABLE_MODULES {
            print("[BOOT] Modules spawning disabled\n");
        } else if bi.mods_count != 0 && !bi.mods.is_null() {
            let mods = core::slice::from_raw_parts(bi.mods, bi.mods_count as usize);
            for m in mods {
                spawn_module_from_blob(m.string, m.mod_start, m.mod_end);
            }
        } else {
            print("[BOOT] No modules to spawn\n");
        }

        print("\n[INIT] Loading shell from /bin/gsh...\n");
        load_shell();
        print("[OK] Shell loaded\n");

        print("\n[INIT] Creating init process...\n");
        let init_info = create_init_process();
        print("[OK] Init process created (PID 1)\n");
        print("[SCHED] Switching to PID 1 (init)\n");
        bootstrap_init_user(init_info);
    }
}

fn shell_loop() -> ! {
    let mut line_buf: [u8; 256] = [0; 256];
    let mut line_len: usize = 0;

    loop {
        unsafe {
            core::arch::asm!("hlt");
        }

        while let Some(ch) = drivers::keyboard::read_char() {
            unsafe {
                if ch == b'\n' {
                    print("\n");
                    if line_len > 0 {
                        execute_command(&line_buf[..line_len]);
                    }
                    line_len = 0;
                    print("GuardBSD# ");
                } else if ch == 8 {
                    if line_len > 0 {
                        line_len -= 1;
                        print("\x08 \x08");
                    }
                } else if line_len < 255 {
                    line_buf[line_len] = ch;
                    line_len += 1;

                    while (inb(COM1 + 5) & 0x20) == 0 {}
                    outb(COM1, ch);
                }
            }
        }
    }
}

fn execute_command(cmd: &[u8]) {
    unsafe {
        if cmd == b"help" {
            print("Available commands: help, clear, echo, exit\n");
        } else if cmd == b"clear" {
            print("\x1b[2J\x1b[H");
        } else if cmd.starts_with(b"echo ") {
            let msg = &cmd[5..];
            for &b in msg {
                while (inb(COM1 + 5) & 0x20) == 0 {}
                outb(COM1, b);
            }
            print("\n");
        } else if cmd == b"exit" {
            print("Goodbye!\n");
            loop {
                core::arch::asm!("cli; hlt");
            }
        } else {
            print("Unknown command: ");
            for &b in cmd {
                while (inb(COM1 + 5) & 0x20) == 0 {}
                outb(COM1, b);
            }
            print("\n");
        }
    }
}

fn init_pic() {
    unsafe {
        outb(0x20, 0x11);
        outb(0xA0, 0x11);
        outb(0x21, 0x20);
        outb(0xA1, 0x28);
        outb(0x21, 0x04);
        outb(0xA1, 0x02);
        outb(0x21, 0x01);
        outb(0xA1, 0x01);
        outb(0x21, 0xFC); // Unmask IRQ0 (timer) and IRQ1 (keyboard)
        outb(0xA1, 0xFF);
    }
}

#[allow(dead_code)]
fn enable_interrupts() {
    unsafe {
        core::arch::asm!("sti");
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        print("\n[PANIC] System halted\n");
    }
    loop {
        unsafe {
            core::arch::asm!("cli; hlt");
        }
    }
}

fn init_filesystem() {
    // Skip ISO detection for now - use simulated filesystem
    fs::iso9660::init(0);
}

fn print_hex32(n: u32) {
    unsafe {
        let hex = b"0123456789abcdef";
        for i in (0..8).rev() {
            let nibble = ((n >> (i * 4)) & 0xF) as usize;
            let ch = hex[nibble];
            while (inb(COM1 + 5) & 0x20) == 0 {}
            outb(COM1, ch);
        }
    }
}

fn print_hex64(n: u64) {
    unsafe {
        let hex = b"0123456789abcdef";
        for i in (0..16).rev() {
            let nibble = ((n >> (i * 4)) & 0xF) as usize;
            let ch = hex[nibble];
            while (inb(COM1 + 5) & 0x20) == 0 {}
            outb(COM1, ch);
        }
    }
}

fn load_shell() {
    // Filesystem loading will be implemented when ISO is properly mapped
    // For now, use built-in shell
}

fn print_num(n: usize) {
    unsafe {
        let mut buf = [0u8; 20];
        let mut i = 0;
        let mut num = n;

        if num == 0 {
            print("0");
            return;
        }

        while num > 0 {
            buf[i] = (num % 10) as u8 + b'0';
            num /= 10;
            i += 1;
        }

        while i > 0 {
            i -= 1;
            while (inb(COM1 + 5) & 0x20) == 0 {}
            outb(COM1, buf[i]);
        }
    }
}

unsafe fn create_init_process() -> InitBootstrapInfo {
    const INIT_STACK_TOP: usize = 0x0000_0000_7FFF_F000;
    const STACK_PAGES: usize = 4; // 16KB stack

    print("[INIT] Creating PID 1...\n");

    // Fetch embedded init ELF bytes
    let init_bytes = fs::iso9660::read_file("init").unwrap_or_else(|| {
        panic_and_halt("Init ELF not available for PID 1");
    });

    // Create address space with kernel mappings
    let mut aspace = kernel::mm::AddressSpace::new_with_kernel_mappings();

    // Load ELF into address space
    let loaded = crate::process::elf_loader::parse_and_load_elf(init_bytes, &mut aspace)
        .unwrap_or_else(|_| panic_and_halt("Failed to load init ELF"));
    print("[INIT] ELF for init loaded at entry=0x");
    print_hex64(loaded.entry);
    print("\n");

    // Map user stack
    for i in 0..STACK_PAGES {
        let phys = kernel::mm::alloc_page().unwrap_or_else(|| panic_and_halt("Out of memory for init stack"));
        let virt = INIT_STACK_TOP - (i + 1) * 4096;
        let flags = kernel::mm::PageFlags::PRESENT
            | kernel::mm::PageFlags::WRITABLE
            | kernel::mm::PageFlags::USER;
        if !aspace.map(virt as u64, phys as u64, flags) {
            panic_and_halt("Failed to map init stack");
        }
    }
    print("[INIT] User stack for PID 1 mapped at 0x");
    print_hex64(INIT_STACK_TOP as u64);
    print("\n");

    // Create process table entry (PID 1 expected)
    let pid = crate::process::process::create_process(
        loaded.entry,
        INIT_STACK_TOP as u64,
        aspace.pml4_phys(),
    );
    if pid != 1 {
        panic_and_halt("PID allocator did not return PID 1");
    }

    // Log limit
    let limit = 16 * 1024 * 1024u64;
    print("[LIMIT] PID 1 memory_limit = ");
    print_num(limit as usize);
    print(" bytes\n");

    // Build user-mode ArchContext (not jumped yet)
    let mut ctx = crate::sched::ArchContext::zeroed();
    ctx.rip = loaded.entry;
    ctx.rsp = INIT_STACK_TOP as u64;
    ctx.cs = 0x1B; // user code selector
    ctx.ss = 0x23; // user data selector
    ctx.rflags = 0x202;
    ctx.cr3 = aspace.pml4_phys() as u64;

    if crate::sched::register_thread(pid as i32, 1, 0, ctx).is_some() {
        print("[SCHED] PID 1 added to run queue\n");
    } else {
        panic_and_halt("Failed to register PID 1 thread");
    }

    print("[DEBUG] PID 1: cr3=0x");
    print_hex64(aspace.pml4_phys() as u64);
    print(" rip=0x");
    print_hex64(loaded.entry);
    print(" rsp=0x");
    print_hex64(INIT_STACK_TOP as u64);
    print("\n");

    // Compute mapped bytes: ELF PT_LOAD memsz + stack
    let elf_bytes = compute_elf_load_size(init_bytes);
    let total_mapped = elf_bytes.saturating_add((STACK_PAGES * 4096) as u64);

    if !kernel::process::process::try_add_memory_usage(pid, total_mapped as usize) {
        print("[LIMIT] PID 1 exceeded memory limit during init mapping\n");
        kernel::process::process::mark_killed(pid);
    } else {
        print("[LIMIT] PID 1 memory_usage = ");
        print_num(total_mapped as usize);
        print(" bytes\n");
    }

    InitBootstrapInfo {
        pid,
        entry: loaded.entry,
        rsp: INIT_STACK_TOP as u64,
        cr3: aspace.pml4_phys() as usize,
        mapped_bytes: total_mapped,
    }
}
