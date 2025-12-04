// userland/libgbsd/src/syscall.rs
// BSD-style syscall interface
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

// GuardBSD syscall numbers
pub const SYS_EXIT: u64 = 1;
pub const SYS_OPEN: u64 = 5;
pub const SYS_CLOSE: u64 = 6;
pub const SYS_READ: u64 = 3;
pub const SYS_WRITE: u64 = 4;
pub const SYS_PORT_CREATE: u64 = 20;
pub const SYS_PORT_DESTROY: u64 = 21;
pub const SYS_PORT_SEND: u64 = 22;
pub const SYS_PORT_RECEIVE: u64 = 23;
pub const SYS_PORT_CALL: u64 = 24;
pub const SYS_CAP_GRANT: u64 = 25;
pub const SYS_CAP_REVOKE: u64 = 26;
pub const SYS_CAP_DELEGATE: u64 = 27;
pub const SYS_CAP_COPY: u64 = 28;
pub const SYS_PORT_SYNC_CALL: u64 = 29;
pub const SYS_PORT_REPLY: u64 = 30;
pub const SYS_GET_ERROR_STATS: u64 = 31;
pub const SYS_LOG_READ: u64 = 40;
pub const SYS_LOG_ACK: u64 = 41;
pub const SYS_LOG_REGISTER_DAEMON: u64 = 42;
pub const SYS_GETPID: u64 = 7;
pub const SYS_EXEC: u64 = 20;
pub const SYS_STAT: u64 = 8;
pub const SYS_RENAME: u64 = 9;
pub const SYS_UNLINK: u64 = 10;
pub const SYS_SYNC: u64 = 11;

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
pub fn getpid() -> u64 {
    unsafe {
        syscall0(SYS_GETPID)
    }
}

#[inline(always)]
pub fn exec(path: &[u8]) -> ! {
    unsafe {
        syscall1(SYS_EXEC, path.as_ptr() as u64);
    }
    loop {} // Should never reach here
}
