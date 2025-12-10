// kernel/fs/kfile.rs
// Kernel-internal file API skeleton (no VFS implementation yet)

#![allow(dead_code)]

/// Kernel-internal file descriptor type (opaque).
#[derive(Copy, Clone, Debug)]
pub struct KFile {
    fd: i32,
}

/// Error codes for kernel-internal file operations.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KfError {
    NotImplemented, // ENOSYS placeholder
    IoError,
    InvalidPath,
}

bitflags::bitflags! {
    /// Open flags for kernel-internal files.
    pub struct KfOpenFlags: u32 {
        const READ   = 0x1;
        const WRITE  = 0x2;
        const APPEND = 0x4;
        const CREATE = 0x8;
    }
}

/// Open a kernel-internal file.
///
/// Stubbed to Err(NotImplemented) until VFS is wired.
pub fn kf_open(_path: &str, _flags: KfOpenFlags, _mode: u32) -> Result<KFile, KfError> {
    Err(KfError::NotImplemented)
}

/// Append bytes to an already opened file.
///
/// Stubbed to Err(NotImplemented) until VFS is wired.
pub fn kf_write_all(_file: KFile, _data: &[u8]) -> Result<(), KfError> {
    Err(KfError::NotImplemented)
}
