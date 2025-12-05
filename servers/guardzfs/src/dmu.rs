// servers/guardzfs/src/dmu.rs
// GuardZFS Data Management Unit
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::blockptr::*;

pub const DNODE_SIZE: usize = 512;
pub const MAX_DNODES: usize = 8192;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq)]
pub enum ObjectType {
    None = 0,
    File = 1,
    Directory = 2,
    Symlink = 3,
    ZapDirectory = 4, // ZFS Attribute Processor
}

/// DMU Object (dnode) - represents a file, directory, etc.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct DmuObject {
    pub object_id: u64,
    pub object_type: ObjectType,
    pub bonustype: u8,
    pub bonuslen: u16,

    // Size
    pub datablksz: u32, // Data block size
    pub dnodesize: u16, // This dnode's size

    // Block pointers
    pub blkptr: [BlockPointer; 3], // Direct blocks
    pub indirect: BlockPointer,    // Indirect block

    // Statistics
    pub maxblkid: u64,   // Maximum block ID
    pub used_bytes: u64, // Bytes used

    // Bonus buffer (for small data)
    pub bonus: [u8; 128],

    // Padding
    pub padding: [u8; 64],
}

impl DmuObject {
    pub const fn empty() -> Self {
        Self {
            object_id: 0,
            object_type: ObjectType::None,
            bonustype: 0,
            bonuslen: 0,
            datablksz: 4096,
            dnodesize: DNODE_SIZE as u16,
            blkptr: [BlockPointer::empty(); 3],
            indirect: BlockPointer::empty(),
            maxblkid: 0,
            used_bytes: 0,
            bonus: [0; 128],
            padding: [0; 64],
        }
    }

    pub fn new_file(object_id: u64) -> Self {
        let mut obj = Self::empty();
        obj.object_id = object_id;
        obj.object_type = ObjectType::File;
        obj
    }

    pub fn new_directory(object_id: u64) -> Self {
        let mut obj = Self::empty();
        obj.object_id = object_id;
        obj.object_type = ObjectType::Directory;
        obj
    }

    pub fn add_block_pointer(&mut self, bp: BlockPointer) -> bool {
        for i in 0..3 {
            if !self.blkptr[i].is_valid() {
                self.blkptr[i] = bp;
                self.used_bytes += bp.lsize as u64;
                return true;
            }
        }
        false // All direct pointers used, need indirect
    }

    pub fn get_block_pointer(&self, block_idx: u64) -> Option<&BlockPointer> {
        if block_idx < 3 {
            let bp = &self.blkptr[block_idx as usize];
            if bp.is_valid() {
                return Some(bp);
            }
        }
        None
    }
}

pub struct ObjectDirectory {
    objects: [DmuObject; MAX_DNODES],
    next_object_id: u64,
}

impl ObjectDirectory {
    pub const fn new() -> Self {
        Self {
            objects: [DmuObject::empty(); MAX_DNODES],
            next_object_id: 1,
        }
    }

    pub fn allocate_object(&mut self, obj_type: ObjectType) -> Option<u64> {
        for obj in &mut self.objects {
            if obj.object_type == ObjectType::None {
                let id = self.next_object_id;
                self.next_object_id += 1;

                *obj = match obj_type {
                    ObjectType::File => DmuObject::new_file(id),
                    ObjectType::Directory => DmuObject::new_directory(id),
                    _ => DmuObject::empty(),
                };

                return Some(id);
            }
        }
        None
    }

    pub fn free_object(&mut self, object_id: u64) {
        for obj in &mut self.objects {
            if obj.object_id == object_id {
                *obj = DmuObject::empty();
                return;
            }
        }
    }

    pub fn get(&self, object_id: u64) -> Option<&DmuObject> {
        for obj in &self.objects {
            if obj.object_id == object_id && obj.object_type != ObjectType::None {
                return Some(obj);
            }
        }
        None
    }

    pub fn get_mut(&mut self, object_id: u64) -> Option<&mut DmuObject> {
        for obj in &mut self.objects {
            if obj.object_id == object_id && obj.object_type != ObjectType::None {
                return Some(obj);
            }
        }
        None
    }
}
