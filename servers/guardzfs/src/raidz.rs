// servers/guardzfs/src/raidz.rs
// GuardZFS RAID-Z Implementation
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::vdev::*;
use crate::blockptr::*;

pub const BLOCK_SIZE: usize = 4096;

/// Calculate RAID-Z1 parity (single parity disk)
pub fn raidz1_calculate_parity(data_blocks: &[&[u8]]) -> [u8; BLOCK_SIZE] {
    let mut parity = [0u8; BLOCK_SIZE];
    
    for block in data_blocks {
        for i in 0..BLOCK_SIZE.min(block.len()) {
            parity[i] ^= block[i];
        }
    }
    
    parity
}

/// Calculate RAID-Z2 parity (double parity)
/// Uses P+Q parity like RAID-6
pub fn raidz2_calculate_parity(data_blocks: &[&[u8]]) -> ([u8; BLOCK_SIZE], [u8; BLOCK_SIZE]) {
    let mut p_parity = [0u8; BLOCK_SIZE];
    let mut q_parity = [0u8; BLOCK_SIZE];
    
    // P parity (XOR)
    for block in data_blocks {
        for i in 0..BLOCK_SIZE.min(block.len()) {
            p_parity[i] ^= block[i];
        }
    }
    
    // Q parity (Galois field multiplication)
    for (disk_idx, block) in data_blocks.iter().enumerate() {
        let multiplier = gf_pow(2, disk_idx);
        for i in 0..BLOCK_SIZE.min(block.len()) {
            q_parity[i] ^= gf_mul(block[i], multiplier);
        }
    }
    
    (p_parity, q_parity)
}

/// Reconstruct missing data block from RAID-Z1
pub fn raidz1_reconstruct(blocks: &[Option<&[u8]>], missing_idx: usize) -> [u8; BLOCK_SIZE] {
    let mut reconstructed = [0u8; BLOCK_SIZE];
    
    // XOR all available blocks
    for (idx, block_opt) in blocks.iter().enumerate() {
        if idx != missing_idx {
            if let Some(block) = block_opt {
                for i in 0..BLOCK_SIZE.min(block.len()) {
                    reconstructed[i] ^= block[i];
                }
            }
        }
    }
    
    reconstructed
}

/// Reconstruct missing blocks from RAID-Z2 (can recover 2 failures)
pub fn raidz2_reconstruct(
    blocks: &[Option<&[u8]>],
    missing_indices: &[usize]
) -> Result<([u8; BLOCK_SIZE], [u8; BLOCK_SIZE]), &'static str> {
    if missing_indices.len() > 2 {
        return Err("Too many failures");
    }
    
    if missing_indices.is_empty() {
        return Err("No failures");
    }
    
    if missing_indices.len() == 1 {
        // Single failure - use P parity (XOR)
        let reconstructed = raidz1_reconstruct(blocks, missing_indices[0]);
        return Ok((reconstructed, [0; BLOCK_SIZE]));
    }
    
    // Double failure - use P and Q parity
    // TODO: Implement full Reed-Solomon decoding for 2 failures
    Err("Double reconstruction not fully implemented")
}

/// Galois Field (2^8) multiplication for RAID-6 Q parity
fn gf_mul(a: u8, b: u8) -> u8 {
    let mut result = 0u8;
    let mut a_val = a;
    let mut b_val = b;
    
    for _ in 0..8 {
        if b_val & 1 != 0 {
            result ^= a_val;
        }
        
        let high_bit = a_val & 0x80;
        a_val <<= 1;
        
        if high_bit != 0 {
            a_val ^= 0x1D; // GF(2^8) polynomial
        }
        
        b_val >>= 1;
    }
    
    result
}

fn gf_pow(base: u8, exp: usize) -> u8 {
    let mut result = 1u8;
    for _ in 0..exp {
        result = gf_mul(result, base);
    }
    result
}

/// RAID-Z write operation
pub fn raidz_write(
    vdev: &VirtualDevice,
    offset: u64,
    data: &[u8],
    write_fn: &mut dyn FnMut(u32, u64, &[u8]) -> Result<(), ()>
) -> Result<(), &'static str> {
    if data.len() != BLOCK_SIZE {
        return Err("Invalid block size");
    }
    
    match vdev.vdev_type {
        VdevType::Single => {
            // Simple write to single disk
            write_fn(vdev.children[0], offset, data).map_err(|_| "Write failed")?;
            Ok(())
        }
        
        VdevType::Mirror => {
            // Write to all mirror copies
            for i in 0..vdev.child_count as usize {
                write_fn(vdev.children[i], offset, data).map_err(|_| "Write failed")?;
            }
            Ok(())
        }
        
        VdevType::RaidZ1 => {
            // Split data into chunks
            let chunk_size = BLOCK_SIZE / (vdev.stripe_width as usize);
            let mut chunks = [[0u8; BLOCK_SIZE]; MAX_VDEV_CHILDREN];
            
            // Distribute data across data disks
            for i in 0..vdev.stripe_width as usize {
                let start = i * chunk_size;
                let end = start + chunk_size;
                chunks[i][..chunk_size].copy_from_slice(&data[start..end]);
            }
            
            // Calculate parity
            let mut chunk_ptrs = [&[][..]; MAX_VDEV_CHILDREN];
            for i in 0..vdev.stripe_width as usize {
                chunk_ptrs[i] = &chunks[i];
            }
            let parity = raidz1_calculate_parity(&chunk_ptrs[..vdev.stripe_width as usize]);
            
            // Write data chunks
            for i in 0..vdev.stripe_width as usize {
                write_fn(vdev.children[i], offset, &chunks[i]).map_err(|_| "Write failed")?;
            }
            
            // Write parity
            write_fn(vdev.children[vdev.stripe_width as usize], offset, &parity)
                .map_err(|_| "Parity write failed")?;
            
            Ok(())
        }
        
        VdevType::RaidZ2 => {
            // Similar to RAID-Z1 but with 2 parity disks
            let chunk_size = BLOCK_SIZE / (vdev.stripe_width as usize);
            let mut chunks = [[0u8; BLOCK_SIZE]; MAX_VDEV_CHILDREN];
            
            for i in 0..vdev.stripe_width as usize {
                let start = i * chunk_size;
                let end = start + chunk_size;
                chunks[i][..chunk_size].copy_from_slice(&data[start..end]);
            }
            
            let mut chunk_ptrs = [&[][..]; MAX_VDEV_CHILDREN];
            for i in 0..vdev.stripe_width as usize {
                chunk_ptrs[i] = &chunks[i];
            }
            let (p_parity, q_parity) = raidz2_calculate_parity(&chunk_ptrs[..vdev.stripe_width as usize]);
            
            // Write data chunks
            for i in 0..vdev.stripe_width as usize {
                write_fn(vdev.children[i], offset, &chunks[i]).map_err(|_| "Write failed")?;
            }
            
            // Write P and Q parity
            write_fn(vdev.children[vdev.stripe_width as usize], offset, &p_parity)
                .map_err(|_| "P parity failed")?;
            write_fn(vdev.children[vdev.stripe_width as usize + 1], offset, &q_parity)
                .map_err(|_| "Q parity failed")?;
            
            Ok(())
        }
    }
}

/// RAID-Z read operation with self-healing
pub fn raidz_read(
    vdev: &VirtualDevice,
    offset: u64,
    buf: &mut [u8],
    read_fn: &mut dyn FnMut(u32, u64, &mut [u8]) -> Result<(), ()>
) -> Result<(), &'static str> {
    match vdev.vdev_type {
        VdevType::Single => {
            read_fn(vdev.children[0], offset, buf).map_err(|_| "Read failed")?;
            Ok(())
        }
        
        VdevType::Mirror => {
            // Try first mirror
            if read_fn(vdev.children[0], offset, buf).is_ok() {
                return Ok(());
            }
            
            // Try second mirror
            if vdev.child_count > 1 {
                read_fn(vdev.children[1], offset, buf).map_err(|_| "All mirrors failed")?;
                
                // TODO: Repair first mirror (self-healing)
                Ok(())
            } else {
                Err("Mirror read failed")
            }
        }
        
        VdevType::RaidZ1 | VdevType::RaidZ2 => {
            // Read all data chunks
            let chunk_size = BLOCK_SIZE / (vdev.stripe_width as usize);
            let mut chunks = [[0u8; BLOCK_SIZE]; MAX_VDEV_CHILDREN];
            let mut failed_disk = None;
            
            // Try to read data disks
            for i in 0..vdev.stripe_width as usize {
                if read_fn(vdev.children[i], offset, &mut chunks[i]).is_err() {
                    failed_disk = Some(i);
                }
            }
            
            if let Some(failed_idx) = failed_disk {
                // Read parity and reconstruct
                read_fn(
                    vdev.children[vdev.stripe_width as usize],
                    offset,
                    &mut chunks[vdev.stripe_width as usize]
                ).map_err(|_| "Parity read failed")?;
                
                // Build block references array
                let mut block_refs = [None; 16];
                for i in 0..=vdev.stripe_width as usize {
                    if i != failed_idx {
                        block_refs[i] = Some(&chunks[i] as &[u8]);
                    }
                }
                
                chunks[failed_idx] = raidz1_reconstruct(&block_refs[..=vdev.stripe_width as usize], failed_idx);
                
                // TODO: Repair failed disk (self-healing)
            }
            
            // Reassemble data
            for i in 0..vdev.stripe_width as usize {
                let start = i * chunk_size;
                buf[start..start + chunk_size].copy_from_slice(&chunks[i][..chunk_size]);
            }
            
            Ok(())
        }
    }
}


