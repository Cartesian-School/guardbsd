//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: ramfs
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Serwer RAMFS GuardBSD.

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
        let pid = getpid().unwrap_or(0);
        klog_info!(
            "ramfs",
            "RAMFS mounted (nodes={}, max_file={}, pid={})",
            RAMFS.node_count(),
            4096,
            pid
        );
    }

    // Use known port for RAMFS
    let port = 1001;
    klog_info!("ramfs", "RAMFS server started (port={})", port);

    let mut req_buf = [0u8; 512];
    let mut resp_buf = [0u8; 512];

    loop {
        // Wait for VFS requests via IPC
        if port_receive(port, req_buf.as_mut_ptr(), req_buf.len()).is_ok() {
            let op = u32::from_le_bytes([req_buf[0], req_buf[1], req_buf[2], req_buf[3]]);
            let reply_port =
                u32::from_le_bytes([req_buf[4], req_buf[5], req_buf[6], req_buf[7]]);
            let result = handle_request(op, &req_buf[8..], reply_port);

            resp_buf[0..8].copy_from_slice(&result.to_le_bytes());
            let _ = port_send(reply_port as u64, resp_buf.as_ptr(), resp_buf.len());
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

fn handle_request(op: u32, data: &[u8], reply_port: u32) -> i64 {
    unsafe {
        match op {
            1 => {
                // Open
                if data.len() >= 264 {
                    let flags = u32::from_le_bytes([data[256], data[257], data[258], data[259]]);
                    klog_info!(
                        "ramfs",
                        "open request, reply_port={}, flags=0x{:x}",
                        reply_port,
                        flags
                    );
                    ops::open(&mut RAMFS, &data[..256], flags)
                } else {
                    -22 // EINVAL
                }
            }
            3 => {
                // Read
                if data.len() >= 4 {
                    let fd = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                    let mut buf = [0u8; 4096];
                    ops::read(&mut RAMFS, fd, &mut buf)
                } else {
                    -22 // EINVAL
                }
            }
            4 => {
                // Write
                if data.len() >= 4 {
                    let fd = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                    ops::write(&mut RAMFS, fd, &data[4..])
                } else {
                    -22 // EINVAL
                }
            }
            6 => {
                // Mkdir
                if data.len() >= 256 {
                    ops::mkdir(&mut RAMFS, &data[..256])
                } else {
                    -22 // EINVAL
                }
            }
            8 => {
                // Unlink
                if data.len() >= 256 {
                    ops::unlink(&mut RAMFS, &data[..256])
                } else {
                    -22 // EINVAL
                }
            }
            9 => {
                // Mknod (create device node)
                // Format: [path:256][dev_id:u32]
                if data.len() >= 260 {
                    let dev_id = u32::from_le_bytes([data[256], data[257], data[258], data[259]]);
                    ops::mknod(&mut RAMFS, &data[..256], dev_id)
                } else {
                    -22 // EINVAL
                }
            }
            _ => -38, // ENOSYS
        }
    }
}
