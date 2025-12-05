// servers/guardzfs/src/vdev.rs
// GuardZFS Virtual Devices
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

pub const MAX_VDEV_CHILDREN: usize = 8;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq)]
pub enum VdevType {
    Single = 0,    // Single disk (no redundancy)
    Mirror = 1,    // 2-way mirror (RAID-1)
    RaidZ1 = 2,    // Single parity (RAID-5)
    RaidZ2 = 3,    // Double parity (RAID-6)
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq)]
pub enum VdevState {
    Offline = 0,
    Degraded = 1,
    Online = 2,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct VirtualDevice {
    pub vdev_type: VdevType,
    pub state: VdevState,
    
    // Child devices (disk IDs from disk registry)
    pub children: [u32; MAX_VDEV_CHILDREN],
    pub child_count: u8,
    
    // Statistics
    pub total_blocks: u64,
    pub free_blocks: u64,
    
    // RAID-Z specific
    pub parity_count: u8,         // 1 or 2
    pub stripe_width: u8,         // Number of data disks
    
    // Error tracking
    pub read_errors: u32,
    pub write_errors: u32,
    pub checksum_errors: u32,
}

impl VirtualDevice {
    pub const fn empty() -> Self {
        Self {
            vdev_type: VdevType::Single,
            state: VdevState::Offline,
            children: [0; MAX_VDEV_CHILDREN],
            child_count: 0,
            total_blocks: 0,
            free_blocks: 0,
            parity_count: 0,
            stripe_width: 0,
            read_errors: 0,
            write_errors: 0,
            checksum_errors: 0,
        }
    }
    
    pub fn new_single(disk_id: u32, total_blocks: u64) -> Self {
        let mut vdev = Self::empty();
        vdev.vdev_type = VdevType::Single;
        vdev.state = VdevState::Online;
        vdev.children[0] = disk_id;
        vdev.child_count = 1;
        vdev.total_blocks = total_blocks;
        vdev.free_blocks = total_blocks;
        vdev
    }
    
    pub fn new_mirror(disk1: u32, disk2: u32, blocks_per_disk: u64) -> Self {
        let mut vdev = Self::empty();
        vdev.vdev_type = VdevType::Mirror;
        vdev.state = VdevState::Online;
        vdev.children[0] = disk1;
        vdev.children[1] = disk2;
        vdev.child_count = 2;
        vdev.total_blocks = blocks_per_disk; // Mirror doesn't increase capacity
        vdev.free_blocks = blocks_per_disk;
        vdev
    }
    
    pub fn new_raidz1(disks: &[u32], blocks_per_disk: u64) -> Self {
        let mut vdev = Self::empty();
        vdev.vdev_type = VdevType::RaidZ1;
        vdev.state = VdevState::Online;
        
        let disk_count = disks.len().min(MAX_VDEV_CHILDREN);
        vdev.child_count = disk_count as u8;
        for (i, &disk_id) in disks.iter().enumerate().take(disk_count) {
            vdev.children[i] = disk_id;
        }
        
        vdev.parity_count = 1;
        vdev.stripe_width = (disk_count - 1) as u8;
        
        // Usable space = (N-1) disks × blocks_per_disk
        vdev.total_blocks = blocks_per_disk * (disk_count - 1) as u64;
        vdev.free_blocks = vdev.total_blocks;
        
        vdev
    }
    
    pub fn new_raidz2(disks: &[u32], blocks_per_disk: u64) -> Self {
        let mut vdev = Self::empty();
        vdev.vdev_type = VdevType::RaidZ2;
        vdev.state = VdevState::Online;
        
        let disk_count = disks.len().min(MAX_VDEV_CHILDREN);
        vdev.child_count = disk_count as u8;
        for (i, &disk_id) in disks.iter().enumerate().take(disk_count) {
            vdev.children[i] = disk_id;
        }
        
        vdev.parity_count = 2;
        vdev.stripe_width = (disk_count - 2) as u8;
        
        // Usable space = (N-2) disks × blocks_per_disk
        vdev.total_blocks = blocks_per_disk * (disk_count - 2) as u64;
        vdev.free_blocks = vdev.total_blocks;
        
        vdev
    }
    
    pub fn allocate_block(&mut self) -> Option<u64> {
        if self.free_blocks > 0 {
            let block = self.total_blocks - self.free_blocks;
            self.free_blocks -= 1;
            Some(block)
        } else {
            None
        }
    }
    
    pub fn free_block(&mut self) {
        if self.free_blocks < self.total_blocks {
            self.free_blocks += 1;
        }
    }
    
    pub fn can_tolerate_failures(&self) -> u8 {
        match self.vdev_type {
            VdevType::Single => 0,
            VdevType::Mirror => 1,
            VdevType::RaidZ1 => 1,
            VdevType::RaidZ2 => 2,
        }
    }
}

