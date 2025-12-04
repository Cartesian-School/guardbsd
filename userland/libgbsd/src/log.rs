// userland/libgbsd/src/log.rs
// Kernel logging syscall bridge
// ============================================================================

use crate::error::{Error, Result};
use crate::syscall::*;
pub use kernel_log::{LogLevel, UserLogRecord, LOG_MSG_MAX, LOG_RING_SIZE, LOG_SUBSYS_MAX};

#[inline]
pub fn read_kernel_logs(buf: &mut [UserLogRecord]) -> Result<usize> {
    if buf.is_empty() {
        return Ok(0);
    }
    let ret = unsafe { syscall2(SYS_LOG_READ, buf.as_mut_ptr() as u64, buf.len() as u64) };
    decode_result(ret).map(|v| v as usize)
}

#[inline]
pub fn ack_kernel_logs(count: usize) -> Result<()> {
    let ret = unsafe { syscall1(SYS_LOG_ACK, count as u64) };
    decode_result(ret).map(|_| ())
}

#[inline]
pub fn register_kernel_log_daemon(pid: u64) -> Result<()> {
    let ret = unsafe { syscall1(SYS_LOG_REGISTER_DAEMON, pid) };
    decode_result(ret).map(|_| ())
}

fn decode_result(ret: u64) -> Result<u64> {
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(Error::from_code((-ret_i64) as u64))
    } else if ret >= 0xFFFF_FFFF_0000_0000 {
        Err(Error::from_code(ret))
    } else {
        Ok(ret)
    }
}
