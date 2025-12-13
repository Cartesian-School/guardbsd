//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: storage
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Obsługa tablic partycji.

const MBR_SIGNATURE: [u8; 2] = [0x55, 0xAA];
const MBR_PARTITION_TABLE_OFFSET: usize = 446;
const MBR_PARTITION_ENTRY_SIZE: usize = 16;
const MAX_PARTITIONS: usize = 4;

// MBR partition types
pub const PART_TYPE_EMPTY: u8 = 0x00;
pub const PART_TYPE_FAT16: u8 = 0x06;
pub const PART_TYPE_NTFS: u8 = 0x07;
pub const PART_TYPE_FAT32: u8 = 0x0B;
pub const PART_TYPE_FAT32_LBA: u8 = 0x0C;
pub const PART_TYPE_EXTENDED: u8 = 0x05;
pub const PART_TYPE_EXTENDED_LBA: u8 = 0x0F;
pub const PART_TYPE_LINUX: u8 = 0x83;
pub const PART_TYPE_LINUX_SWAP: u8 = 0x82;

// Boot flags
const BOOT_FLAG_INACTIVE: u8 = 0x00;
const BOOT_FLAG_ACTIVE: u8 = 0x80;

#[derive(Clone, Copy, Debug)]
pub struct Partition {
    pub start_lba: u64,
    pub sectors: u64,
    pub part_type: u8,
    pub active: bool,
}

impl Partition {
    pub const fn new() -> Self {
        Self {
            start_lba: 0,
            sectors: 0,
            part_type: 0,
            active: false,
        }
    }

    /// Returns the ending LBA (exclusive).
    pub fn end_lba(&self) -> Option<u64> {
        self.start_lba.checked_add(self.sectors)
    }

    /// Check if this partition is an extended partition.
    pub fn is_extended(&self) -> bool {
        matches!(
            self.part_type,
            PART_TYPE_EXTENDED | PART_TYPE_EXTENDED_LBA | 0x85
        )
    }

    /// Check if this partition is empty.
    pub fn is_empty(&self) -> bool {
        self.part_type == PART_TYPE_EMPTY || self.sectors == 0
    }
}

pub struct PartitionTable {
    partitions: [Partition; MAX_PARTITIONS],
    count: usize,
}

impl PartitionTable {
    pub const fn new() -> Self {
        Self {
            partitions: [Partition::new(); MAX_PARTITIONS],
            count: 0,
        }
    }

    /// Parse MBR partition table from 512-byte MBR sector.
    pub fn parse_mbr(&mut self, mbr: &[u8]) -> Result<(), i64> {
        if mbr.len() < 512 {
            return Err(-22); // EINVAL
        }

        // Check MBR signature (last 2 bytes)
        if mbr[510] != MBR_SIGNATURE[0] || mbr[511] != MBR_SIGNATURE[1] {
            return Err(-22); // EINVAL - invalid MBR signature
        }

        self.count = 0;

        // Parse 4 partition entries
        for i in 0..MAX_PARTITIONS {
            let offset = MBR_PARTITION_TABLE_OFFSET + (i * MBR_PARTITION_ENTRY_SIZE);
            
            let status = mbr[offset];
            let part_type = mbr[offset + 4];

            // Skip empty partitions
            if part_type == PART_TYPE_EMPTY {
                continue;
            }

            // Validate boot flag
            if status != BOOT_FLAG_INACTIVE && status != BOOT_FLAG_ACTIVE {
                // Invalid boot flag - skip this partition
                continue;
            }

            let start_lba = u32::from_le_bytes([
                mbr[offset + 8],
                mbr[offset + 9],
                mbr[offset + 10],
                mbr[offset + 11],
            ]) as u64;

            let sectors = u32::from_le_bytes([
                mbr[offset + 12],
                mbr[offset + 13],
                mbr[offset + 14],
                mbr[offset + 15],
            ]) as u64;

            // Validation: partition must have non-zero size
            if sectors == 0 {
                continue;
            }

            // Validation: starting at LBA 0 is suspicious (MBR is there)
            // but technically valid - we'll accept it but it's unusual
            if start_lba == 0 {
                // Log warning if logging available
                // For now, we accept it
            }

            // Validation: check for address space overflow
            let end_lba = start_lba.checked_add(sectors)
                .ok_or(-22)?; // EINVAL - address overflow

            // Validation: check for overlapping with existing partitions
            for j in 0..self.count {
                let existing = &self.partitions[j];
                
                // Calculate existing partition end with overflow check
                let existing_end = existing.start_lba
                    .checked_add(existing.sectors)
                    .ok_or(-22)?; // EINVAL - this shouldn't happen if we validated earlier

                // Check for overlap: [start, end) intervals
                // Overlap if: start < existing_end AND end > existing_start
                if start_lba < existing_end && end_lba > existing.start_lba {
                    return Err(-22); // EINVAL - overlapping partitions
                }
            }

            // Add partition
            self.partitions[self.count] = Partition {
                start_lba,
                sectors,
                part_type,
                active: status == BOOT_FLAG_ACTIVE,
            };
            self.count += 1;
        }

        Ok(())
    }

    /// Get partition by index (0-3).
    pub fn get(&self, index: usize) -> Option<&Partition> {
        if index < self.count {
            Some(&self.partitions[index])
        } else {
            None
        }
    }

    /// Get mutable partition by index (0-3).
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Partition> {
        if index < self.count {
            Some(&mut self.partitions[index])
        } else {
            None
        }
    }

    /// Get number of valid partitions found.
    pub fn count(&self) -> usize {
        self.count
    }

    /// Check if partition table is empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Find the active (bootable) partition.
    pub fn find_active(&self) -> Option<usize> {
        for i in 0..self.count {
            if self.partitions[i].active {
                return Some(i);
            }
        }
        None
    }

    /// Iterate over all valid partitions.
    pub fn iter(&self) -> impl Iterator<Item = &Partition> {
        self.partitions[..self.count].iter()
    }
}