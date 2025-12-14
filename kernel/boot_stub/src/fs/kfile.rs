//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: boot_stub
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Wewnętrzne API plików boot stuba oparte na VFS poprzez wywołania systemowe.

#![allow(dead_code)]

use shared::syscall_numbers::*;
const MAX_WRITE_CHUNK: usize = 4000; // consistent with prior VFS helpers

/// Kernel-internal file descriptor type (opaque).
#[derive(Copy, Clone, Debug)]
pub struct KFile {
    fd: i32,
}

impl KFile {
    #[inline]
    pub const fn fd(&self) -> i32 {
        self.fd
    }
}

/// Error codes for kernel-internal file operations.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KfError {
    NotImplemented, // VFS unavailable
    NoSuchFile,
    AccessDenied,
    IoError,
    InvalidFlags,
    VfsDown,
    Unknown(i32),
}

bitflags::bitflags! {
    /// Open flags for kernel-internal files.
    pub struct KfOpenFlags: u32 {
        const READ   = 0x1;
        const WRITE  = 0x2;
        const CREATE = 0x4;
        const TRUNC  = 0x8;
        const APPEND = 0x10;
    }
}

fn map_errno_to_kf_error(errno: i32) -> KfError {
    match errno {
        -2 => KfError::NoSuchFile,    // ENOENT
        -13 => KfError::AccessDenied, // EACCES
        -5 => KfError::IoError,       // EIO
        -95 => KfError::InvalidFlags, // EOPNOTSUPP
        -6 => KfError::VfsDown,       // arbitrary: service missing
        _ => KfError::Unknown(errno),
    }
}

fn syscall_open(path_cstr: *const u8, flags: u32, mode: u32) -> i32 {
    let ret: isize;
    unsafe {
        ret = fs_syscall3(
            SYS_OPEN as usize,
            path_cstr as usize,
            flags as usize,
            mode as usize,
        );
    }
    ret as i32
}

fn syscall_write(fd: i32, buf: *const u8, len: usize) -> i32 {
    let ret: isize;
    unsafe {
        ret = fs_syscall3(SYS_WRITE as usize, fd as usize, buf as usize, len);
    }
    ret as i32
}

fn syscall_mkdir(path_cstr: *const u8) -> i32 {
    let ret: isize;
    unsafe {
        ret = fs_syscall3(SYS_MKDIR as usize, path_cstr as usize, 0, 0);
    }
    ret as i32
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn fs_syscall3(num: usize, arg1: usize, arg2: usize, arg3: usize) -> isize {
    let ret: i64;
    core::arch::asm!(
        "syscall",
        in("rax") num as u64,
        in("rdi") arg1 as u64,
        in("rsi") arg2 as u64,
        in("rdx") arg3 as u64,
        lateout("rax") ret,
        options(nostack, preserves_flags),
    );
    ret as isize
}

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn fs_syscall3(num: usize, arg1: usize, arg2: usize, arg3: usize) -> isize {
    let ret: i32;
    core::arch::asm!(
        "int 0x80",
        in("eax") num as u32,
        in("ebx") arg1 as u32,
        in("ecx") arg2 as u32,
        in("edx") arg3 as u32,
        lateout("eax") ret,
        options(nostack),
    );
    ret as isize
}

fn vfs_open(path: &str, flags: u32, mode: u32) -> Result<i32, i32> {
    let mut path_buf = [0u8; 256];
    let bytes = path.as_bytes();
    let len = core::cmp::min(bytes.len(), 255);
    path_buf[..len].copy_from_slice(&bytes[..len]);
    path_buf[len] = 0;

    let ret = syscall_open(path_buf.as_ptr(), flags, mode);
    if ret >= 0 {
        Ok(ret)
    } else {
        Err(ret)
    }
}

fn vfs_write(fd: i32, data: &[u8]) -> Result<usize, i32> {
    let chunk_len = core::cmp::min(data.len(), MAX_WRITE_CHUNK);
    let ret = syscall_write(fd, data.as_ptr(), chunk_len);
    if ret >= 0 {
        Ok(ret as usize)
    } else {
        Err(ret)
    }
}

fn vfs_mkdir(path: &str) -> Result<(), i32> {
    let mut path_buf = [0u8; 256];
    let bytes = path.as_bytes();
    let len = core::cmp::min(bytes.len(), 255);
    path_buf[..len].copy_from_slice(&bytes[..len]);
    path_buf[len] = 0;

    let ret = syscall_mkdir(path_buf.as_ptr());
    if ret >= 0 {
        Ok(())
    } else {
        Err(ret)
    }
}

/// Open a kernel-internal file via VFS.
pub fn kf_open(path: &str, flags: KfOpenFlags, mode: u32) -> Result<KFile, KfError> {
    match vfs_open(path, flags.bits(), mode) {
        Ok(fd) => Ok(KFile { fd }),
        Err(errno) => Err(map_errno_to_kf_error(errno)),
    }
}

/// Append bytes to an already opened file via VFS.
pub fn kf_write_all(file: KFile, data: &[u8]) -> Result<(), KfError> {
    let mut written = 0usize;
    while written < data.len() {
        let chunk = &data[written..];
        let res = vfs_write(file.fd, chunk);
        match res {
            Ok(0) => return Err(KfError::IoError),
            Ok(n) => {
                written += n;
            }
            Err(errno) => return Err(map_errno_to_kf_error(errno)),
        }
    }
    Ok(())
}

/// Best-effort directory creation for log path.
pub fn kf_mkdir(path: &str) -> Result<(), KfError> {
    match vfs_mkdir(path) {
        Ok(()) => Ok(()),
        Err(errno) => Err(map_errno_to_kf_error(errno)),
    }
}
