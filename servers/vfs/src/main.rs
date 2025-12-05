// servers/vfs/src/main.rs
// GuardBSD Virtual File System Server
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#![no_std]
#![no_main]

use crate::ops::*;
use gbsd::*;

mod ops;
mod vnode;

const MAX_OPEN_FILES: usize = 256;
const MAX_MOUNTS: usize = 16;

#[derive(Clone, Copy, PartialEq)]
enum MountType {
    RamFs,
    DevFs,
}

#[derive(Copy, Clone)]
struct MountPoint {
    path: [u8; 256],
    path_len: usize,
    mount_type: MountType,
    port: usize, // IPC port for the filesystem server
}

struct MountTable {
    mounts: [Option<MountPoint>; MAX_MOUNTS],
}

impl MountTable {
    fn new() -> Self {
        MountTable {
            mounts: [None; MAX_MOUNTS],
        }
    }

    fn mount(&mut self, path: &str, mount_type: MountType, port: usize) -> bool {
        for i in 0..MAX_MOUNTS {
            if self.mounts[i].is_none() {
                let mut path_bytes = [0u8; 256];
                let path_data = path.as_bytes();
                let copy_len = path_data.len().min(255);
                path_bytes[..copy_len].copy_from_slice(&path_data[..copy_len]);

                self.mounts[i] = Some(MountPoint {
                    path: path_bytes,
                    path_len: copy_len,
                    mount_type,
                    port,
                });
                return true;
            }
        }
        false
    }

    fn find_mount(&self, path: &str) -> Option<&MountPoint> {
        for mount in &self.mounts {
            if let Some(mp) = mount {
                // Simple prefix matching for now
                let mp_path = &mp.path[..mp.path_len];
                let mp_path_str = core::str::from_utf8(mp_path).unwrap_or("");
                if path.starts_with(mp_path_str) {
                    return Some(mp);
                }
            }
        }
        None
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    vfs_main();
}

fn vfs_main() -> ! {
    // Create VFS service port
    let port = match port_create() {
        Ok(p) => {
            let pid = getpid().unwrap_or(0);
            klog_info!("vfs", "VFS server started (port={}, pid={})", p, pid);
            p
        }
        Err(_) => {
            klog_error!("vfs", "failed to create VFS service port");
            exit(1);
        }
    };

    // Initialize VFS mounts
    let mut mounts = MountTable::new();

    // For now, use a known port for RAMFS
    // In production, this would use service discovery
    let ramfs_port = 1001; // Known RAMFS port

    // Mount RAMFS at root
    if mounts.mount("/", MountType::RamFs, ramfs_port) {
        klog_info!("vfs", "mounted RAMFS at / (port={})", ramfs_port);
    } else {
        klog_error!("vfs", "failed to mount RAMFS at /");
    }

    // IPC request handling
    let mut req_buf = [0u8; 512];
    let mut resp_buf = [0u8; 512];

    // VFS server loop
    loop {
        // Wait for VFS requests via IPC
        if port_receive(port, req_buf.as_mut_ptr(), req_buf.len()).is_ok() {
            let req = VfsRequest::from_bytes(&req_buf);
            let resp = process_vfs_request(&req, &mut mounts, port as usize);

            // Send response
            resp_buf.copy_from_slice(&resp.to_bytes());
            let _ = port_send(port, resp_buf.as_ptr(), resp_buf.len());
        }

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
