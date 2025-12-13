//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: guardzfs
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Pula storage GuardZFS (zpool).

use crate::blockptr::*;
use crate::vdev::*;

pub const MAX_POOLS: usize = 4;
pub const MAX_VDEVS_PER_POOL: usize = 8;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq)]
pub enum PoolHealth {
    Online = 0,   // All vdevs healthy
    Degraded = 1, // Some vdevs failed but data intact
    Faulted = 2,  // Too many failures, data loss
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PoolLabel {
    pub magic: [u8; 8], // "GUARDZFS"
    pub version: u32,
    pub pool_guid: u64,
    pub name: [u8; 64],
    pub state: u8,
    pub txg: u64, // Current transaction group
    pub uber_block_offset: u64,
    pub checksum: [u8; 32],
}

impl PoolLabel {
    pub const MAGIC: &'static [u8; 8] = b"GUARDZFS";

    pub const fn new() -> Self {
        Self {
            magic: *Self::MAGIC,
            version: 1,
            pool_guid: 0,
            name: [0; 64],
            state: 0,
            txg: 0,
            uber_block_offset: 0,
            checksum: [0; 32],
        }
    }

    pub fn validate(&self) -> bool {
        &self.magic == Self::MAGIC && self.version == 1
    }
}

#[repr(C)]
pub struct StoragePool {
    pub name: [u8; 64],
    pub guid: u64,
    pub health: PoolHealth,

    // Virtual devices
    pub vdevs: [VirtualDevice; MAX_VDEVS_PER_POOL],
    pub vdev_count: u8,

    // Capacity
    pub total_blocks: u64,
    pub free_blocks: u64,
    pub allocated_blocks: u64,

    // Transaction groups
    pub current_txg: u64,
    pub syncing_txg: u64,

    // Statistics
    pub read_ops: u64,
    pub write_ops: u64,
    pub bytes_read: u64,
    pub bytes_written: u64,
}

impl StoragePool {
    pub const fn new() -> Self {
        Self {
            name: [0; 64],
            guid: 0,
            health: PoolHealth::Online,
            vdevs: [VirtualDevice::empty(); MAX_VDEVS_PER_POOL],
            vdev_count: 0,
            total_blocks: 0,
            free_blocks: 0,
            allocated_blocks: 0,
            current_txg: 1,
            syncing_txg: 0,
            read_ops: 0,
            write_ops: 0,
            bytes_read: 0,
            bytes_written: 0,
        }
    }

    pub fn init(&mut self, name: &str, guid: u64) {
        let name_bytes = name.as_bytes();
        let len = name_bytes.len().min(63);
        self.name[..len].copy_from_slice(&name_bytes[..len]);
        self.guid = guid;
        self.current_txg = 1;
    }

    pub fn add_vdev(&mut self, vdev: VirtualDevice) -> Result<(), &'static str> {
        if self.vdev_count >= MAX_VDEVS_PER_POOL as u8 {
            return Err("Pool full");
        }

        self.vdevs[self.vdev_count as usize] = vdev;
        self.vdev_count += 1;

        // Update pool capacity
        self.total_blocks += vdev.total_blocks;
        self.free_blocks += vdev.free_blocks;

        Ok(())
    }

    pub fn allocate_block(&mut self) -> Option<(u8, u64)> {
        // Round-robin allocation across vdevs
        for i in 0..self.vdev_count {
            if let Some(block) = self.vdevs[i as usize].allocate_block() {
                self.free_blocks -= 1;
                self.allocated_blocks += 1;
                return Some((i, block));
            }
        }
        None
    }

    pub fn free_block(&mut self, vdev_id: u8, _block: u64) {
        if (vdev_id as usize) < self.vdev_count as usize {
            self.vdevs[vdev_id as usize].free_block();
            self.free_blocks += 1;
            self.allocated_blocks -= 1;
        }
    }

    pub fn begin_txg(&mut self) {
        self.current_txg += 1;
    }

    pub fn sync_txg(&mut self) {
        self.syncing_txg = self.current_txg;
        // TODO: Flush all dirty data to disk
        // TODO: Write uber block
    }

    pub fn get_health(&mut self) -> PoolHealth {
        let mut online_count = 0;
        let mut degraded_count = 0;

        for i in 0..self.vdev_count {
            match self.vdevs[i as usize].state {
                VdevState::Online => online_count += 1,
                VdevState::Degraded => degraded_count += 1,
                VdevState::Offline => {}
            }
        }

        if online_count == self.vdev_count {
            self.health = PoolHealth::Online;
        } else if degraded_count > 0 {
            self.health = PoolHealth::Degraded;
        } else {
            self.health = PoolHealth::Faulted;
        }

        self.health
    }

    pub fn get_used_percent(&self) -> u8 {
        if self.total_blocks == 0 {
            return 0;
        }
        ((self.allocated_blocks * 100) / self.total_blocks) as u8
    }
}

pub struct PoolRegistry {
    pools: [Option<StoragePool>; MAX_POOLS],
    count: usize,
}

impl PoolRegistry {
    pub const fn new() -> Self {
        Self {
            pools: [None, None, None, None],
            count: 0,
        }
    }

    pub fn create_pool(&mut self, name: &str, guid: u64) -> Option<usize> {
        if self.count >= MAX_POOLS {
            return None;
        }

        let mut pool = StoragePool::new();
        pool.init(name, guid);

        self.pools[self.count] = Some(pool);
        let id = self.count;
        self.count += 1;

        Some(id)
    }

    pub fn get(&self, id: usize) -> Option<&StoragePool> {
        if id < self.count {
            self.pools[id].as_ref()
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut StoragePool> {
        if id < self.count {
            self.pools[id].as_mut()
        } else {
            None
        }
    }
}

static mut POOL_REGISTRY: PoolRegistry = PoolRegistry::new();

pub fn create_pool(name: &str, guid: u64) -> Option<usize> {
    unsafe { POOL_REGISTRY.create_pool(name, guid) }
}

pub fn get_pool(id: usize) -> Option<&'static StoragePool> {
    unsafe { POOL_REGISTRY.get(id) }
}

pub fn get_pool_mut(id: usize) -> Option<&'static mut StoragePool> {
    unsafe { POOL_REGISTRY.get_mut(id) }
}
