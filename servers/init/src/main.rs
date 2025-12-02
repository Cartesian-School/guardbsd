// servers/init/src/main.rs
// GuardBSD Init Process (PID 1)
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#![no_std]
#![no_main]

use core::panic::PanicInfo;

// GuardBSD syscall interface
mod syscalls {
    // GuardBSD syscall numbers (custom, not Linux)
    pub const GBSD_SYS_PORT_CREATE: u64 = 20;
    pub const GBSD_SYS_EXIT: u64 = 1;

    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    pub fn port_create() -> u64 {
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "int 0x80",
                in("rax") GBSD_SYS_PORT_CREATE,
                in("rdi") 0u64,
                lateout("rax") ret,
                options(nostack)
            );
        }
        ret
    }

    #[cfg(target_arch = "aarch64")]
    #[inline(always)]
    pub fn port_create() -> u64 {
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "svc #0",
                in("x8") GBSD_SYS_PORT_CREATE,
                in("x0") 0u64,
                lateout("x0") ret,
                options(nostack)
            );
        }
        ret
    }

    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    pub fn exit(code: u64) -> ! {
        unsafe {
            core::arch::asm!(
                "int 0x80",
                in("rax") GBSD_SYS_EXIT,
                in("rdi") code,
                options(noreturn)
            );
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[inline(always)]
    pub fn exit(code: u64) -> ! {
        unsafe {
            core::arch::asm!(
                "svc #0",
                in("x8") GBSD_SYS_EXIT,
                in("x0") code,
                options(noreturn)
            );
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    init_main();
    syscalls::exit(0);
}

fn init_main() {
    let port = syscalls::port_create();
    
    if port == 0 {
        syscalls::exit(1);
    }
    
    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("pause", options(nomem, nostack));
        }
        
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("yield", options(nomem, nostack));
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    syscalls::exit(1);
}
