// servers/guardfs/src/ops.rs
// GuardFS File Operations
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::*;
use crate::inode::*;
use crate::bitmap::*;
use crate::journal::*;
use crate::dir::*;
use crate::compress::*;
use crate::snapshot::*;

pub struct GuardFs {
    pub superblock: GuardFsSuperblock,
    pub inodes: InodeTable,
    pub bitmap: BlockBitmap,
    pub journal: Journal,
    pub snapshots: SnapshotManager,
    
    // In-memory data blocks (simplified - production would use disk I/O)
    pub blocks: [[u8; BLOCK_SIZE as usize]; 8192], // 32MB RAM disk
}

impl GuardFs {
    pub fn new() -> Self {
        Self {
            superblock: GuardFsSuperblock::new(),
            inodes: InodeTable::new(),
            bitmap: BlockBitmap::new(),
            journal: Journal::new(),
            snapshots: SnapshotManager::new(),
            blocks: [[0; BLOCK_SIZE as usize]; 8192],
        }
    }
    
    pub fn format(&mut self, size_blocks: u64, label: &str) {
        // Initialize superblock
        self.superblock.init(size_blocks, label);
        
        // Initialize inode table (reserve root)
        self.inodes.init();
        
        // Initialize bitmap (reserve metadata)
        self.bitmap.init(298);
        
        // Create root directory
        let _ = self.mkdir_at(ROOT_INODE, "/", 0o755);
    }
    
    pub fn open(&mut self, path: &str, flags: u32) -> Result<u32, i32> {
        // Resolve path to inode
        if let Some(inode_num) = self.resolve_path(path) {
            // File exists - return inode number as "FD"
            return Ok(inode_num);
        }
        
        // File doesn't exist
        if flags & 0x200 != 0 { // O_CREAT
            return self.create_file(path, 0o644);
        }
        
        Err(-2) // ENOENT
    }
    
    pub fn create_file(&mut self, path: &str, mode: u16) -> Result<u32, i32> {
        let filename = get_filename(path).ok_or(-22)?; // EINVAL
        let parent_path = get_parent_path(path);
        
        // Resolve parent directory
        let parent_inode = self.resolve_path(parent_path).ok_or(-2)?; // ENOENT
        
        // Allocate inode
        let inode_num = self.inodes.allocate().ok_or(-28)?; // ENOSPC
        
        // Create inode
        let inode = GuardFsInode::new_file(0, 0, mode);
        self.inodes.inodes[inode_num as usize] = inode;
        
        // Add to parent directory
        self.add_dir_entry(parent_inode, inode_num, filename, S_IFREG as u8)?;
        
        // Journal the operation
        self.journal.log(JournalOp::InodeUpdate, inode_num, 0, None);
        
        Ok(inode_num)
    }
    
    pub fn read(&mut self, inode_num: u32, buf: &mut [u8], offset: u64) -> Result<usize, i32> {
        let inode = self.inodes.get(inode_num).ok_or(-2)?; // ENOENT
        
        if offset >= inode.size {
            return Ok(0); // EOF
        }
        
        let mut bytes_read = 0;
        let mut current_offset = offset;
        
        while bytes_read < buf.len() && current_offset < inode.size {
            let block_num = inode.get_block_for_offset(current_offset).ok_or(-5)?; // EIO
            let block_offset = (current_offset % BLOCK_SIZE as u64) as usize;
            let remaining_in_block = BLOCK_SIZE as usize - block_offset;
            let to_read = remaining_in_block.min(buf.len() - bytes_read);
            
            // Read from block
            let block_data = &self.blocks[block_num as usize];
            
            // Decompress if needed
            if inode.compressed != 0 {
                let mut decomp_buf = [0u8; BLOCK_SIZE as usize];
                decompress(block_data, &mut decomp_buf, inode.uncomp_size as usize).ok_or(-5)?;
                buf[bytes_read..bytes_read + to_read]
                    .copy_from_slice(&decomp_buf[block_offset..block_offset + to_read]);
            } else {
                buf[bytes_read..bytes_read + to_read]
                    .copy_from_slice(&block_data[block_offset..block_offset + to_read]);
            }
            
            bytes_read += to_read;
            current_offset += to_read as u64;
        }
        
        Ok(bytes_read)
    }
    
    pub fn write(&mut self, inode_num: u32, buf: &[u8], offset: u64) -> Result<usize, i32> {
        let inode = self.inodes.get_mut(inode_num).ok_or(-2)?; // ENOENT
        
        let mut bytes_written = 0;
        let mut current_offset = offset;
        
        while bytes_written < buf.len() {
            // Get or allocate block
            let block_num = if let Some(bn) = inode.get_block_for_offset(current_offset) {
                bn
            } else {
                // Allocate new extent
                let new_block = self.bitmap.allocate().ok_or(-28)?; // ENOSPC
                let extent = Extent {
                    start_block: new_block,
                    length: 1,
                    flags: 0,
                };
                inode.add_extent(extent);
                self.journal.log(JournalOp::ExtentAdd, inode_num, new_block, None);
                new_block
            };
            
            let block_offset = (current_offset % BLOCK_SIZE as u64) as usize;
            let remaining_in_block = BLOCK_SIZE as usize - block_offset;
            let to_write = remaining_in_block.min(buf.len() - bytes_written);
            
            // Write to block
            self.blocks[block_num as usize][block_offset..block_offset + to_write]
                .copy_from_slice(&buf[bytes_written..bytes_written + to_write]);
            
            // Try to compress if enabled
            if self.superblock.compression_enabled != 0 {
                let mut comp_buf = [0u8; MAX_COMPRESSED_SIZE];
                if let Some(comp_size) = compress(&self.blocks[block_num as usize], &mut comp_buf) {
                    // Compression successful
                    self.blocks[block_num as usize][..comp_size].copy_from_slice(&comp_buf[..comp_size]);
                    inode.compressed = 1;
                    inode.uncomp_size = BLOCK_SIZE;
                }
            }
            
            bytes_written += to_write;
            current_offset += to_write as u64;
        }
        
        // Update inode size
        if current_offset > inode.size {
            inode.size = current_offset;
        }
        
        // Update mtime
        inode.mtime = get_time();
        
        // Journal the write
        self.journal.log(JournalOp::InodeUpdate, inode_num, 0, None);
        
        Ok(bytes_written)
    }
    
    pub fn mkdir(&mut self, path: &str, mode: u16) -> Result<(), i32> {
        let dirname = get_filename(path).ok_or(-22)?;
        let parent_path = get_parent_path(path);
        
        let parent_inode = self.resolve_path(parent_path).ok_or(-2)?;
        
        self.mkdir_at(parent_inode, dirname, mode)
    }
    
    fn mkdir_at(&mut self, parent_inode: u32, name: &str, mode: u16) -> Result<(), i32> {
        // Allocate inode
        let inode_num = self.inodes.allocate().ok_or(-28)?;
        
        // Create directory inode
        let inode = GuardFsInode::new_dir(0, 0, mode);
        self.inodes.inodes[inode_num as usize] = inode;
        
        // Allocate block for directory data
        let block_num = self.bitmap.allocate().ok_or(-28)?;
        let extent = Extent {
            start_block: block_num,
            length: 1,
            flags: 0,
        };
        self.inodes.inodes[inode_num as usize].add_extent(extent);
        
        // Initialize directory with . and ..
        let mut dir = Directory::new();
        dir.entries[0] = DirEntry::new(inode_num, ".", S_IFDIR as u8);
        dir.entries[1] = DirEntry::new(parent_inode, "..", S_IFDIR as u8);
        
        // Write directory to block
        self.blocks[block_num as usize].copy_from_slice(dir.to_bytes());
        
        // Add to parent directory
        if name != "/" {
            self.add_dir_entry(parent_inode, inode_num, name, S_IFDIR as u8)?;
        }
        
        Ok(())
    }
    
    pub fn unlink(&mut self, path: &str) -> Result<(), i32> {
        let filename = get_filename(path).ok_or(-22)?;
        let parent_path = get_parent_path(path);
        
        let parent_inode = self.resolve_path(parent_path).ok_or(-2)?;
        let inode_num = self.resolve_path(path).ok_or(-2)?;
        
        // Remove from parent directory
        self.remove_dir_entry(parent_inode, filename)?;
        
        // Decrement link count
        if let Some(inode) = self.inodes.get_mut(inode_num) {
            inode.link_count -= 1;
            
            if inode.link_count == 0 {
                // Free all extents
                for extent in &inode.extents {
                    if !extent.is_empty() {
                        self.bitmap.free_extent(extent.start_block, extent.length);
                    }
                }
                
                // Free inode
                self.inodes.free(inode_num);
            }
        }
        
        Ok(())
    }
    
    fn resolve_path(&self, path: &str) -> Option<u32> {
        if path == "/" {
            return Some(ROOT_INODE);
        }
        
        let mut parts = [""; 16]; // Max path depth
        let part_count = split_path(path, &mut parts);
        let mut current_inode = ROOT_INODE;
        
        for i in 0..part_count {
            let part = parts[i];
            let inode = self.inodes.get(current_inode)?;
            if !inode.is_dir() {
                return None;
            }
            
            // Read directory
            let block_num = inode.extents[0].start_block;
            let dir_data = &self.blocks[block_num as usize];
            let dir = Directory::from_bytes(dir_data);
            
            // Find entry
            let entry = dir.find_entry(part)?;
            current_inode = entry.inode;
        }
        
        Some(current_inode)
    }
    
    fn add_dir_entry(&mut self, dir_inode: u32, child_inode: u32, name: &str, file_type: u8) -> Result<(), i32> {
        let inode = self.inodes.get(dir_inode).ok_or(-2)?;
        let block_num = inode.extents[0].start_block;
        
        let mut dir = Directory::from_bytes(&self.blocks[block_num as usize]);
        let entry = DirEntry::new(child_inode, name, file_type);
        
        if !dir.add_entry(entry) {
            return Err(-28); // ENOSPC
        }
        
        self.blocks[block_num as usize].copy_from_slice(dir.to_bytes());
        Ok(())
    }
    
    fn remove_dir_entry(&mut self, dir_inode: u32, name: &str) -> Result<(), i32> {
        let inode = self.inodes.get(dir_inode).ok_or(-2)?;
        let block_num = inode.extents[0].start_block;
        
        let mut dir = Directory::from_bytes(&self.blocks[block_num as usize]);
        
        if !dir.remove_entry(name) {
            return Err(-2); // ENOENT
        }
        
        self.blocks[block_num as usize].copy_from_slice(dir.to_bytes());
        Ok(())
    }
    
    // Snapshot operations
    pub fn create_snapshot(&mut self, name: &str) -> Result<u32, i32> {
        let snap_id = self.snapshots.create(name, ROOT_INODE).ok_or(-28)?;
        
        // Increment snapshot generation for all inodes
        for i in 0..DEFAULT_INODES as usize {
            self.inodes.inodes[i].snapshot_gen += 1;
        }
        
        self.superblock.snapshot_count += 1;
        
        Ok(snap_id)
    }
    
    pub fn delete_snapshot(&mut self, id: u32) -> Result<(), i32> {
        if self.snapshots.delete(id) {
            self.superblock.snapshot_count -= 1;
            Ok(())
        } else {
            Err(-2) // ENOENT
        }
    }
}

fn get_time() -> u64 {
    static mut TIME: u64 = 1700000000;
    unsafe {
        TIME += 1;
        TIME
    }
}

