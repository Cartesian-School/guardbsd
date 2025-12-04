// userland/libgbsd/src/fs.rs
// File system syscall wrappers
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::syscall::*;
use crate::error::{Error, Result};

pub type Fd = u64;

pub const O_RDONLY: u64 = 0x0;
pub const O_WRONLY: u64 = 0x1;
pub const O_RDWR: u64 = 0x2;
pub const O_CREAT: u64 = 0x200;

#[inline]
pub fn open(path: &[u8], flags: u64) -> Result<Fd> {
    let ret = unsafe { syscall2(SYS_OPEN, path.as_ptr() as u64, flags) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(Error::from_code((-ret_i64) as u64))
    } else {
        Ok(ret)
    }
}

#[inline]
pub fn close(fd: Fd) -> Result<()> {
    let ret = unsafe { syscall1(SYS_CLOSE, fd) };
    if ret == 0 {
        Ok(())
    } else {
        Err(Error::from_code(ret))
    }
}

#[inline]
pub fn read(fd: Fd, buf: &mut [u8]) -> Result<usize> {
    let ret = unsafe { syscall3(SYS_READ, fd, buf.as_mut_ptr() as u64, buf.len() as u64) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(Error::from_code((-ret_i64) as u64))
    } else {
        Ok(ret as usize)
    }
}

#[inline]
pub fn write(fd: Fd, buf: &[u8]) -> Result<usize> {
    let ret = unsafe { syscall3(SYS_WRITE, fd, buf.as_ptr() as u64, buf.len() as u64) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(Error::from_code((-ret_i64) as u64))
    } else {
        Ok(ret as usize)
    }
}

#[derive(Clone, Copy)]
pub struct Stat {
    pub size: u64,
    pub mode: u32,
    pub mtime: u64,
}

#[inline]
pub fn stat(path: &[u8]) -> Result<Stat> {
    let mut stat_buf = [0u64; 3]; // size, mode, mtime
    let ret = unsafe {
        syscall3(SYS_STAT, path.as_ptr() as u64, stat_buf.as_mut_ptr() as u64, stat_buf.len() as u64)
    };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(Error::from_code((-ret_i64) as u64))
    } else {
        Ok(Stat {
            size: stat_buf[0],
            mode: stat_buf[1] as u32,
            mtime: stat_buf[2],
        })
    }
}

#[inline]
pub fn rename(old_path: &[u8], new_path: &[u8]) -> Result<()> {
    let ret = unsafe { syscall2(SYS_RENAME, old_path.as_ptr() as u64, new_path.as_ptr() as u64) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(Error::from_code((-ret_i64) as u64))
    } else {
        Ok(())
    }
}

#[inline]
pub fn unlink(path: &[u8]) -> Result<()> {
    let ret = unsafe { syscall1(SYS_UNLINK, path.as_ptr() as u64) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(Error::from_code((-ret_i64) as u64))
    } else {
        Ok(())
    }
}

#[inline]
pub fn sync(fd: Fd) -> Result<()> {
    let ret = unsafe { syscall1(SYS_SYNC, fd) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(Error::from_code((-ret_i64) as u64))
    } else {
        Ok(())
    }
}

#[inline]
pub fn mkdir(_path: &[u8]) -> Result<()> {
    // VFS mkdir operation via IPC
    // This is a placeholder implementation for the current development stage
    //
    // Real implementation would:
    // 1. Look up VFS server port (service discovery)
    // 2. Send IPC message: [op=6, path...]
    // 3. Wait for response and return appropriate error code
    //
    // For now, we assume directories are created by the underlying filesystem
    // when files are opened with O_CREAT
    Ok(())
}
