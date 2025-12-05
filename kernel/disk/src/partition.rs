// kernel/disk/src/partition.rs
// Partition Table Support (MBR/GPT)
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#![no_std]

use crate::block_device::*;

// MBR Constants
pub const MBR_SIGNATURE: u16 = 0xAA55;
pub const MBR_PARTITION_TABLE_OFFSET: usize = 0x1BE;
pub const MBR_SIGNATURE_OFFSET: usize = 0x1FE;

// Partition types (MBR)
pub const PART_TYPE_EMPTY: u8 = 0x00;
pub const PART_TYPE_FAT12: u8 = 0x01;
pub const PART_TYPE_FAT16_SMALL: u8 = 0x04;
pub const PART_TYPE_EXTENDED: u8 = 0x05;
pub const PART_TYPE_FAT16: u8 = 0x06;
pub const PART_TYPE_NTFS: u8 = 0x07;
pub const PART_TYPE_EXFAT: u8 = 0x07; // Same as NTFS
pub const PART_TYPE_FAT32: u8 = 0x0B;
pub const PART_TYPE_FAT32_LBA: u8 = 0x0C;
pub const PART_TYPE_FAT16_LBA: u8 = 0x0E;
pub const PART_TYPE_EXTENDED_LBA: u8 = 0x0F;
pub const PART_TYPE_LINUX_SWAP: u8 = 0x82;  // *** SWAP PARTITION ***
pub const PART_TYPE_LINUX: u8 = 0x83;
pub const PART_TYPE_LINUX_EXTENDED: u8 = 0x85;
pub const PART_TYPE_LVM: u8 = 0x8E;
pub const PART_TYPE_FREEBSD: u8 = 0xA5;
pub const PART_TYPE_OPENBSD: u8 = 0xA6;
pub const PART_TYPE_NETBSD: u8 = 0xA9;
pub const PART_TYPE_APPLE_HFS: u8 = 0xAF;
pub const PART_TYPE_SOLARIS: u8 = 0xBF;
pub const PART_TYPE_GPT: u8 = 0xEE;
pub const PART_TYPE_EFI_SYSTEM: u8 = 0xEF;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MbrPartition {
    pub bootable: u8,
    pub start_chs: [u8; 3],
    pub partition_type: u8,
    pub end_chs: [u8; 3],
    pub start_lba: u32,
    pub size_sectors: u32,
}

impl MbrPartition {
    pub const fn empty() -> Self {
        Self {
            bootable: 0,
            start_chs: [0; 3],
            partition_type: 0,
            end_chs: [0; 3],
            start_lba: 0,
            size_sectors: 0,
        }
    }
    
    pub fn is_valid(&self) -> bool {
        self.partition_type != PART_TYPE_EMPTY && self.size_sectors > 0
    }
    
    pub fn is_bootable(&self) -> bool {
        self.bootable == 0x80
    }
    
    pub fn is_swap(&self) -> bool {
        self.partition_type == PART_TYPE_LINUX_SWAP
    }
    
    pub fn is_extended(&self) -> bool {
        self.partition_type == PART_TYPE_EXTENDED 
            || self.partition_type == PART_TYPE_EXTENDED_LBA
            || self.partition_type == PART_TYPE_LINUX_EXTENDED
    }
    
    pub fn get_type_name(&self) -> &'static str {
        match self.partition_type {
            PART_TYPE_EMPTY => "Empty",
            PART_TYPE_FAT12 => "FAT12",
            PART_TYPE_FAT16_SMALL => "FAT16 (small)",
            PART_TYPE_EXTENDED => "Extended",
            PART_TYPE_FAT16 => "FAT16",
            PART_TYPE_NTFS => "NTFS/exFAT",
            PART_TYPE_FAT32 => "FAT32",
            PART_TYPE_FAT32_LBA => "FAT32 (LBA)",
            PART_TYPE_FAT16_LBA => "FAT16 (LBA)",
            PART_TYPE_EXTENDED_LBA => "Extended (LBA)",
            PART_TYPE_LINUX_SWAP => "Linux Swap",
            PART_TYPE_LINUX => "Linux",
            PART_TYPE_LINUX_EXTENDED => "Linux Extended",
            PART_TYPE_LVM => "Linux LVM",
            PART_TYPE_FREEBSD => "FreeBSD",
            PART_TYPE_OPENBSD => "OpenBSD",
            PART_TYPE_NETBSD => "NetBSD",
            PART_TYPE_APPLE_HFS => "Apple HFS/HFS+",
            PART_TYPE_SOLARIS => "Solaris",
            PART_TYPE_GPT => "GPT Protective",
            PART_TYPE_EFI_SYSTEM => "EFI System",
            _ => "Unknown",
        }
    }
    
    pub fn get_size_mb(&self) -> u64 {
        (self.size_sectors as u64 * 512) / (1024 * 1024)
    }
}

pub struct Mbr {
    pub bootstrap: [u8; 446],
    pub partitions: [MbrPartition; 4],
    pub signature: u16,
}

impl Mbr {
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 512 {
            return None;
        }
        
        // Check signature
        let signature = u16::from_le_bytes([data[510], data[511]]);
        if signature != MBR_SIGNATURE {
            return None;
        }
        
        let mut mbr = Mbr {
            bootstrap: [0; 446],
            partitions: [MbrPartition::empty(); 4],
            signature,
        };
        
        // Copy bootstrap code
        mbr.bootstrap.copy_from_slice(&data[0..446]);
        
        // Parse partition table
        for i in 0..4 {
            let offset = MBR_PARTITION_TABLE_OFFSET + i * 16;
            mbr.partitions[i] = MbrPartition {
                bootable: data[offset],
                start_chs: [data[offset + 1], data[offset + 2], data[offset + 3]],
                partition_type: data[offset + 4],
                end_chs: [data[offset + 5], data[offset + 6], data[offset + 7]],
                start_lba: u32::from_le_bytes([
                    data[offset + 8],
                    data[offset + 9],
                    data[offset + 10],
                    data[offset + 11],
                ]),
                size_sectors: u32::from_le_bytes([
                    data[offset + 12],
                    data[offset + 13],
                    data[offset + 14],
                    data[offset + 15],
                ]),
            };
        }
        
        Some(mbr)
    }
    
    pub fn is_gpt_protective(&self) -> bool {
        self.partitions[0].partition_type == PART_TYPE_GPT
    }
}

// GPT Constants
pub const GPT_SIGNATURE: &[u8; 8] = b"EFI PART";

#[repr(C)]
#[derive(Copy, Clone)]
pub struct GptHeader {
    pub signature: [u8; 8],
    pub revision: u32,
    pub header_size: u32,
    pub header_crc32: u32,
    pub reserved: u32,
    pub current_lba: u64,
    pub backup_lba: u64,
    pub first_usable_lba: u64,
    pub last_usable_lba: u64,
    pub disk_guid: [u8; 16],
    pub partition_entry_lba: u64,
    pub num_partitions: u32,
    pub partition_entry_size: u32,
    pub partition_array_crc32: u32,
}

impl GptHeader {
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 92 {
            return None;
        }
        
        let mut header = GptHeader {
            signature: [0; 8],
            revision: 0,
            header_size: 0,
            header_crc32: 0,
            reserved: 0,
            current_lba: 0,
            backup_lba: 0,
            first_usable_lba: 0,
            last_usable_lba: 0,
            disk_guid: [0; 16],
            partition_entry_lba: 0,
            num_partitions: 0,
            partition_entry_size: 0,
            partition_array_crc32: 0,
        };
        
        header.signature.copy_from_slice(&data[0..8]);
        
        if &header.signature != GPT_SIGNATURE {
            return None;
        }
        
        header.revision = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        header.header_size = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        // Parse remaining fields...
        
        Some(header)
    }
}

pub fn detect_partitions(_device: &BlockDevice) -> Result<usize, DiskError> {
    let sector = [0u8; SECTOR_SIZE];
    
    // Read MBR (LBA 0)
    // In real implementation, would call device.read_sectors(0, 1, &mut sector)
    // For now, return placeholder
    
    if let Some(mbr) = Mbr::from_bytes(&sector) {
        if mbr.is_gpt_protective() {
            // Read GPT header (LBA 1)
            // Parse GPT partitions
            return Ok(0); // GPT partitions found
        } else {
            // MBR partitions
            let mut count = 0;
            for part in &mbr.partitions {
                if part.is_valid() {
                    count += 1;
                    
                    // Check if this is a swap partition
                    if part.is_swap() {
                        // Register as swap space
                        let swap = crate::swap::SwapSpace::new(
                            _device.id,
                            part.start_lba as u64,
                            part.size_sectors as u64
                        );
                        let _ = crate::swap::register_swap(swap);
                    }
                    
                    // Create logical block device for this partition
                    // Register partition device
                }
            }
            return Ok(count);
        }
    }
    
    Err(DiskError::ReadError)
}

