// servers/guardzfs/src/ops.rs
// GuardZFS File Operations
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::blockptr::*;
use crate::dmu::*;
use crate::pool::*;
use crate::raidz::*;
use crate::zap::*;
use crate::*;

pub struct GuardZfs {
    pub pool_id: usize,
    pub objdir: ObjectDirectory,
    pub root_object: u64,
}

impl GuardZfs {
    pub const fn new() -> Self {
        Self {
            pool_id: 0,
            objdir: ObjectDirectory::new(),
            root_object: 1,
        }
    }

    pub fn create(pool_id: usize) -> Self {
        let mut zfs = Self::new();
        zfs.pool_id = pool_id;

        // Create root directory object
        if let Some(root_id) = zfs.objdir.allocate_object(ObjectType::Directory) {
            zfs.root_object = root_id;
        }

        zfs
    }

    pub fn open(&mut self, path: &str, _flags: u32) -> Result<u64, i32> {
        // Resolve path to object ID
        self.resolve_path(path).ok_or(-2) // ENOENT
    }

    pub fn create_file(&mut self, path: &str) -> Result<u64, i32> {
        let (parent_path, filename) = split_path_parent(path);

        // Find parent directory
        let parent_id = self.resolve_path(parent_path).ok_or(-2)?;

        // Allocate new object
        let object_id = self.objdir.allocate_object(ObjectType::File).ok_or(-28)?; // ENOSPC

        // Add to parent directory
        self.add_dir_entry(parent_id, filename, object_id)?;

        Ok(object_id)
    }

    pub fn read(&mut self, object_id: u64, buf: &mut [u8], offset: u64) -> Result<usize, i32> {
        let obj = self.objdir.get(object_id).ok_or(-2)?; // ENOENT

        let block_idx = offset / 4096;
        let block_offset = (offset % 4096) as usize;

        // Get block pointer
        if let Some(bp) = obj.get_block_pointer(block_idx) {
            // Read from pool
            let pool = get_pool_mut(self.pool_id).ok_or(-5)?; // EIO
            let vdev = &pool.vdevs[bp.dva[0].vdev_id as usize];

            let mut block_buf = [0u8; 4096];

            // Read via RAID-Z if applicable
            let result = raidz_read(
                vdev,
                bp.dva[0].offset,
                &mut block_buf,
                &mut |_disk_id, _offset, _buf| Err(()), // Stub
            );

            if result.is_ok() {
                // Verify checksum
                if !bp.verify(&block_buf) {
                    return Err(-5); // EIO - checksum mismatch
                }

                // Copy to user buffer
                let to_copy = buf.len().min(4096 - block_offset);
                buf[..to_copy].copy_from_slice(&block_buf[block_offset..block_offset + to_copy]);

                pool.read_ops += 1;
                pool.bytes_read += to_copy as u64;

                return Ok(to_copy);
            }
        }

        Err(-5) // EIO
    }

    pub fn write(&mut self, object_id: u64, buf: &[u8], _offset: u64) -> Result<usize, i32> {
        let obj = self.objdir.get_mut(object_id).ok_or(-2)?;

        let pool = get_pool_mut(self.pool_id).ok_or(-5)?;

        // Allocate block in pool
        let (vdev_id, block_num) = pool.allocate_block().ok_or(-28)?; // ENOSPC

        // Create block pointer with checksum
        let mut block_buf = [0u8; 4096];
        let write_len = buf.len().min(4096);
        block_buf[..write_len].copy_from_slice(&buf[..write_len]);

        let bp = BlockPointer::new(vdev_id as u32, block_num, &block_buf);

        // Add to object
        if !obj.add_block_pointer(bp) {
            return Err(-28); // ENOSPC - need indirect blocks
        }

        // Write via RAID-Z
        let vdev = &pool.vdevs[vdev_id as usize];
        let _result = raidz_write(
            vdev,
            block_num,
            &block_buf,
            &mut |_disk_id, _offset, _data| Err(()), // Stub
        );

        pool.write_ops += 1;
        pool.bytes_written += write_len as u64;

        Ok(write_len)
    }

    pub fn mkdir(&mut self, path: &str) -> Result<(), i32> {
        let (parent_path, dirname) = split_path_parent(path);

        let parent_id = self.resolve_path(parent_path).ok_or(-2)?;
        let object_id = self
            .objdir
            .allocate_object(ObjectType::Directory)
            .ok_or(-28)?;

        self.add_dir_entry(parent_id, dirname, object_id)?;

        Ok(())
    }

    fn resolve_path(&self, path: &str) -> Option<u64> {
        if path == "/" {
            return Some(self.root_object);
        }

        // TODO: Implement full path resolution via ZAP lookups
        None
    }

    fn add_dir_entry(&mut self, _parent_id: u64, _name: &str, _child_id: u64) -> Result<(), i32> {
        // TODO: Read parent ZAP directory, add entry, write back
        Ok(())
    }
}

fn split_path_parent(path: &str) -> (&str, &str) {
    if let Some(pos) = path.rfind('/') {
        if pos == 0 {
            ("/", &path[1..])
        } else {
            (&path[..pos], &path[pos + 1..])
        }
    } else {
        ("/", path)
    }
}
