// servers/guardzfs/src/blockptr.rs
// GuardZFS Block Pointers
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::checksum::sha256;

pub const BP_SIZE: usize = 128;

/// Device Address (DVA) - where data is physically stored
#[repr(C)]
#[derive(Copy, Clone)]
pub struct DeviceAddress {
    pub vdev_id: u32,         // Which virtual device
    pub offset: u64,          // Block offset on that device
    pub asize: u32,           // Allocated size
}

impl DeviceAddress {
    pub const fn empty() -> Self {
        Self {
            vdev_id: 0,
            offset: 0,
            asize: 0,
        }
    }
    
    pub fn is_valid(&self) -> bool {
        self.offset != 0
    }
}

/// Block Pointer - The heart of ZFS
/// Contains location, checksum, compression info for every block
#[repr(C)]
#[derive(Copy, Clone)]
pub struct BlockPointer {
    // Physical locations (up to 3 copies for ditto blocks)
    pub dva: [DeviceAddress; 3],
    
    // Sizes
    pub lsize: u32,           // Logical size (uncompressed)
    pub psize: u32,           // Physical size (compressed)
    
    // Checksum (SHA-256)
    pub checksum: [u8; 32],
    pub checksum_type: u8,    // 0 = none, 1 = SHA256
    
    // Compression
    pub comp_algo: u8,        // 0 = none, 1 = LZ4
    
    // Metadata
    pub level: u8,            // Indirection level (0 = data, 1+ = indirect)
    pub object_type: u8,      // File, directory, etc.
    
    // Transaction info
    pub birth_txg: u64,       // Transaction group when created
    pub fill_count: u64,      // Number of non-zero blocks below
    
    // Padding to 128 bytes
    pub padding: [u8; 16],
}

impl BlockPointer {
    pub const fn empty() -> Self {
        Self {
            dva: [DeviceAddress::empty(); 3],
            lsize: 0,
            psize: 0,
            checksum: [0; 32],
            checksum_type: 0,
            comp_algo: 0,
            level: 0,
            object_type: 0,
            birth_txg: 0,
            fill_count: 0,
            padding: [0; 16],
        }
    }
    
    pub fn new(vdev_id: u32, offset: u64, data: &[u8]) -> Self {
        let mut bp = Self::empty();
        
        bp.dva[0] = DeviceAddress {
            vdev_id,
            offset,
            asize: data.len() as u32,
        };
        
        bp.lsize = data.len() as u32;
        bp.psize = data.len() as u32;
        bp.checksum_type = 1; // SHA-256
        bp.checksum = sha256(data);
        bp.birth_txg = 1; // TODO: Get current TXG
        
        bp
    }
    
    pub fn is_valid(&self) -> bool {
        self.dva[0].is_valid()
    }
    
    pub fn verify(&self, data: &[u8]) -> bool {
        if self.checksum_type == 0 {
            return true; // No checksum
        }
        
        let computed = sha256(data);
        computed == self.checksum
    }
    
    pub fn is_hole(&self) -> bool {
        self.lsize == 0
    }
}

/// Indirect Block - contains array of block pointers
pub struct IndirectBlock {
    pub pointers: [BlockPointer; 32], // 32 * 128 = 4KB
}

impl IndirectBlock {
    pub const fn new() -> Self {
        Self {
            pointers: [BlockPointer::empty(); 32],
        }
    }
    
    pub fn to_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const _ as *const u8,
                core::mem::size_of::<Self>()
            )
        }
    }
    
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut ib = Self::new();
        if data.len() >= core::mem::size_of::<Self>() {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    data.as_ptr(),
                    &mut ib as *mut _ as *mut u8,
                    core::mem::size_of::<Self>()
                );
            }
        }
        ib
    }
}

