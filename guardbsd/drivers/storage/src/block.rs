// drivers/storage/src/block.rs
// Block device operations
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::disk::Disk;
use crate::partition::PartitionTable;

pub struct BlockDevice {
    disk: Disk,
    partitions: PartitionTable,
}

impl BlockDevice {
    pub const fn new(disk: Disk) -> Self {
        Self {
            disk,
            partitions: PartitionTable::new(),
        }
    }

    pub fn init(&mut self) -> Result<(), i64> {
        // Read MBR
        let mut mbr = [0u8; 512];
        self.disk.read_sector(0, &mut mbr)?;
        
        // Parse partition table
        self.partitions.parse_mbr(&mbr)?;
        
        Ok(())
    }

    pub fn read(&mut self, partition: usize, lba: u64, buf: &mut [u8]) -> Result<usize, i64> {
        if let Some(part) = self.partitions.get(partition) {
            let absolute_lba = part.start_lba + lba;
            if lba >= part.sectors {
                return Err(-22); // EINVAL
            }
            self.disk.read_sector(absolute_lba, buf)
        } else {
            Err(-19) // ENODEV
        }
    }

    pub fn write(&mut self, partition: usize, lba: u64, buf: &[u8]) -> Result<usize, i64> {
        if let Some(part) = self.partitions.get(partition) {
            let absolute_lba = part.start_lba + lba;
            if lba >= part.sectors {
                return Err(-22); // EINVAL
            }
            self.disk.write_sector(absolute_lba, buf)
        } else {
            Err(-19) // ENODEV
        }
    }

    pub fn partition_count(&self) -> usize {
        self.partitions.count()
    }

    pub fn capacity(&self) -> u64 {
        self.disk.capacity()
    }
}
