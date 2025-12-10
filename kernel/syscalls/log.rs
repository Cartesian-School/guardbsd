// Logging syscalls
// BSD 3-Clause License
// Reads kernel log bytes from the in-memory log backend ring into userspace.

#![no_std]

/// Read kernel log ring buffer into user-space buffer.
///
/// Syscall ABI:
///   rax = SYS_LOG_READ
///   rdi = user_buf (userspace pointer)
///   rsi = len (max bytes to copy)
/// Return:
///   >= 0  -> number of bytes copied
///   <  0  -> -EINVAL on bad pointer/len
pub fn sys_log_read(user_buf: *mut u8, len: usize) -> isize {
    // 1) basic checks
    if user_buf.is_null() || len == 0 {
        return 0;
    }

    // 2) simple user-space address validation (matches signal path boundary)
    const KERNEL_CANONICAL_BASE: u64 = 0xFFFF_8000_0000_0000;
    let addr = user_buf as u64;
    if addr >= KERNEL_CANONICAL_BASE {
        // EINVAL
        return -22;
    }

    // 3) cap read size to ring size
    const LOG_READ_MAX: usize = 4096;
    let to_copy = core::cmp::min(len, LOG_READ_MAX);

    // 4) local kernel buffer
    let mut kbuf = [0u8; LOG_READ_MAX];

    // 5) pull bytes from log backend ring
    let n = kernel_log::log_backend::copy_mem(&mut kbuf[..to_copy]);
    if n == 0 {
        return 0;
    }

    // 6) copy into user buffer
    unsafe {
        core::ptr::copy_nonoverlapping(kbuf.as_ptr(), user_buf, n);
    }

    n as isize
}
