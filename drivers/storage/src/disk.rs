//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: storage
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Abstrakcja I/O dysku.

const SECTOR_SIZE: usize = 512;

pub struct Disk {
    sectors: u64,
    cache: [u8; SECTOR_SIZE],
    cache_lba: Option<u64>,
    cache_dirty: bool, // True if cache has been modified
    // TODO: Add actual storage backend (ATA/AHCI/NVMe/RAM disk)
    // For now, we simulate with in-memory storage
    #[cfg(feature = "mock_storage")]
    mock_storage: alloc::vec::Vec<[u8; SECTOR_SIZE]>,
}

impl Disk {
    pub const fn new(sectors: u64) -> Self {
        Self {
            sectors,
            cache: [0; SECTOR_SIZE],
            cache_lba: None,
            cache_dirty: false,
            #[cfg(feature = "mock_storage")]
            mock_storage: alloc::vec::Vec::new(),
        }
    }

    /// Read a sector from disk into the provided buffer.
    /// Buffer must be at least SECTOR_SIZE (512) bytes.
    pub fn read_sector(&mut self, lba: u64, buf: &mut [u8]) -> Result<usize, i64> {
        // Validate LBA
        if lba >= self.sectors {
            return Err(-22); // EINVAL
        }

        // Validate buffer size
        if buf.len() < SECTOR_SIZE {
            return Err(-22); // EINVAL
        }

        // Check if sector is in cache
        if self.cache_lba == Some(lba) {
            // Cache hit - return cached data
            buf[..SECTOR_SIZE].copy_from_slice(&self.cache);
        } else {
            // Cache miss - flush cache if dirty, then read from disk
            if self.cache_dirty {
                self.flush_cache_internal()?;
            }

            // Read from disk
            self.read_from_disk(lba, &mut self.cache)?;

            // Update cache
            buf[..SECTOR_SIZE].copy_from_slice(&self.cache);
            self.cache_lba = Some(lba);
            self.cache_dirty = false;
        }

        Ok(SECTOR_SIZE)
    }

    /// Write a sector to disk from the provided buffer.
    /// Buffer must be at least SECTOR_SIZE (512) bytes.
    pub fn write_sector(&mut self, lba: u64, buf: &[u8]) -> Result<usize, i64> {
        // Validate LBA
        if lba >= self.sectors {
            return Err(-22); // EINVAL
        }

        // Validate buffer size
        if buf.len() < SECTOR_SIZE {
            return Err(-22); // EINVAL
        }

        // If writing to different sector, flush old cache first
        if self.cache_lba != Some(lba) && self.cache_dirty {
            self.flush_cache_internal()?;
        }

        // Write to cache (write-back strategy)
        self.cache[..SECTOR_SIZE].copy_from_slice(&buf[..SECTOR_SIZE]);
        self.cache_lba = Some(lba);
        self.cache_dirty = true;

        // For critical data, consider immediate write-through:
        // self.write_to_disk(lba, &self.cache)?;
        // self.cache_dirty = false;

        Ok(SECTOR_SIZE)
    }

    /// Flush dirty cache to disk if needed.
    pub fn flush_cache(&mut self) -> Result<(), i64> {
        if self.cache_dirty {
            self.flush_cache_internal()?;
        }
        Ok(())
    }

    /// Get total disk capacity in bytes.
    pub fn capacity(&self) -> u64 {
        self.sectors * SECTOR_SIZE as u64
    }

    /// Get total number of sectors.
    pub fn sector_count(&self) -> u64 {
        self.sectors
    }

    /// Get sector size in bytes.
    pub const fn sector_size(&self) -> usize {
        SECTOR_SIZE
    }

    // Internal helper methods

    fn flush_cache_internal(&mut self) -> Result<(), i64> {
        if let Some(lba) = self.cache_lba {
            if self.cache_dirty {
                self.write_to_disk(lba, &self.cache)?;
                self.cache_dirty = false;
            }
        }
        Ok(())
    }

    fn read_from_disk(&mut self, lba: u64, buf: &mut [u8; SECTOR_SIZE]) -> Result<(), i64> {
        // TODO: Replace with actual disk I/O (ATA/AHCI/NVMe)
        // For now, simulate with zeros or mock storage

        #[cfg(feature = "mock_storage")]
        {
            if (lba as usize) < self.mock_storage.len() {
                buf.copy_from_slice(&self.mock_storage[lba as usize]);
            } else {
                buf.fill(0);
            }
        }

        #[cfg(not(feature = "mock_storage"))]
        {
            // Simulate empty disk
            buf.fill(0);
        }

        Ok(())
    }

    fn write_to_disk(&mut self, lba: u64, buf: &[u8; SECTOR_SIZE]) -> Result<(), i64> {
        // TODO: Replace with actual disk I/O (ATA/AHCI/NVMe)
        // For now, simulate or use mock storage

        #[cfg(feature = "mock_storage")]
        {
            // Expand mock storage if needed
            while self.mock_storage.len() <= lba as usize {
                self.mock_storage.push([0; SECTOR_SIZE]);
            }
            self.mock_storage[lba as usize].copy_from_slice(buf);
        }

        #[cfg(not(feature = "mock_storage"))]
        {
            // Simulate write (no-op for now)
            let _ = (lba, buf); // Silence unused warnings
        }

        Ok(())
    }
}

impl Drop for Disk {
    fn drop(&mut self) {
        // Flush cache on drop to prevent data loss
        let _ = self.flush_cache();
    }
}
