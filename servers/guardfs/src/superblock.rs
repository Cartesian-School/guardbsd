// servers/guardfs/src/superblock.rs
// GuardFS Superblock Management
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause


pub const GUARDFS_MAGIC: &[u8; 8] = b"GUARDFS\0";
pub const GUARDFS_VERSION: u32 = 1;
pub const BLOCK_SIZE: u32 = 4096;
pub const INODE_SIZE: u16 = 256;
pub const DEFAULT_INODES: u32 = 1024;
pub const DEFAULT_JOURNAL_BLOCKS: u32 = 8;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct GuardFsSuperblock {
    // Magic and version
    pub magic: [u8; 8],
    pub version: u32,
    pub flags: u32,
    
    // Block information
    pub block_size: u32,
    pub total_blocks: u64,
    pub free_blocks: u64,
    
    // Inode information
    pub total_inodes: u32,
    pub free_inodes: u32,
    pub inode_size: u16,
    pub inodes_per_block: u16,
    
    // Journal
    pub journal_start: u64,
    pub journal_size: u32,
    pub journal_seq: u64,
    
    // Snapshots
    pub snapshot_count: u16,
    pub snapshot_start: u64,
    
    // Compression
    pub compression_enabled: u8,
    pub compression_algo: u8,
    pub compression_level: u8,
    
    // Checksums
    pub checksum_algo: u8,
    
    // Filesystem metadata
    pub root_inode: u32,
    pub mount_time: u64,
    pub unmount_time: u64,
    pub mount_count: u32,
    pub max_mount_count: u32,
    pub state: u16,
    pub errors: u16,
    
    // UUID and label
    pub uuid: [u8; 16],
    pub label: [u8; 64],
    
    // Reserved
    pub reserved: [u8; 3712],
    
    // Checksum
    pub sb_checksum: [u8; 32],
}

impl GuardFsSuperblock {
    pub const fn new() -> Self {
        Self {
            magic: *GUARDFS_MAGIC,
            version: GUARDFS_VERSION,
            flags: 0,
            
            block_size: BLOCK_SIZE,
            total_blocks: 0,
            free_blocks: 0,
            
            total_inodes: DEFAULT_INODES,
            free_inodes: DEFAULT_INODES,
            inode_size: INODE_SIZE,
            inodes_per_block: (BLOCK_SIZE as u16) / INODE_SIZE,
            
            journal_start: 2,
            journal_size: DEFAULT_JOURNAL_BLOCKS,
            journal_seq: 0,
            
            snapshot_count: 0,
            snapshot_start: 0,
            
            compression_enabled: 1,
            compression_algo: 1, // LZ4
            compression_level: 3,
            
            checksum_algo: 1, // CRC32
            
            root_inode: 2,
            mount_time: 0,
            unmount_time: 0,
            mount_count: 0,
            max_mount_count: 20,
            state: 1, // Clean
            errors: 1, // Continue on error
            
            uuid: [0; 16],
            label: [0; 64],
            
            reserved: [0; 3712],
            sb_checksum: [0; 32],
        }
    }
    
    pub fn init(&mut self, total_blocks: u64, label: &str) {
        self.total_blocks = total_blocks;
        self.free_blocks = total_blocks - 298; // Reserve metadata blocks
        
        // Set label
        let label_bytes = label.as_bytes();
        let copy_len = label_bytes.len().min(63);
        self.label[..copy_len].copy_from_slice(&label_bytes[..copy_len]);
        
        // Generate simple UUID (timestamp-based)
        self.generate_uuid();
        
        // Calculate checksum
        self.update_checksum();
    }
    
    pub fn validate(&self) -> bool {
        // Check magic
        if &self.magic != GUARDFS_MAGIC {
            return false;
        }
        
        // Check version
        if self.version != GUARDFS_VERSION {
            return false;
        }
        
        // Check block size
        if self.block_size != BLOCK_SIZE {
            return false;
        }
        
        // Verify checksum
        self.verify_checksum()
    }
    
    fn generate_uuid(&mut self) {
        // Simple UUID generation (timestamp + counter)
        let time = self.mount_count as u64;
        self.uuid[0..8].copy_from_slice(&time.to_le_bytes());
        self.uuid[8..16].copy_from_slice(&self.total_blocks.to_le_bytes());
    }
    
    fn update_checksum(&mut self) {
        // Calculate CRC32 of superblock (excluding checksum field)
        let checksum = self.calculate_crc32();
        self.sb_checksum[0..4].copy_from_slice(&checksum.to_le_bytes());
    }
    
    fn verify_checksum(&self) -> bool {
        let stored_checksum = u32::from_le_bytes([
            self.sb_checksum[0],
            self.sb_checksum[1],
            self.sb_checksum[2],
            self.sb_checksum[3]
        ]);
        
        let calculated_checksum = self.calculate_crc32();
        stored_checksum == calculated_checksum
    }
    
    fn calculate_crc32(&self) -> u32 {
        // Simple CRC32 implementation
        let data = unsafe {
            core::slice::from_raw_parts(
                self as *const _ as *const u8,
                core::mem::size_of::<Self>() - 32 // Exclude checksum field
            )
        };
        
        crc32(data)
    }
}

// Simple CRC32 implementation
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_superblock_validation() {
        let mut sb = GuardFsSuperblock::new();
        sb.init(10000, "test_fs");
        assert!(sb.validate());
    }
    
    #[test]
    fn test_crc32() {
        let data = b"Hello, World!";
        let checksum = crc32(data);
        assert!(checksum != 0);
    }
}

