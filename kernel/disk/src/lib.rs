// kernel/disk/src/lib.rs
// GuardBSD Disk Infrastructure
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#![no_std]

pub mod ata;
pub mod block_device;
pub mod cache;
pub mod partition;
pub mod swap;

pub use ata::*;
pub use block_device::*;
pub use cache::*;
pub use partition::*;
pub use swap::*;

/// Initialize disk subsystem
pub fn init() -> usize {
    // Probe ATA disks
    let ata_count = probe_ata_disks();

    // TODO: Probe AHCI/SATA disks
    // TODO: Probe NVMe disks

    // Detect partitions on all disks
    for i in 0..disk_count() {
        if let Some(device) = get_disk(i as u32) {
            let _ = detect_partitions(device);
        }
    }

    ata_count
}

/// Read block with caching
pub fn read_block_cached(
    device_id: u32,
    block_num: u64,
    buf: &mut [u8; BLOCK_SIZE],
) -> Result<(), DiskError> {
    if let Some(device) = get_disk(device_id) {
        cache_read(device, block_num, buf)
    } else {
        Err(DiskError::DeviceNotFound)
    }
}

/// Write block with caching
pub fn write_block_cached(
    device_id: u32,
    block_num: u64,
    buf: &[u8; BLOCK_SIZE],
) -> Result<(), DiskError> {
    if let Some(device) = get_disk(device_id) {
        cache_write(device, block_num, buf)
    } else {
        Err(DiskError::DeviceNotFound)
    }
}

/// Flush all dirty blocks to disk
pub fn sync() -> usize {
    cache_flush()
}
