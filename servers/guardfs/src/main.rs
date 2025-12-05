// servers/guardfs/src/main.rs
// GuardFS Server Main Loop
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergiej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#![no_std]
#![no_main]

extern crate alloc;

use guardfs::*;
use libgbsd::*;

static mut GUARDFS: Option<GuardFs> = None;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Initialize GuardFS
    unsafe {
        GUARDFS = Some(GuardFs::new());

        // Format filesystem (32MB)
        if let Some(ref mut fs) = GUARDFS {
            fs.format(8192, "guardfs_root");
        }
    }

    // Create IPC port for GuardFS
    let port = ipc::port_create();
    if port < 0 {
        loop {
            unsafe {
                core::arch::asm!("hlt");
            }
        }
    }

    println("[GuardFS] Server started");
    println("[GuardFS] Features: Journaling + Snapshots + Compression");
    println("[GuardFS] Capacity: 32MB, 1024 inodes, 8 snapshots");

    // Main server loop
    loop {
        let mut req_buf = [0u8; 4096];
        let result = ipc::port_receive(port as u64, req_buf.as_mut_ptr(), 4096);

        if result < 0 {
            continue;
        }

        // Parse request
        let op = u32::from_le_bytes([req_buf[0], req_buf[1], req_buf[2], req_buf[3]]);
        let reply_port =
            u32::from_le_bytes([req_buf[4], req_buf[5], req_buf[6], req_buf[7]]) as u64;

        // Process request
        let response = handle_request(op, &req_buf[8..]);

        // Send response
        let mut resp_buf = [0u8; 4096];
        resp_buf[0..8].copy_from_slice(&response.to_le_bytes());

        let _ = ipc::port_send(reply_port, resp_buf.as_ptr(), 4096);
    }
}

fn handle_request(op: u32, data: &[u8]) -> i64 {
    unsafe {
        if let Some(ref mut fs) = GUARDFS {
            match op {
                1 => handle_open(fs, data),
                2 => handle_close(fs, data),
                3 => handle_read(fs, data),
                4 => handle_write(fs, data),
                5 => handle_stat(fs, data),
                6 => handle_mkdir(fs, data),
                8 => handle_unlink(fs, data),
                10 => handle_snapshot_create(fs, data),
                11 => handle_snapshot_delete(fs, data),
                12 => handle_snapshot_list(fs, data),
                _ => -38, // ENOSYS
            }
        } else {
            -5 // EIO
        }
    }
}

fn handle_open(fs: &mut GuardFs, data: &[u8]) -> i64 {
    // Parse path and flags
    let path_end = data.iter().position(|&c| c == 0).unwrap_or(256);
    if path_end == 0 {
        return -22; // EINVAL
    }

    let path = core::str::from_utf8(&data[..path_end]).unwrap_or("");
    let flags = u32::from_le_bytes([data[256], data[257], data[258], data[259]]);

    match fs.open(path, flags) {
        Ok(fd) => fd as i64,
        Err(e) => e as i64,
    }
}

fn handle_close(_fs: &mut GuardFs, _data: &[u8]) -> i64 {
    // Close is mostly a no-op in this implementation
    0
}

fn handle_read(fs: &mut GuardFs, data: &[u8]) -> i64 {
    let inode_num = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let length = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    let offset = u64::from_le_bytes([
        data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15],
    ]);

    let mut buf = [0u8; 4096];
    match fs.read(inode_num, &mut buf[..length as usize], offset) {
        Ok(bytes_read) => bytes_read as i64,
        Err(e) => e as i64,
    }
}

fn handle_write(fs: &mut GuardFs, data: &[u8]) -> i64 {
    let inode_num = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let length = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    let offset = u64::from_le_bytes([
        data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15],
    ]);

    match fs.write(inode_num, &data[16..16 + length as usize], offset) {
        Ok(bytes_written) => bytes_written as i64,
        Err(e) => e as i64,
    }
}

fn handle_stat(_fs: &mut GuardFs, _data: &[u8]) -> i64 {
    // TODO: Implement stat
    -38 // ENOSYS
}

fn handle_mkdir(fs: &mut GuardFs, data: &[u8]) -> i64 {
    let path_end = data.iter().position(|&c| c == 0).unwrap_or(256);
    if path_end == 0 {
        return -22;
    }

    let path = core::str::from_utf8(&data[..path_end]).unwrap_or("");
    let mode = u16::from_le_bytes([data[260], data[261]]);

    match fs.mkdir(path, mode) {
        Ok(_) => 0,
        Err(e) => e as i64,
    }
}

fn handle_unlink(fs: &mut GuardFs, data: &[u8]) -> i64 {
    let path_end = data.iter().position(|&c| c == 0).unwrap_or(256);
    if path_end == 0 {
        return -22;
    }

    let path = core::str::from_utf8(&data[..path_end]).unwrap_or("");

    match fs.unlink(path) {
        Ok(_) => 0,
        Err(e) => e as i64,
    }
}

fn handle_snapshot_create(fs: &mut GuardFs, data: &[u8]) -> i64 {
    let name_end = data.iter().position(|&c| c == 0).unwrap_or(64);
    if name_end == 0 {
        return -22;
    }

    let name = core::str::from_utf8(&data[..name_end]).unwrap_or("");

    match fs.create_snapshot(name) {
        Ok(id) => id as i64,
        Err(e) => e as i64,
    }
}

fn handle_snapshot_delete(fs: &mut GuardFs, data: &[u8]) -> i64 {
    let snap_id = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);

    match fs.delete_snapshot(snap_id) {
        Ok(_) => 0,
        Err(e) => e as i64,
    }
}

fn handle_snapshot_list(_fs: &mut GuardFs, _data: &[u8]) -> i64 {
    // TODO: Return snapshot list
    0
}

// Helper functions
fn println(msg: &str) {
    // Simple serial output
    for byte in msg.as_bytes() {
        unsafe {
            while (libgbsd::inb(0x3F8 + 5) & 0x20) == 0 {}
            libgbsd::outb(0x3F8, *byte);
        }
    }
    unsafe {
        while (libgbsd::inb(0x3F8 + 5) & 0x20) == 0 {}
        libgbsd::outb(0x3F8, b'\n');
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
