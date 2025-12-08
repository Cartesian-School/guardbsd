// Filesystem Syscall Implementations
// BSD 3-Clause License
// File descriptor syscalls: open, read, write, close

#![no_std]

extern crate alloc;

use crate::process::types::{Process, Pid, FileDescriptor, MAX_FD_PER_PROCESS};

// Process table access (from process.rs)
extern "C" {
    static mut PROCESS_TABLE: [Option<Process>; 64];
    static mut CURRENT_PROCESS: Option<Pid>;
}

/// Get current process ID
unsafe fn get_current_pid() -> Option<Pid> {
    CURRENT_PROCESS
}

/// Find process by PID (mutable reference)
fn find_process_mut(pid: Pid) -> Option<&'static mut Process> {
    unsafe {
        for slot in PROCESS_TABLE.iter_mut() {
            if let Some(proc) = slot {
                if proc.pid == pid {
                    return Some(proc);
                }
            }
        }
    }
    None
}

// Error codes (POSIX compatible)
const EBADF: isize = -9;    // Bad file descriptor
const ENOENT: isize = -2;   // No such file or directory
const EINVAL: isize = -22;  // Invalid argument
const ENOMEM: isize = -12;  // Out of memory
const EMFILE: isize = -24;  // Too many open files
const EFAULT: isize = -14;  // Bad address

// Special inode numbers for stdio
const INODE_STDIN: u64 = 0;
const INODE_STDOUT: u64 = 1;
const INODE_STDERR: u64 = 2;

/// Open a file and return a file descriptor
/// Returns fd number on success, negative error code on failure
pub fn sys_open(path: *const u8, flags: u32) -> isize {
    unsafe {
        // Get current process
        let pid = match get_current_pid() {
            Some(p) => p,
            None => return EINVAL,
        };
        
        let proc = match find_process_mut(pid) {
            Some(p) => p,
            None => return EINVAL,
        };
        
        // Validate path pointer
        if path.is_null() {
            return EFAULT;
        }
        
        // Copy path string from userspace
        let mut path_buf = [0u8; 256];
        let mut i = 0;
        while i < 255 {
            let byte = *path.add(i);
            if byte == 0 {
                break;
            }
            path_buf[i] = byte;
            i += 1;
        }
        let path_len = i;
        
        if path_len == 0 {
            return ENOENT;
        }
        
        let path_str = match core::str::from_utf8(&path_buf[..path_len]) {
            Ok(s) => s,
            Err(_) => return EINVAL,
        };
        
        // Try to open file from ISO9660 filesystem
        let file_data = crate::fs::iso9660::read_file(path_str);
        
        let inode = if let Some(data) = file_data {
            // File exists in ISO - use its address as inode
            data.as_ptr() as u64
        } else {
            // File not found
            return ENOENT;
        };
        
        // Allocate file descriptor
        let fd_num = proc.alloc_fd();
        if fd_num.is_none() {
            return EMFILE;
        }
        let fd_num = fd_num.unwrap();
        
        // Store file descriptor
        proc.fd_table[fd_num] = Some(FileDescriptor {
            inode,
            offset: 0,
            flags,
        });
        
        fd_num as isize
    }
}

/// Read from a file descriptor
/// Returns number of bytes read on success, negative error code on failure
pub fn sys_read(fd: u32, buf: *mut u8, len: usize) -> isize {
    unsafe {
        // Get current process
        let pid = match get_current_pid() {
            Some(p) => p,
            None => return EINVAL,
        };
        
        let proc = match find_process_mut(pid) {
            Some(p) => p,
            None => return EINVAL,
        };
        
        // Validate buffer pointer
        if buf.is_null() {
            return EFAULT;
        }
        
        // Check fd bounds
        if fd as usize >= MAX_FD_PER_PROCESS {
            return EBADF;
        }
        
        // Special handling for stdin (fd 0)
        if fd == 0 {
            // stdin not yet implemented - return 0 (EOF)
            return 0;
        }
        
        // Get file descriptor
        let fd_info = match &proc.fd_table[fd as usize] {
            Some(info) => *info,
            None => return EBADF,
        };
        
        // Read from file (using inode as data pointer)
        // This works for ISO9660 files where inode is the data address
        let file_data = fd_info.inode as *const u8;
        let offset = fd_info.offset as usize;
        
        // For regular files from ISO, we need to know the size
        // For now, read what's requested (caller should know size)
        // In a real implementation, we'd track file size
        
        // Read data
        let bytes_to_read = len;
        core::ptr::copy_nonoverlapping(
            file_data.add(offset),
            buf,
            bytes_to_read
        );
        
        // Update offset
        if let Some(fd_entry) = &mut proc.fd_table[fd as usize] {
            fd_entry.offset += bytes_to_read as u64;
        }
        
        bytes_to_read as isize
    }
}

/// Write to a file descriptor
/// Returns number of bytes written on success, negative error code on failure
pub fn sys_write(fd: u32, buf: *const u8, len: usize) -> isize {
    unsafe {
        // Get current process
        let pid = match get_current_pid() {
            Some(p) => p,
            None => return EINVAL,
        };
        
        let proc = match find_process_mut(pid) {
            Some(p) => p,
            None => return EINVAL,
        };
        
        // Validate buffer pointer
        if buf.is_null() {
            return EFAULT;
        }
        
        // Special handling for stdout (fd 1) and stderr (fd 2)
        if fd == 1 || fd == 2 {
            // Write to serial console
            let slice = core::slice::from_raw_parts(buf, len);
            for &byte in slice {
                serial_putc(byte);
            }
            return len as isize;
        }
        
        // Check fd bounds
        if fd as usize >= MAX_FD_PER_PROCESS {
            return EBADF;
        }
        
        // Get file descriptor
        let fd_info = match &proc.fd_table[fd as usize] {
            Some(info) => *info,
            None => return EBADF,
        };
        
        // Files from ISO are read-only, so write fails
        // In the future, this would write to writable filesystems
        return EINVAL;
    }
}

/// Close a file descriptor
/// Returns 0 on success, negative error code on failure
pub fn sys_close(fd: u32) -> isize {
    unsafe {
        // Get current process
        let pid = match get_current_pid() {
            Some(p) => p,
            None => return EINVAL,
        };
        
        let proc = match find_process_mut(pid) {
            Some(p) => p,
            None => return EINVAL,
        };
        
        // Check fd bounds
        if fd as usize >= MAX_FD_PER_PROCESS {
            return EBADF;
        }
        
        // Don't allow closing stdin/stdout/stderr
        if fd <= 2 {
            return EINVAL;
        }
        
        // Free file descriptor
        if proc.free_fd(fd as usize) {
            0
        } else {
            EBADF
        }
    }
}

// Serial port helper (for stdout/stderr)
const COM1: u16 = 0x3F8;

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

