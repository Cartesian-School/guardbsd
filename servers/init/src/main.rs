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
    pub const GBSD_SYS_FORK: u64 = SYS_FORK as u64;

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
    pub fn fork() -> u64 {
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "int 0x80",
                in("rax") GBSD_SYS_FORK,
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
    pub fn fork() -> u64 {
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "svc #0",
                in("x8") GBSD_SYS_FORK,
                lateout("x0") ret,
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
    
    // Phase 1: Start RAMFS (must be first - provides storage backend)
    let ramfs_pid = start_ramfs();
    if ramfs_pid == 0 {
        // RAMFS failed to start - system cannot function
        // For now, continue anyway (servers may already be loaded)
        return;
    }
    
    // Wait for RAMFS to initialize (simple delay)
    wait_for_server_ready();
    
    // Phase 2: Start VFS (depends on RAMFS)
    let vfs_pid = start_vfs();
    if vfs_pid == 0 {
        // VFS failed to start - system can still work with direct kernel paths
        return;
    }
    
    // Wait for VFS to mount RAMFS
    wait_for_server_ready();
    
    // Phase 3: Start device server (for device registration and management)
    let devd_pid = start_devd();
    if devd_pid == 0 {
        // devd failed to start - system can work without it but devices won't be managed
        return;
    }
    
    // Wait for devd to initialize
    wait_for_server_ready();
}

fn start_logd() {
    // Start the logging daemon
    let path = b"/bin/logd\0";
    let ret = syscalls::exec(path);

    if ret != 0 {
        // Logd failed to start - log error but continue
        // System can still function without logd (logs go to serial)
        // In production, this would log to serial console
        return;
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
        // Shell failed to start - log error but don't exit
        // System continues running without interactive shell
        return;
    }
}

fn start_ramfs() -> u64 {
    // Fork and exec RAMFS server
    let pid = syscalls::fork();
    if pid == 0 {
        // Child process - exec ramfs
        let path = b"/servers/ramfs\0";
        let ret = syscalls::exec(path);
        if ret != 0 {
            // Exec failed - exit child
            syscalls::exit(1);
        }
    }
    // Parent returns PID of child (or 0 on fork failure)
    pid
}

fn start_vfs() -> u64 {
    // Fork and exec VFS server
    let pid = syscalls::fork();
    if pid == 0 {
        // Child process - exec vfs
        let path = b"/servers/vfs\0";
        let ret = syscalls::exec(path);
        if ret != 0 {
            // Exec failed - exit child
            syscalls::exit(1);
        }
    }
    // Parent returns PID of child (or 0 on fork failure)
    pid
}

fn start_devd() -> u64 {
    // Fork and exec device daemon
    let pid = syscalls::fork();
    if pid == 0 {
        // Child process - exec devd
        let path = b"/servers/devd\0";
        let ret = syscalls::exec(path);
        if ret != 0 {
            // Exec failed - exit child
            syscalls::exit(1);
        }
    }
    // Parent returns PID of child (or 0 on fork failure)
    pid
}

fn wait_for_server_ready() {
    // Simple delay to let servers initialize
    // In production, this would use IPC handshake
    for _ in 0..10000 {
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
