// kernel/disk/src/block_device.rs
// Block Device Abstraction Layer
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#![no_std]

pub const BLOCK_SIZE: usize = 4096;
pub const SECTOR_SIZE: usize = 512;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum DriverType {
    ATA = 0,
    AHCI = 1,
    NVMe = 2,
    VirtIO = 3,
}

#[derive(Copy, Clone, Debug)]
pub enum DiskError {
    DeviceNotFound,
    InvalidBlock,
    ReadError,
    WriteError,
    Timeout,
    DmaError,
    DeviceBusy,
    NotSupported,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DiskInfo {
    pub model: [u8; 40],
    pub serial: [u8; 20],
    pub firmware: [u8; 8],
    pub total_sectors: u64,
    pub sector_size: u16,
    pub supports_lba48: bool,
    pub supports_dma: bool,
    pub supports_ncq: bool,
}

impl DiskInfo {
    pub const fn empty() -> Self {
        Self {
            model: [0; 40],
            serial: [0; 20],
            firmware: [0; 8],
            total_sectors: 0,
            sector_size: 512,
            supports_lba48: false,
            supports_dma: false,
            supports_ncq: false,
        }
    }
}

pub trait DiskDriver {
    /// Read sectors from disk
    fn read_sectors(&mut self, lba: u64, count: u32, buf: &mut [u8]) -> Result<(), DiskError>;
    
    /// Write sectors to disk
    fn write_sectors(&mut self, lba: u64, count: u32, buf: &[u8]) -> Result<(), DiskError>;
    
    /// Flush write cache to disk
    fn flush(&mut self) -> Result<(), DiskError>;
    
    /// Get disk information
    fn identify(&mut self) -> Result<DiskInfo, DiskError>;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BlockDevice {
    pub id: u32,
    pub driver_type: DriverType,
    pub block_size: usize,
    pub total_blocks: u64,
    pub info: DiskInfo,
    // Driver pointer would go here in full implementation
}

impl BlockDevice {
    pub const fn new(id: u32, driver_type: DriverType) -> Self {
        Self {
            id,
            driver_type,
            block_size: BLOCK_SIZE,
            total_blocks: 0,
            info: DiskInfo::empty(),
        }
    }
    
    pub fn init(&mut self, info: DiskInfo) {
        self.info = info;
        self.total_blocks = (info.total_sectors * info.sector_size as u64) / BLOCK_SIZE as u64;
    }
    
    /// Read a 4KB block
    pub fn read_block(&self, block_num: u64, _buf: &mut [u8; BLOCK_SIZE]) -> Result<(), DiskError> {
        if block_num >= self.total_blocks {
            return Err(DiskError::InvalidBlock);
        }
        
        // Convert block to sectors (4KB / 512 = 8 sectors per block)
        let _lba = block_num * 8;
        
        // In full implementation, call driver's read_sectors
        // For now, return error as we don't have actual driver reference
        Err(DiskError::NotSupported)
    }
    
    /// Write a 4KB block
    pub fn write_block(&self, block_num: u64, _buf: &[u8; BLOCK_SIZE]) -> Result<(), DiskError> {
        if block_num >= self.total_blocks {
            return Err(DiskError::InvalidBlock);
        }
        
        let _lba = block_num * 8;
        
        // In full implementation, call driver's write_sectors
        Err(DiskError::NotSupported)
    }
    
    /// Read multiple blocks
    pub fn read_blocks(&self, start_block: u64, count: u32, buf: &mut [u8]) -> Result<usize, DiskError> {
        if buf.len() < (count as usize * BLOCK_SIZE) {
            return Err(DiskError::InvalidBlock);
        }
        
        for i in 0..count {
            let block_num = start_block + i as u64;
            let offset = i as usize * BLOCK_SIZE;
            let mut block_buf = [0u8; BLOCK_SIZE];
            
            self.read_block(block_num, &mut block_buf)?;
            buf[offset..offset + BLOCK_SIZE].copy_from_slice(&block_buf);
        }
        
        Ok(count as usize * BLOCK_SIZE)
    }
    
    /// Write multiple blocks
    pub fn write_blocks(&self, start_block: u64, count: u32, buf: &[u8]) -> Result<usize, DiskError> {
        if buf.len() < (count as usize * BLOCK_SIZE) {
            return Err(DiskError::InvalidBlock);
        }
        
        for i in 0..count {
            let block_num = start_block + i as u64;
            let offset = i as usize * BLOCK_SIZE;
            let mut block_buf = [0u8; BLOCK_SIZE];
            
            block_buf.copy_from_slice(&buf[offset..offset + BLOCK_SIZE]);
            self.write_block(block_num, &block_buf)?;
        }
        
        Ok(count as usize * BLOCK_SIZE)
    }
    
    pub fn get_capacity_mb(&self) -> u64 {
        (self.total_blocks * BLOCK_SIZE as u64) / (1024 * 1024)
    }
    
    pub fn get_model(&self) -> &str {
        let len = self.info.model.iter().position(|&c| c == 0).unwrap_or(40);
        core::str::from_utf8(&self.info.model[..len]).unwrap_or("<invalid>")
    }
}

/// Global disk registry
pub const MAX_DISKS: usize = 8;

pub struct DiskRegistry {
    disks: [Option<BlockDevice>; MAX_DISKS],
    count: usize,
}

impl DiskRegistry {
    pub const fn new() -> Self {
        Self {
            disks: [None; MAX_DISKS],
            count: 0,
        }
    }
    
    pub fn register(&mut self, device: BlockDevice) -> Option<u32> {
        if self.count < MAX_DISKS {
            self.disks[self.count] = Some(device);
            let id = self.count as u32;
            self.count += 1;
            Some(id)
        } else {
            None
        }
    }
    
    pub fn get(&self, id: u32) -> Option<&BlockDevice> {
        if (id as usize) < self.count {
            self.disks[id as usize].as_ref()
        } else {
            None
        }
    }
    
    pub fn get_mut(&mut self, id: u32) -> Option<&mut BlockDevice> {
        if (id as usize) < self.count {
            self.disks[id as usize].as_mut()
        } else {
            None
        }
    }
    
    pub fn count(&self) -> usize {
        self.count
    }
}

static mut DISK_REGISTRY: DiskRegistry = DiskRegistry::new();

pub fn register_disk(device: BlockDevice) -> Option<u32> {
    unsafe { DISK_REGISTRY.register(device) }
}

pub fn get_disk(id: u32) -> Option<&'static BlockDevice> {
    unsafe { DISK_REGISTRY.get(id) }
}

pub fn get_disk_mut(id: u32) -> Option<&'static mut BlockDevice> {
    unsafe { DISK_REGISTRY.get_mut(id) }
}

pub fn disk_count() -> usize {
    unsafe { DISK_REGISTRY.count() }
}

