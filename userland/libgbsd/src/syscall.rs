//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: libgbsd
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Interfejs wywołań systemowych w stylu BSD.

// Import canonical syscall numbers from shared module
// This ensures userland and kernel always agree on syscall numbers
include!("../../../shared/syscall_numbers.rs");

// Re-export as u64 for compatibility with syscall wrappers
pub const SYS_EXIT_U64: u64 = SYS_EXIT as u64;
pub const SYS_FORK_U64: u64 = SYS_FORK as u64;
pub const SYS_EXEC_U64: u64 = SYS_EXEC as u64;
pub const SYS_WAIT_U64: u64 = SYS_WAIT as u64;
pub const SYS_GETPID_U64: u64 = SYS_GETPID as u64;
pub const SYS_KILL_U64: u64 = SYS_KILL as u64;
pub const SYS_YIELD_U64: u64 = SYS_YIELD as u64;
pub const SYS_READ_U64: u64 = SYS_READ as u64;
pub const SYS_WRITE_U64: u64 = SYS_WRITE as u64;
pub const SYS_OPEN_U64: u64 = SYS_OPEN as u64;
pub const SYS_CLOSE_U64: u64 = SYS_CLOSE as u64;
pub const SYS_DUP_U64: u64 = SYS_DUP as u64;
pub const SYS_DUP2_U64: u64 = SYS_DUP2 as u64;
pub const SYS_WAITPID_U64: u64 = SYS_WAITPID as u64;
pub const SYS_SETPGID_U64: u64 = SYS_SETPGID as u64;
pub const SYS_GETPGID_U64: u64 = SYS_GETPGID as u64;
pub const SYS_TCSETPGRP_U64: u64 = SYS_TCSETPGRP as u64;
pub const SYS_TCGETPGRP_U64: u64 = SYS_TCGETPGRP as u64;

use crate::error::Result;

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn syscall0(n: u64) -> u64 {
    let ret: u64;
    core::arch::asm!(
        "int 0x80",
        in("rax") n,
        lateout("rax") ret,
        options(nostack)
    );
    ret
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn syscall1(n: u64, arg1: u64) -> u64 {
    let ret: u64;
    core::arch::asm!(
        "int 0x80",
        in("rax") n,
        in("rdi") arg1,
        lateout("rax") ret,
        options(nostack)
    );
    ret
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn syscall2(n: u64, arg1: u64, arg2: u64) -> u64 {
    let ret: u64;
    core::arch::asm!(
        "int 0x80",
        in("rax") n,
        in("rdi") arg1,
        in("rsi") arg2,
        lateout("rax") ret,
        options(nostack)
    );
    ret
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn syscall3(n: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
    let ret: u64;
    core::arch::asm!(
        "int 0x80",
        in("rax") n,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        lateout("rax") ret,
        options(nostack)
    );
    ret
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn syscall4(n: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> u64 {
    let ret: u64;
    core::arch::asm!(
        "int 0x80",
        in("rax") n,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        in("r10") arg4,
        lateout("rax") ret,
        options(nostack)
    );
    ret
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub unsafe fn syscall0(n: u64) -> u64 {
    let ret: u64;
    core::arch::asm!(
        "svc #0",
        in("x8") n,
        lateout("x0") ret,
        options(nostack)
    );
    ret
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub unsafe fn syscall1(n: u64, arg1: u64) -> u64 {
    let ret: u64;
    core::arch::asm!(
        "svc #0",
        in("x8") n,
        in("x0") arg1,
        lateout("x0") ret,
        options(nostack)
    );
    ret
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub unsafe fn syscall2(n: u64, arg1: u64, arg2: u64) -> u64 {
    let ret: u64;
    core::arch::asm!(
        "svc #0",
        in("x8") n,
        in("x0") arg1,
        in("x1") arg2,
        lateout("x0") ret,
        options(nostack)
    );
    ret
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub unsafe fn syscall3(n: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
    let ret: u64;
    core::arch::asm!(
        "svc #0",
        in("x8") n,
        in("x0") arg1,
        in("x1") arg2,
        in("x2") arg3,
        lateout("x0") ret,
        options(nostack)
    );
    ret
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub unsafe fn syscall4(n: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> u64 {
    let ret: u64;
    core::arch::asm!(
        "svc #0",
        in("x8") n,
        in("x0") arg1,
        in("x1") arg2,
        in("x2") arg3,
        in("x3") arg4,
        lateout("x0") ret,
        options(nostack)
    );
    ret
}

#[inline(always)]
pub fn exit(code: u64) -> ! {
    unsafe {
        syscall1(SYS_EXIT as u64, code);
    }
    loop {}
}

#[inline(always)]
pub fn getpid() -> Result<u64> {
    let ret = unsafe { syscall0(SYS_GETPID as u64) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(crate::error::Error::from_code((-ret_i64) as i32))
    } else {
        Ok(ret)
    }
}

#[inline(always)]
pub fn fork() -> Result<u64> {
    let ret = unsafe { syscall0(SYS_FORK as u64) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(crate::error::Error::from_code((-ret_i64) as i32))
    } else {
        Ok(ret)
    }
}

#[inline(always)]
pub fn exec(path: &[u8]) -> Result<()> {
    let ret = unsafe { syscall1(SYS_EXEC as u64, path.as_ptr() as u64) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(crate::error::Error::from_code((-ret_i64) as i32))
    } else {
        Ok(())
    }
}

#[inline(always)]
pub fn wait(status: &mut i32) -> Result<u64> {
    let ret = unsafe { syscall1(SYS_WAIT as u64, status as *mut i32 as u64) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(crate::error::Error::from_code((-ret_i64) as i32))
    } else {
        Ok(ret)
    }
}

#[inline(always)]
pub fn yield_cpu() {
    unsafe {
        syscall0(SYS_YIELD as u64);
    }
}
