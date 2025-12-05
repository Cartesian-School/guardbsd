// drivers/storage/src/disk.rs
// Disk I/O abstraction
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

const SECTOR_SIZE: usize = 512;

pub struct Disk {
    sectors: u64,
    cache: [u8; 512],
}

impl Disk {
    pub const fn new(sectors: u64) -> Self {
        Self {
            sectors,
            cache: [0; 512],
        }
    }

    pub fn read_sector(&mut self, lba: u64, buf: &mut [u8]) -> Result<usize, i64> {
        if lba >= self.sectors {
            return Err(-22); // EINVAL
        }

        if buf.len() < SECTOR_SIZE {
            return Err(-22); // EINVAL
        }

        // Simulate disk read (in real implementation, would use ATA/AHCI)
        buf[..SECTOR_SIZE].fill(0);
        Ok(SECTOR_SIZE)
    }

    pub fn write_sector(&mut self, lba: u64, buf: &[u8]) -> Result<usize, i64> {
        if lba >= self.sectors {
            return Err(-22); // EINVAL
        }

        if buf.len() < SECTOR_SIZE {
            return Err(-22); // EINVAL
        }

        // Simulate disk write (in real implementation, would use ATA/AHCI)
        self.cache[..SECTOR_SIZE].copy_from_slice(&buf[..SECTOR_SIZE]);
        Ok(SECTOR_SIZE)
    }

    pub fn capacity(&self) -> u64 {
        self.sectors * SECTOR_SIZE as u64
    }
}
