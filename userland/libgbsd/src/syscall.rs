// userland/libgbsd/src/syscall.rs
// BSD-style syscall interface
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

// GuardBSD syscall numbers (canonical, shared with kernel/boot_stub)
pub const SYS_EXIT: u64 = 0;
pub const SYS_WRITE: u64 = 1;
pub const SYS_READ: u64 = 2;   // ENOSYS placeholder
pub const SYS_FORK: u64 = 3;   // reserved
pub const SYS_EXEC: u64 = 4;   // to be implemented
pub const SYS_WAIT: u64 = 5;   // reserved
pub const SYS_YIELD: u64 = 6;  // ENOSYS placeholder
pub const SYS_GETPID: u64 = 7; // to be implemented

pub const SYS_OPEN: u64 = 8;   // reserved
pub const SYS_CLOSE: u64 = 9;  // reserved
pub const SYS_MKDIR: u64 = 10; // reserved
pub const SYS_STAT: u64 = 11;  // reserved
pub const SYS_RENAME: u64 = 12;// reserved
pub const SYS_UNLINK: u64 = 13;// reserved
pub const SYS_SYNC: u64 = 14;  // reserved

pub const SYS_LOG_READ: u64 = 20;   // reserved
pub const SYS_LOG_ACK: u64 = 21;    // reserved
pub const SYS_LOG_REGISTER_DAEMON: u64 = 22; // reserved

pub const SYS_IPC_PORT_CREATE: u64 = 30; // reserved
pub const SYS_IPC_SEND: u64 = 31;        // reserved
pub const SYS_IPC_RECV: u64 = 32;        // reserved

use crate::error::{Error, Result};

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
        syscall1(SYS_EXIT, code);
    }
    loop {}
}

#[inline(always)]
pub fn getpid() -> Result<u64> {
    let ret = unsafe { syscall0(SYS_GETPID) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(crate::error::Error::from_code((-ret_i64) as u64))
    } else {
        Ok(ret)
    }
}

#[inline(always)]
pub fn fork() -> Result<u64> {
    let ret = unsafe { syscall0(SYS_FORK) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(crate::error::Error::from_code((-ret_i64) as u64))
    } else {
        Ok(ret)
    }
}

#[inline(always)]
pub fn exec(path: &[u8]) -> Result<()> {
    let ret = unsafe { syscall1(SYS_EXEC, path.as_ptr() as u64) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(crate::error::Error::from_code((-ret_i64) as u64))
    } else {
        Ok(())
    }
}

#[inline(always)]
pub fn wait(status: &mut i32) -> Result<u64> {
    let ret = unsafe { syscall1(SYS_WAIT, status as *mut i32 as u64) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(crate::error::Error::from_code((-ret_i64) as u64))
    } else {
        Ok(ret)
    }
}

#[inline(always)]
pub fn yield_cpu() {
    unsafe {
        syscall0(SYS_YIELD);
    }
}
