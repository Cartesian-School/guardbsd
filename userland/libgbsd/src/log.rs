// userland/libgbsd/src/log.rs
// Kernel logging syscall bridge
// ============================================================================

pub use kernel_log::{LogLevel, UserLogRecord, LOG_MSG_MAX, LOG_RING_SIZE, LOG_SUBSYS_MAX};
use crate::syscall;

/// Read raw kernel log bytes into the provided buffer.
/// Returns number of bytes copied or negative errno.
#[inline(always)]
pub fn log_read(buf: &mut [u8]) -> isize {
    if buf.is_empty() {
        return 0;
    }
    let ret: isize;
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") syscall::SYS_LOG_READ as u64,
            in("rdi") buf.as_mut_ptr(),
            in("rsi") buf.len() as u64,
            lateout("rax") ret,
            options(nostack)
        );
    }
    ret
}
