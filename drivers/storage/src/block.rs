//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: storage
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Operacje na urządzeniach blokowych.

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

    /// Initialize the block device by reading and parsing the MBR.
    pub fn init(&mut self) -> Result<(), i64> {
        // Read MBR (sector 0)
        let mut mbr = [0u8; 512];
        self.disk.read_sector(0, &mut mbr)?;

        // Parse partition table
        self.partitions.parse_mbr(&mbr)?;

        Ok(())
    }

    /// Read a single sector from a partition.
    /// `partition`: partition index (0-3)
    /// `lba`: logical block address relative to partition start
    /// `buf`: buffer to read into (must be at least 512 bytes)
    pub fn read(&mut self, partition: usize, lba: u64, buf: &mut [u8]) -> Result<usize, i64> {
        let part = self.partitions.get(partition).ok_or(-19)?; // ENODEV

        // Validate LBA is within partition bounds
        if lba >= part.sectors {
            return Err(-22); // EINVAL - beyond partition end
        }

        // Calculate absolute LBA with overflow check
        let absolute_lba = part.start_lba.checked_add(lba).ok_or(-22)?; // EINVAL - address overflow

        // Validate absolute LBA is within disk bounds
        if absolute_lba >= self.disk.sector_count() {
            return Err(-22); // EINVAL - beyond disk end
        }

        self.disk.read_sector(absolute_lba, buf)
    }

    /// Write a single sector to a partition.
    /// `partition`: partition index (0-3)
    /// `lba`: logical block address relative to partition start
    /// `buf`: buffer to write from (must be at least 512 bytes)
    pub fn write(&mut self, partition: usize, lba: u64, buf: &[u8]) -> Result<usize, i64> {
        let part = self.partitions.get(partition).ok_or(-19)?; // ENODEV

        // Validate LBA is within partition bounds
        if lba >= part.sectors {
            return Err(-22); // EINVAL - beyond partition end
        }

        // Calculate absolute LBA with overflow check
        let absolute_lba = part.start_lba.checked_add(lba).ok_or(-22)?; // EINVAL - address overflow

        // Validate absolute LBA is within disk bounds
        if absolute_lba >= self.disk.sector_count() {
            return Err(-22); // EINVAL - beyond disk end
        }

        self.disk.write_sector(absolute_lba, buf)
    }

    /// Read a sector directly from the disk (bypassing partitions).
    /// Useful for reading MBR or other raw disk data.
    pub fn read_raw(&mut self, lba: u64, buf: &mut [u8]) -> Result<usize, i64> {
        if lba >= self.disk.sector_count() {
            return Err(-22); // EINVAL
        }
        self.disk.read_sector(lba, buf)
    }

    /// Write a sector directly to the disk (bypassing partitions).
    /// USE WITH CAUTION - can corrupt partition table!
    pub fn write_raw(&mut self, lba: u64, buf: &[u8]) -> Result<usize, i64> {
        if lba >= self.disk.sector_count() {
            return Err(-22); // EINVAL
        }
        self.disk.write_sector(lba, buf)
    }

    /// Get the number of partitions found.
    pub fn partition_count(&self) -> usize {
        self.partitions.count()
    }

    /// Get total disk capacity in bytes.
    pub fn capacity(&self) -> u64 {
        self.disk.capacity()
    }

    /// Get partition information.
    pub fn get_partition(&self, index: usize) -> Option<&crate::partition::Partition> {
        self.partitions.get(index)
    }

    /// Get the total number of sectors on the disk.
    pub fn sector_count(&self) -> u64 {
        self.disk.sector_count()
    }
}
