// userland/libgbsd/src/fs.rs
// File system syscall wrappers
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::syscall::{syscall1, syscall2, syscall3, SYS_OPEN, SYS_CLOSE, SYS_READ, SYS_WRITE, SYS_STAT, SYS_MKDIR, SYS_UNLINK, SYS_RENAME, SYS_SYNC, SYS_CHDIR, SYS_GETCWD, SYS_MOUNT, SYS_UMOUNT};
use crate::error::{Error, Result};

pub type Fd = u64;

pub const O_RDONLY: u64 = 0x0;
pub const O_WRONLY: u64 = 0x1;
pub const O_RDWR: u64 = 0x2;
pub const O_CREAT: u64 = 0x200;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Stat {
    pub dev: u64,      // device
    pub ino: u64,      // inode
    pub mode: u32,     // protection
    pub nlink: u32,    // number of hard links
    pub uid: u32,      // user ID of owner
    pub gid: u32,      // group ID of owner
    pub rdev: u64,     // device type (if inode device)
    pub size: u64,     // total size, in bytes
    pub blksize: u32,  // blocksize for filesystem I/O
    pub blocks: u64,   // number of 512B blocks allocated
    pub atime: u64,    // time of last access
    pub mtime: u64,    // time of last modification
    pub ctime: u64,    // time of last status change
}

// Filesystem syscalls are reserved in the kernel but not implemented yet.

/// # Errors
///
/// Returns an error if the system call fails.
#[inline]
pub fn open(path: &[u8], flags: u64) -> Result<Fd> {
    let ret = unsafe { syscall2(SYS_OPEN as u64, path.as_ptr() as u64, flags) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(Error::from_code((-ret_i64) as u64))
    } else {
        Ok(ret as Fd)
    }
}

/// # Errors
///
/// Returns an error if the system call fails.
#[inline]
pub fn close(fd: Fd) -> Result<()> {
    let ret = unsafe { syscall1(SYS_CLOSE as u64, fd) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(Error::from_code((-ret_i64) as u64))
    } else {
        Ok(())
    }
}

/// # Errors
///
/// Returns an error if the system call fails or if the file descriptor is invalid.
#[inline]
pub fn read(fd: Fd, buf: &mut [u8]) -> Result<usize> {
    let ret = unsafe { syscall3(SYS_READ as u64, fd, buf.as_mut_ptr() as u64, buf.len() as u64) };
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
    let ret = unsafe { syscall3(SYS_WRITE as u64, fd, buf.as_ptr() as u64, buf.len() as u64) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(Error::from_code((-ret_i64) as u64))
    } else {
        Ok(usize::try_from(ret).unwrap_or(0))
    }
}

/// # Errors
///
/// Returns an error if the system call fails.
#[inline]
pub fn mkdir(path: &[u8]) -> Result<()> {
    let ret = unsafe { syscall2(SYS_MKDIR as u64, path.as_ptr() as u64, 0o755) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(Error::from_code((-ret_i64) as u64))
    } else {
        Ok(())
    }
}

/// # Errors
///
/// Returns an error if the system call fails.
#[inline]
pub fn stat(path: &[u8]) -> Result<Stat> {
    let mut stat_buf = Stat {
        dev: 0, ino: 0, mode: 0, nlink: 0, uid: 0, gid: 0, rdev: 0,
        size: 0, blksize: 0, blocks: 0, atime: 0, mtime: 0, ctime: 0,
    };
    let ret = unsafe { syscall2(SYS_STAT as u64, path.as_ptr() as u64, &mut stat_buf as *mut Stat as u64) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(Error::from_code((-ret_i64) as u64))
    } else {
        Ok(stat_buf)
    }
}

/// # Errors
///
/// Returns an error if the system call fails.
#[inline]
pub fn rename(old_path: &[u8], new_path: &[u8]) -> Result<()> {
    let ret = unsafe { syscall2(SYS_RENAME as u64, old_path.as_ptr() as u64, new_path.as_ptr() as u64) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(Error::from_code((-ret_i64) as u64))
    } else {
        Ok(())
    }
}

/// # Errors
///
/// Returns an error if the system call fails.
#[inline]
pub fn unlink(path: &[u8]) -> Result<()> {
    let ret = unsafe { syscall1(SYS_UNLINK as u64, path.as_ptr() as u64) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(Error::from_code((-ret_i64) as u64))
    } else {
        Ok(())
    }
}

/// # Errors
///
/// Returns an error if the system call fails.
#[inline]
pub fn sync(fd: Fd) -> Result<()> {
    let ret = unsafe { syscall1(SYS_SYNC as u64, fd) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(Error::from_code((-ret_i64) as u64))
    } else {
        Ok(())
    }
}

/// # Errors
///
/// Returns an error if the system call fails.
#[inline]
pub fn chdir(path: &[u8]) -> Result<()> {
    let ret = unsafe { syscall1(SYS_CHDIR as u64, path.as_ptr() as u64) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(Error::from_code((-ret_i64) as u64))
    } else {
        Ok(())
    }
}

/// # Errors
///
/// Returns an error if the system call fails.
#[inline]
pub fn getcwd(buf: &mut [u8]) -> Result<usize> {
    let ret = unsafe { syscall2(SYS_GETCWD as u64, buf.as_mut_ptr() as u64, buf.len() as u64) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(Error::from_code((-ret_i64) as u64))
    } else {
        Ok(ret as usize)
    }
}

/// # Errors
///
/// Returns an error if the system call fails.
#[inline]
pub fn mount(source: &[u8], target: &[u8], fstype: &[u8]) -> Result<()> {
    let ret = unsafe { syscall3(SYS_MOUNT as u64, source.as_ptr() as u64, target.as_ptr() as u64, fstype.as_ptr() as u64) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(Error::from_code((-ret_i64) as u64))
    } else {
        Ok(())
    }
}

/// # Errors
///
/// Returns an error if the system call fails.
#[inline]
pub fn umount(target: &[u8]) -> Result<()> {
    let ret = unsafe { syscall1(SYS_UMOUNT as u64, target.as_ptr() as u64) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(Error::from_code((-ret_i64) as u64))
    } else {
        Ok(())
    }
}
