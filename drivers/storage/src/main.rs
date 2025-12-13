//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: storage
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Sterownik storage GuardBSD.

#![no_std]
#![no_main]

use gbsd::*;
use core::sync::atomic::{AtomicBool, Ordering};

mod block;
mod disk;
mod partition;

use block::BlockDevice;
use disk::Disk;

const STORAGE_MAJOR: u16 = 8;
const STORAGE_MINOR: u16 = 0;
const DISK_SECTORS: u64 = 2048; // 1 MB disk (2048 * 512 bytes)

const MAX_REQUEST_SIZE: usize = 4096;
const MAX_RESPONSE_SIZE: usize = 4096;

// Operation codes
const OP_READ: u32 = 1;
const OP_WRITE: u32 = 2;
const OP_GET_INFO: u32 = 3;
const OP_FLUSH: u32 = 4;
const OP_GET_PARTITION_INFO: u32 = 5;

// Use Option for safer initialization
static mut BLOCK_DEV: Option<BlockDevice> = None;
static INITIALIZED: AtomicBool = AtomicBool::new(false);

#[no_mangle]
pub extern "C" fn _start() -> ! {
    storage_main()
}

fn storage_main() -> ! {
    // Initialize block device
    unsafe {
        BLOCK_DEV = Some(BlockDevice::new(Disk::new(DISK_SECTORS)));
        
        if let Some(ref mut dev) = BLOCK_DEV {
            if let Err(e) = dev.init() {
                // No valid partition table found
                // Continue anyway - disk can still be used for raw I/O
            }
        }
    }
    INITIALIZED.store(true, Ordering::Release);

    // Register as block device
    let _dev_id = match dev_register(DEV_BLOCK, STORAGE_MAJOR, STORAGE_MINOR) {
        Ok(id) => id,
        Err(_) => exit(1),
    };

    let port = match port_create() {
        Ok(p) => p,
        Err(_) => exit(2),
    };

    let mut req_buf = [0u8; MAX_REQUEST_SIZE];
    let mut resp_buf = [0u8; MAX_RESPONSE_SIZE];

    loop {
        // Blocking receive - no busy-wait
        match port_receive(port, req_buf.as_mut_ptr(), req_buf.len()) {
            Ok(received_len) => {
                if received_len < 4 {
                    // Invalid request - too small
                    let error = (-22i64).to_le_bytes(); // EINVAL
                    resp_buf[0..8].copy_from_slice(&error);
                    let _ = port_send(port, resp_buf.as_ptr(), 8);
                    continue;
                }

                let op = u32::from_le_bytes([
                    req_buf[0],
                    req_buf[1],
                    req_buf[2],
                    req_buf[3],
                ]);

                let (result, data_len) = handle_request(
                    op,
                    &req_buf[4..received_len],
                    &mut resp_buf[8..],
                );

                // Write result code (8 bytes) + data
                resp_buf[0..8].copy_from_slice(&result.to_le_bytes());
                let total_len = 8 + data_len;
                let _ = port_send(port, resp_buf.as_ptr(), total_len);
            }
            Err(_) => {
                // Port error - continue
                continue;
            }
        }
    }
}

/// Handle a storage request.
/// Returns (result_code, response_data_length)
fn handle_request(op: u32, data: &[u8], resp_data: &mut [u8]) -> (i64, usize) {
    if !INITIALIZED.load(Ordering::Acquire) {
        return (-5, 0); // EIO - not initialized
    }

    unsafe {
        if let Some(ref mut dev) = BLOCK_DEV {
            match op {
                OP_READ => {
                    // Read: [4 bytes partition][8 bytes LBA]
                    if data.len() < 12 {
                        return (-22, 0); // EINVAL
                    }

                    let partition = u32::from_le_bytes([
                        data[0],
                        data[1],
                        data[2],
                        data[3],
                    ]) as usize;

                    let lba = u64::from_le_bytes([
                        data[4], data[5], data[6], data[7],
                        data[8], data[9], data[10], data[11],
                    ]);

                    // Validate partition index
                    if partition >= 4 {
                        return (-22, 0); // EINVAL
                    }

                    // Ensure response buffer can hold a sector
                    if resp_data.len() < 512 {
                        return (-22, 0); // EINVAL
                    }

                    match dev.read(partition, lba, resp_data) {
                        Ok(n) => (0, n), // Success, return bytes read in data
                        Err(e) => (e, 0),
                    }
                }

                OP_WRITE => {
                    // Write: [4 bytes partition][8 bytes LBA][512 bytes data]
                    if data.len() < 12 + 512 {
                        return (-22, 0); // EINVAL
                    }

                    let partition = u32::from_le_bytes([
                        data[0],
                        data[1],
                        data[2],
                        data[3],
                    ]) as usize;

                    let lba = u64::from_le_bytes([
                        data[4], data[5], data[6], data[7],
                        data[8], data[9], data[10], data[11],
                    ]);

                    // Validate partition index
                    if partition >= 4 {
                        return (-22, 0); // EINVAL
                    }

                    let write_data = &data[12..12 + 512];

                    match dev.write(partition, lba, write_data) {
                        Ok(n) => (n as i64, 0),
                        Err(e) => (e, 0),
                    }
                }

                OP_GET_INFO => {
                    // Get device info
                    // Response: [4 bytes partition_count][8 bytes capacity][8 bytes sector_count]
                    if resp_data.len() < 20 {
                        return (-22, 0); // EINVAL
                    }

                    let partition_count = dev.partition_count() as u32;
                    let capacity = dev.capacity();
                    let sector_count = dev.sector_count();

                    resp_data[0..4].copy_from_slice(&partition_count.to_le_bytes());
                    resp_data[4..12].copy_from_slice(&capacity.to_le_bytes());
                    resp_data[12..20].copy_from_slice(&sector_count.to_le_bytes());

                    (0, 20)
                }

                OP_FLUSH => {
                    // Flush disk cache
                    // This would call disk.flush_cache() when that's properly implemented
                    // For now, just return success
                    (0, 0)
                }

                OP_GET_PARTITION_INFO => {
                    // Get info about specific partition
                    // Request: [4 bytes partition_index]
                    if data.len() < 4 {
                        return (-22, 0); // EINVAL
                    }

                    let partition = u32::from_le_bytes([
                        data[0],
                        data[1],
                        data[2],
                        data[3],
                    ]) as usize;

                    if let Some(part) = dev.get_partition(partition) {
                        // Response: [8 bytes start_lba][8 bytes sectors][1 byte type][1 byte active]
                        if resp_data.len() < 18 {
                            return (-22, 0); // EINVAL
                        }

                        resp_data[0..8].copy_from_slice(&part.start_lba.to_le_bytes());
                        resp_data[8..16].copy_from_slice(&part.sectors.to_le_bytes());
                        resp_data[16] = part.part_type;
                        resp_data[17] = if part.active { 1 } else { 0 };

                        (0, 18)
                    } else {
                        (-19, 0) // ENODEV - partition not found
                    }
                }

                _ => (-38, 0), // ENOSYS - not implemented
            }
        } else {
            (-5, 0) // EIO
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack));
        }

        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("wfi", options(nomem, nostack));
        }
    }
}