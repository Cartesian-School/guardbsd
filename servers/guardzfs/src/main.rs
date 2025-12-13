//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: guardzfs
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Serwer GuardZFS.

#![no_std]
#![no_main]

use libgbsd::syscall::*;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

macro_rules! println {
    ($($arg:tt)*) => {{
        // Simple print for now
    }};
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("GuardZFS: Starting ZFS-inspired filesystem server...");

    // Create IPC port
    let port = port_create();
    if port == 0 {
        println!("GuardZFS: Failed to create IPC port");
        exit(1);
    }

    println!("GuardZFS: Listening on port {}", port);

    // Initialize storage pool
    // TODO: Detect disks and create pool

    let mut guardzfs = guardzfs::GuardZfs::new();

    loop {
        let mut req_buf = [0u8; 4096];
        let mut resp_buf = [0u8; 4096];

        // Wait for VFS requests
        let _len = port_receive(port, req_buf.as_mut_ptr(), req_buf.len());

        // Parse operation
        let op = u32::from_le_bytes([req_buf[0], req_buf[1], req_buf[2], req_buf[3]]);
        let reply_port = u64::from_le_bytes([
            req_buf[4],
            req_buf[5],
            req_buf[6],
            req_buf[7],
            req_buf[8],
            req_buf[9],
            req_buf[10],
            req_buf[11],
        ]);

        // Handle request
        let result = handle_request(op, &req_buf[12..], &mut guardzfs);

        // Send response
        resp_buf[0..8].copy_from_slice(&result.to_le_bytes());
        let _ = port_send(reply_port, resp_buf.as_ptr(), resp_buf.len());
    }
}

fn handle_request(op: u32, _data: &[u8], _zfs: &mut guardzfs::GuardZfs) -> i64 {
    match op {
        1 => {
            // VFS_OP_OPEN
            // TODO: Parse path, call zfs.open()
            0
        }
        2 => {
            // VFS_OP_CLOSE
            0
        }
        3 => {
            // VFS_OP_READ
            // TODO: Call zfs.read()
            0
        }
        4 => {
            // VFS_OP_WRITE
            // TODO: Call zfs.write()
            0
        }
        5 => {
            // VFS_OP_MKDIR
            // TODO: Call zfs.mkdir()
            0
        }
        _ => -38, // ENOSYS
    }
}
