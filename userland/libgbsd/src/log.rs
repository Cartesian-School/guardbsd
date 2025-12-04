// userland/libgbsd/src/log.rs
// Kernel logging syscall bridge
// ============================================================================

use crate::error::{Error, Result};
pub use kernel_log::{LogLevel, UserLogRecord, LOG_MSG_MAX, LOG_RING_SIZE, LOG_SUBSYS_MAX};

// Kernel logging syscalls are reserved but not implemented yet.

/// # Errors
///
/// Always returns `Error::NoSys` as kernel logging is not implemented.
#[inline]
pub fn read_kernel_logs(_buf: &mut [UserLogRecord]) -> Result<usize> {
    Err(Error::NoSys)
}

/// # Errors
///
/// Always returns `Error::NoSys` as kernel logging is not implemented.
#[inline]
pub fn ack_kernel_logs(_count: usize) -> Result<()> {
    Err(Error::NoSys)
}

/// # Errors
///
/// Always returns `Error::NoSys` as kernel logging is not implemented.
#[inline]
pub fn register_kernel_log_daemon(_pid: u64) -> Result<()> {
    Err(Error::NoSys)
}
