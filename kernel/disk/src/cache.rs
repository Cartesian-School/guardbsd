//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: disk
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Cache bloków dla I/O dysku.

#![no_std]

use crate::block_device::*;

pub const CACHE_SIZE: usize = 128; // 128 blocks = 512KB

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CacheEntry {
    pub device_id: u32,
    pub block_num: u64,
    pub data: [u8; BLOCK_SIZE],
    pub valid: bool,
    pub dirty: bool,
    pub access_time: u64,
}

impl CacheEntry {
    pub const fn empty() -> Self {
        Self {
            device_id: 0,
            block_num: 0,
            data: [0; BLOCK_SIZE],
            valid: false,
            dirty: false,
            access_time: 0,
        }
    }
}

pub struct BlockCache {
    entries: [CacheEntry; CACHE_SIZE],
    access_counter: u64,
    hits: u64,
    misses: u64,
}

impl BlockCache {
    pub const fn new() -> Self {
        Self {
            entries: [CacheEntry::empty(); CACHE_SIZE],
            access_counter: 0,
            hits: 0,
            misses: 0,
        }
    }

    fn find_entry(&self, device_id: u32, block_num: u64) -> Option<usize> {
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.valid && entry.device_id == device_id && entry.block_num == block_num {
                return Some(i);
            }
        }
        None
    }

    fn find_lru_entry(&self) -> usize {
        let mut lru_idx = 0;
        let mut lru_time = u64::MAX;

        for (i, entry) in self.entries.iter().enumerate() {
            if !entry.valid {
                return i; // Use invalid entry first
            }
            if entry.access_time < lru_time {
                lru_time = entry.access_time;
                lru_idx = i;
            }
        }

        lru_idx
    }

    pub fn read(
        &mut self,
        device: &BlockDevice,
        block_num: u64,
        buf: &mut [u8; BLOCK_SIZE],
    ) -> Result<(), DiskError> {
        self.access_counter += 1;

        // Check if block is in cache
        if let Some(idx) = self.find_entry(device.id, block_num) {
            self.hits += 1;
            buf.copy_from_slice(&self.entries[idx].data);
            self.entries[idx].access_time = self.access_counter;
            return Ok(());
        }

        // Cache miss - read from disk
        self.misses += 1;
        device.read_block(block_num, buf)?;

        // Find LRU entry to evict
        let idx = self.find_lru_entry();

        // Write back if dirty
        if self.entries[idx].valid && self.entries[idx].dirty {
            if let Some(dev) = get_disk(self.entries[idx].device_id) {
                let _ = dev.write_block(self.entries[idx].block_num, &self.entries[idx].data);
            }
        }

        // Insert new entry
        self.entries[idx].device_id = device.id;
        self.entries[idx].block_num = block_num;
        self.entries[idx].data.copy_from_slice(buf);
        self.entries[idx].valid = true;
        self.entries[idx].dirty = false;
        self.entries[idx].access_time = self.access_counter;

        Ok(())
    }

    pub fn write(
        &mut self,
        device: &BlockDevice,
        block_num: u64,
        buf: &[u8; BLOCK_SIZE],
    ) -> Result<(), DiskError> {
        self.access_counter += 1;

        // Check if block is in cache
        if let Some(idx) = self.find_entry(device.id, block_num) {
            self.entries[idx].data.copy_from_slice(buf);
            self.entries[idx].dirty = true;
            self.entries[idx].access_time = self.access_counter;
            return Ok(());
        }

        // Not in cache - find LRU entry
        let idx = self.find_lru_entry();

        // Write back if dirty
        if self.entries[idx].valid && self.entries[idx].dirty {
            if let Some(dev) = get_disk(self.entries[idx].device_id) {
                let _ = dev.write_block(self.entries[idx].block_num, &self.entries[idx].data);
            }
        }

        // Insert new entry
        self.entries[idx].device_id = device.id;
        self.entries[idx].block_num = block_num;
        self.entries[idx].data.copy_from_slice(buf);
        self.entries[idx].valid = true;
        self.entries[idx].dirty = true;
        self.entries[idx].access_time = self.access_counter;

        Ok(())
    }

    pub fn flush(&mut self) -> usize {
        let mut flushed = 0;

        for entry in &mut self.entries {
            if entry.valid && entry.dirty {
                if let Some(device) = get_disk(entry.device_id) {
                    if device.write_block(entry.block_num, &entry.data).is_ok() {
                        entry.dirty = false;
                        flushed += 1;
                    }
                }
            }
        }

        flushed
    }

    pub fn invalidate(&mut self, device_id: u32) {
        for entry in &mut self.entries {
            if entry.valid && entry.device_id == device_id {
                // Write back if dirty
                if entry.dirty {
                    if let Some(device) = get_disk(device_id) {
                        let _ = device.write_block(entry.block_num, &entry.data);
                    }
                }
                entry.valid = false;
            }
        }
    }

    pub fn get_stats(&self) -> (u64, u64, u32) {
        let total = self.hits + self.misses;
        let hit_rate = if total > 0 {
            ((self.hits * 100) / total) as u32
        } else {
            0
        };

        (self.hits, self.misses, hit_rate)
    }
}

static mut GLOBAL_CACHE: BlockCache = BlockCache::new();

pub fn cache_read(
    device: &BlockDevice,
    block_num: u64,
    buf: &mut [u8; BLOCK_SIZE],
) -> Result<(), DiskError> {
    unsafe { GLOBAL_CACHE.read(device, block_num, buf) }
}

pub fn cache_write(
    device: &BlockDevice,
    block_num: u64,
    buf: &[u8; BLOCK_SIZE],
) -> Result<(), DiskError> {
    unsafe { GLOBAL_CACHE.write(device, block_num, buf) }
}

pub fn cache_flush() -> usize {
    unsafe { GLOBAL_CACHE.flush() }
}

pub fn cache_invalidate(device_id: u32) {
    unsafe { GLOBAL_CACHE.invalidate(device_id) }
}

pub fn cache_stats() -> (u64, u64, u32) {
    unsafe { GLOBAL_CACHE.get_stats() }
}
