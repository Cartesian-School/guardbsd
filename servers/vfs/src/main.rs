// servers/vfs/src/main.rs
// GuardBSD Virtual File System Server
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#![no_std]
#![no_main]

use gbsd::*;

mod vnode;
mod ops;

const MAX_OPEN_FILES: usize = 256;
const MAX_MOUNTS: usize = 16;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    vfs_main();
}

fn vfs_main() -> ! {
    // Create VFS service port
    let port = match port_create() {
        Ok(p) => {
            klog_info!("vfs", "VFS server started (port={}, pid={})", p, getpid());
            p
        }
        Err(_) => {
            klog_error!("vfs", "failed to create VFS service port");
            exit(1);
        }
    };

    // VFS server loop
    loop {
        // Wait for requests (future: port_receive)
        
        // Process VFS operations
        
        // Send responses
        
        // Yield CPU
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
