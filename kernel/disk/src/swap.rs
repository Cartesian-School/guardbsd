//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: disk
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Obsługa partycji swap (format Linux).

#![no_std]

use crate::block_device::*;

// Swap signature
pub const SWAP_SIGNATURE: &[u8; 10] = b"SWAPSPACE2";
pub const SWAP_SIGNATURE_OFFSET: usize = 4086; // 4096 - 10

// Swap header structure
#[repr(C)]
#[derive(Copy, Clone)]
pub struct SwapHeader {
    pub version: u32,
    pub last_page: u32,
    pub nr_badpages: u32,
    pub padding: [u32; 5],
    pub badpages: [u32; 1],
}

impl SwapHeader {
    pub const fn empty() -> Self {
        Self {
            version: 0,
            last_page: 0,
            nr_badpages: 0,
            padding: [0; 5],
            badpages: [0],
        }
    }
}

pub struct SwapSpace {
    pub device_id: u32,
    pub start_sector: u64,
    pub size_sectors: u64,
    pub header: SwapHeader,
    pub active: bool,
}

impl SwapSpace {
    pub const fn new(device_id: u32, start_sector: u64, size_sectors: u64) -> Self {
        Self {
            device_id,
            start_sector,
            size_sectors,
            header: SwapHeader::empty(),
            active: false,
        }
    }

    pub fn detect_swap(device: &BlockDevice, start_sector: u64) -> Result<Self, DiskError> {
        // Read first sector (4096 bytes)
        let mut sector = [0u8; 4096];
        // In real implementation, would call device.read_sectors()

        // Check for swap signature
        if &sector[SWAP_SIGNATURE_OFFSET..SWAP_SIGNATURE_OFFSET + 10] == SWAP_SIGNATURE {
            // Parse swap header
            let mut swap = SwapSpace::new(device.id, start_sector, 0);

            // Read header (simplified)
            swap.header.version = u32::from_le_bytes([sector[0], sector[1], sector[2], sector[3]]);

            Ok(swap)
        } else {
            Err(DiskError::NotSupported)
        }
    }

    pub fn activate(&mut self) -> Result<(), DiskError> {
        if self.active {
            return Err(DiskError::DeviceBusy);
        }

        // TODO: Register with memory manager
        self.active = true;
        Ok(())
    }

    pub fn deactivate(&mut self) -> Result<(), DiskError> {
        if !self.active {
            return Ok(());
        }

        // TODO: Unregister from memory manager
        self.active = false;
        Ok(())
    }

    pub fn get_size_mb(&self) -> u64 {
        (self.size_sectors * 512) / (1024 * 1024)
    }
}

// Global swap registry
pub const MAX_SWAP: usize = 4;

pub struct SwapRegistry {
    swaps: [Option<SwapSpace>; MAX_SWAP],
    count: usize,
}

impl SwapRegistry {
    pub const fn new() -> Self {
        Self {
            swaps: [None, None, None, None],
            count: 0,
        }
    }

    pub fn register(&mut self, swap: SwapSpace) -> Option<usize> {
        if self.count < MAX_SWAP {
            self.swaps[self.count] = Some(swap);
            let id = self.count;
            self.count += 1;
            Some(id)
        } else {
            None
        }
    }

    pub fn get(&self, id: usize) -> Option<&SwapSpace> {
        if id < self.count {
            self.swaps[id].as_ref()
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut SwapSpace> {
        if id < self.count {
            self.swaps[id].as_mut()
        } else {
            None
        }
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn get_total_swap_mb(&self) -> u64 {
        let mut total = 0;
        for i in 0..self.count {
            if let Some(ref swap) = self.swaps[i] {
                if swap.active {
                    total += swap.get_size_mb();
                }
            }
        }
        total
    }
}

static mut SWAP_REGISTRY: SwapRegistry = SwapRegistry::new();

pub fn register_swap(swap: SwapSpace) -> Option<usize> {
    unsafe { SWAP_REGISTRY.register(swap) }
}

pub fn get_swap(id: usize) -> Option<&'static SwapSpace> {
    unsafe { SWAP_REGISTRY.get(id) }
}

pub fn get_swap_mut(id: usize) -> Option<&'static mut SwapSpace> {
    unsafe { SWAP_REGISTRY.get_mut(id) }
}

pub fn total_swap_mb() -> u64 {
    unsafe { SWAP_REGISTRY.get_total_swap_mb() }
}

/// Activate swap partition
pub fn swapon(id: usize) -> Result<(), DiskError> {
    if let Some(swap) = get_swap_mut(id) {
        swap.activate()
    } else {
        Err(DiskError::DeviceNotFound)
    }
}

/// Deactivate swap partition
pub fn swapoff(id: usize) -> Result<(), DiskError> {
    if let Some(swap) = get_swap_mut(id) {
        swap.deactivate()
    } else {
        Err(DiskError::DeviceNotFound)
    }
}
