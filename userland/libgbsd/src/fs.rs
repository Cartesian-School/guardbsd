// userland/libgbsd/src/fs.rs
// File system syscall wrappers
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::syscall::{syscall3, SYS_READ, SYS_WRITE};
use crate::error::{Error, Result};

pub type Fd = u64;

pub const O_RDONLY: u64 = 0x0;
pub const O_WRONLY: u64 = 0x1;
pub const O_RDWR: u64 = 0x2;
pub const O_CREAT: u64 = 0x200;

#[derive(Clone, Copy)]
pub struct Stat {
    pub size: u64,
    pub mode: u32,
    pub mtime: u64,
}

// Filesystem syscalls are reserved in the kernel but not implemented yet.

/// # Errors
///
/// Always returns `Error::NoSys` as filesystem operations are not implemented.
#[inline]
pub fn open(_path: &[u8], _flags: u64) -> Result<Fd> {
    Err(Error::NoSys)
}

/// # Errors
///
/// Always returns `Error::NoSys` as filesystem operations are not implemented.
#[inline]
pub fn close(_fd: Fd) -> Result<()> {
    Err(Error::NoSys)
}

/// # Errors
///
/// Returns an error if the system call fails or if the file descriptor is invalid.
#[inline]
pub fn read(fd: Fd, buf: &mut [u8]) -> Result<usize> {
    let ret = unsafe { syscall3(SYS_READ, fd, buf.as_mut_ptr() as u64, buf.len() as u64) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(Error::from_code((-ret_i64) as u64))
    } else {
        Ok(usize::try_from(ret).unwrap_or(0))
    }
}

/// # Errors
///
/// Returns an error if the system call fails or if the file descriptor is invalid.
#[inline]
pub fn write(fd: Fd, buf: &[u8]) -> Result<usize> {
    let ret = unsafe { syscall3(SYS_WRITE, fd, buf.as_ptr() as u64, buf.len() as u64) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(Error::from_code((-ret_i64) as u64))
    } else {
        Ok(usize::try_from(ret).unwrap_or(0))
    }
}

/// # Errors
///
/// Always returns `Error::NoSys` as filesystem operations are not implemented.
#[inline]
pub fn mkdir(_path: &[u8]) -> Result<()> {
    Err(Error::NoSys)
}

/// # Errors
///
/// Always returns `Error::NoSys` as filesystem operations are not implemented.
#[inline]
pub fn stat(_path: &[u8]) -> Result<Stat> {
    Err(Error::NoSys)
}

/// # Errors
///
/// Always returns `Error::NoSys` as filesystem operations are not implemented.
#[inline]
pub fn rename(_old_path: &[u8], _new_path: &[u8]) -> Result<()> {
    Err(Error::NoSys)
}

/// # Errors
///
/// Always returns `Error::NoSys` as filesystem operations are not implemented.
#[inline]
pub fn unlink(_path: &[u8]) -> Result<()> {
    Err(Error::NoSys)
}

/// # Errors
///
/// Always returns `Error::NoSys` as filesystem operations are not implemented.
#[inline]
pub fn sync(_fd: Fd) -> Result<()> {
    Err(Error::NoSys)
}
