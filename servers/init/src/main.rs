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
    // Import canonical syscall numbers from shared module
    include!("../../../shared/syscall_numbers.rs");
    
    // Compatibility aliases for this module
    pub const GBSD_SYS_PORT_CREATE: u64 = SYS_IPC_PORT_CREATE as u64;
    pub const GBSD_SYS_EXIT: u64 = SYS_EXIT as u64;
    pub const GBSD_SYS_EXEC: u64 = SYS_EXEC as u64;

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

    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    pub fn exec(path: &[u8]) -> u64 {
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "int 0x80",
                in("rax") GBSD_SYS_EXEC,
                in("rdi") path.as_ptr() as u64,
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

    #[cfg(target_arch = "aarch64")]
    #[inline(always)]
    pub fn exec(path: &[u8]) -> u64 {
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "svc #0",
                in("x8") GBSD_SYS_EXEC,
                in("x0") path.as_ptr() as u64,
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
    // Create IPC port for init process
    let port = syscalls::port_create();
    if port == 0 {
        syscalls::exit(1);
    }

    // Startup sequence: microkernels → servers → logd → shell
    startup_sequence();

    // After startup, init becomes the parent of all processes
    // In a real system, this would wait for children and handle signals
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

fn startup_sequence() {
    // Phase 1: Microkernels are already loaded by kernel

    // Phase 2: Start system servers
    start_servers();

    // Phase 3: Start logd (logging daemon)
    start_logd();

    // Phase 4: Start shell
    start_shell();
}

fn start_servers() {
    // Start core system servers
    // Note: In microkernel architecture, these run as separate processes
    // For now, just log that they would be started

    // In a real implementation, these would be:
    // exec(b"/servers/devd");
    // exec(b"/servers/vfs");
    // exec(b"/servers/ramfs");
    // exec(b"/servers/netd");
    // exec(b"/servers/netsvc");
}

fn start_logd() {
    // Start the logging daemon
    let path = b"/bin/logd\0";
    let ret = syscalls::exec(path);

    if ret != 0 {
        // Logd failed to start - this is critical
        // In a real system, we might try to start a fallback logger
        syscalls::exit(1);
    }

    // Wait for logd to register itself (simple polling)
    // In practice, this would use IPC to wait for registration
    for _ in 0..1000 {
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

fn start_shell() {
    // Start the GuardBSD shell
    let path = b"/bin/gsh\0";
    let ret = syscalls::exec(path);

    if ret != 0 {
        // Shell failed to start
        syscalls::exit(1);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    syscalls::exit(1);
}
