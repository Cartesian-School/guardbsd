// servers/ramfs/src/main.rs
// GuardBSD RAM Filesystem Server
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#![no_std]
#![no_main]

use gbsd::*;

mod node;
mod ops;

use node::RamFs;

static mut RAMFS: RamFs = RamFs::new();

#[no_mangle]
pub extern "C" fn _start() -> ! {
    ramfs_main()
}

fn ramfs_main() -> ! {
    unsafe {
        RAMFS.init();
    }

    let port = match port_create() {
        Ok(p) => p,
        Err(_) => exit(1),
    };

    let mut req_buf = [0u8; 512];
    let mut resp_buf = [0u8; 512];

    loop {
        // Wait for VFS requests via IPC
        if port_receive(port, req_buf.as_mut_ptr(), req_buf.len()).is_ok() {
            let op = u32::from_le_bytes([req_buf[0], req_buf[1], req_buf[2], req_buf[3]]);
            let result = handle_request(op, &req_buf[8..]);
            
            resp_buf[0..8].copy_from_slice(&result.to_le_bytes());
            let _ = port_send(port, resp_buf.as_ptr(), resp_buf.len());
        }

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

fn handle_request(op: u32, data: &[u8]) -> i64 {
    unsafe {
        match op {
            1 => { // Open
                if data.len() >= 264 {
                    let flags = u32::from_le_bytes([data[256], data[257], data[258], data[259]]);
                    ops::open(&mut RAMFS, &data[..256], flags)
                } else {
                    -22 // EINVAL
                }
            }
            3 => { // Read
                if data.len() >= 4 {
                    let fd = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                    let mut buf = [0u8; 4096];
                    ops::read(&mut RAMFS, fd, &mut buf)
                } else {
                    -22 // EINVAL
                }
            }
            4 => { // Write
                if data.len() >= 4 {
                    let fd = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                    ops::write(&mut RAMFS, fd, &data[4..])
                } else {
                    -22 // EINVAL
                }
            }
            6 => { // Mkdir
                if data.len() >= 256 {
                    ops::mkdir(&mut RAMFS, &data[..256])
                } else {
                    -22 // EINVAL
                }
            }
            8 => { // Unlink
                if data.len() >= 256 {
                    ops::unlink(&mut RAMFS, &data[..256])
                } else {
                    -22 // EINVAL
                }
            }
            _ => -38, // ENOSYS
        }
    }
}
