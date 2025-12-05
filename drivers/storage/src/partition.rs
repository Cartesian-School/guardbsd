// drivers/storage/src/partition.rs
// Partition table support
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#[derive(Clone, Copy)]
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
}

pub struct PartitionTable {
    partitions: [Partition; 4],
    count: usize,
}

impl PartitionTable {
    pub const fn new() -> Self {
        Self {
            partitions: [Partition::new(); 4],
            count: 0,
        }
    }

    pub fn parse_mbr(&mut self, mbr: &[u8]) -> Result<(), i64> {
        if mbr.len() < 512 {
            return Err(-22); // EINVAL
        }

        // Check MBR signature
        if mbr[510] != 0x55 || mbr[511] != 0xAA {
            return Err(-22); // EINVAL
        }

        // Parse partition entries (offset 446)
        self.count = 0;
        for i in 0..4 {
            let offset = 446 + (i * 16);
            let status = mbr[offset];
            let part_type = mbr[offset + 4];

            if part_type != 0 {
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

                self.partitions[self.count] = Partition {
                    start_lba,
                    sectors,
                    part_type,
                    active: status == 0x80,
                };
                self.count += 1;
            }
        }

        Ok(())
    }

    pub fn get(&self, index: usize) -> Option<&Partition> {
        if index < self.count {
            Some(&self.partitions[index])
        } else {
            None
        }
    }

    pub fn count(&self) -> usize {
        self.count
    }
}
