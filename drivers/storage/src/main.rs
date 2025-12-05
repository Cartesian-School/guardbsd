// drivers/storage/src/main.rs
// GuardBSD Storage Driver
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#![no_std]
#![no_main]

use gbsd::*;

mod block;
mod disk;
mod partition;

use block::BlockDevice;
use disk::Disk;

const STORAGE_MAJOR: u16 = 8;
const STORAGE_MINOR: u16 = 0;
const DISK_SECTORS: u64 = 2048; // 1 MB disk (2048 * 512 bytes)

static mut BLOCK_DEV: BlockDevice = BlockDevice::new(Disk::new(DISK_SECTORS));

#[no_mangle]
pub extern "C" fn _start() -> ! {
    storage_main()
}

fn storage_main() -> ! {
    unsafe {
        if let Err(_) = BLOCK_DEV.init() {
            // No valid partition table, continue anyway
        }
    }

    // Register as block device
    let _dev_id = match dev_register(DEV_BLOCK, STORAGE_MAJOR, STORAGE_MINOR) {
        Ok(id) => id,
        Err(_) => exit(1),
    };

    let port = match port_create() {
        Ok(p) => p,
        Err(_) => exit(2),
    };

    let mut req_buf = [0u8; 1024];
    let mut resp_buf = [0u8; 1024];

    loop {
        if port_receive(port, req_buf.as_mut_ptr(), req_buf.len()).is_ok() {
            if req_buf.len() >= 4 {
                let op = u32::from_le_bytes([req_buf[0], req_buf[1], req_buf[2], req_buf[3]]);
                let result = handle_request(op, &req_buf[4..], &mut resp_buf[8..]);

                resp_buf[0..8].copy_from_slice(&result.to_le_bytes());
                let _ = port_send(port, resp_buf.as_ptr(), resp_buf.len());
            }
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

fn handle_request(op: u32, data: &[u8], resp_data: &mut [u8]) -> i64 {
    unsafe {
        match op {
            1 => {
                // Read
                if data.len() >= 12 {
                    let partition =
                        u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
                    let lba = u64::from_le_bytes([
                        data[4], data[5], data[6], data[7], data[8], data[9], data[10], data[11],
                    ]);

                    match BLOCK_DEV.read(partition, lba, resp_data) {
                        Ok(n) => n as i64,
                        Err(e) => e,
                    }
                } else {
                    -22 // EINVAL
                }
            }
            2 => {
                // Write
                if data.len() >= 12 {
                    let partition =
                        u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
                    let lba = u64::from_le_bytes([
                        data[4], data[5], data[6], data[7], data[8], data[9], data[10], data[11],
                    ]);
                    let write_data = &data[12..];

                    match BLOCK_DEV.write(partition, lba, write_data) {
                        Ok(n) => n as i64,
                        Err(e) => e,
                    }
                } else {
                    -22 // EINVAL
                }
            }
            3 => {
                // Get info
                let count = BLOCK_DEV.partition_count() as i64;
                let capacity = BLOCK_DEV.capacity();

                // Return partition count in result, capacity in response data
                resp_data[0..8].copy_from_slice(&capacity.to_le_bytes());
                count
            }
            _ => -38, // ENOSYS
        }
    }
}
