//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: guardfs
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Zarządzanie i-węzłami GuardFS.

use crate::superblock::*;

pub const MAX_EXTENTS: usize = 12;
pub const ROOT_INODE: u32 = 2;

// File types (matches BSD)
pub const S_IFREG: u16 = 0x8000; // Regular file
pub const S_IFDIR: u16 = 0x4000; // Directory
pub const S_IFLNK: u16 = 0xA000; // Symbolic link
pub const S_IFBLK: u16 = 0x6000; // Block device
pub const S_IFCHR: u16 = 0x2000; // Character device
pub const S_IFIFO: u16 = 0x1000; // FIFO
pub const S_IFSOCK: u16 = 0xC000; // Socket

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Extent {
    pub start_block: u64,
    pub length: u32,
    pub flags: u32,
}

impl Extent {
    pub const fn empty() -> Self {
        Self {
            start_block: 0,
            length: 0,
            flags: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct GuardFsInode {
    // Basic metadata
    pub mode: u16,
    pub uid: u16,
    pub gid: u16,
    pub link_count: u16,

    // Size and blocks
    pub size: u64,
    pub blocks: u64,

    // Timestamps (Unix epoch)
    pub atime: u64,
    pub mtime: u64,
    pub ctime: u64,
    pub crtime: u64,

    // Extent pointers
    pub extents: [Extent; MAX_EXTENTS],
    pub indirect1: u64,
    pub indirect2: u64,
    pub indirect3: u64,

    // Snapshot support
    pub snapshot_gen: u64,
    pub cow_block: u64,

    // Compression
    pub compressed: u8,
    pub comp_algo: u8,
    pub uncomp_size: u32,

    // Checksum
    pub checksum: u32,

    // Reserved
    pub reserved: [u8; 88],
}

impl GuardFsInode {
    pub const fn empty() -> Self {
        Self {
            mode: 0,
            uid: 0,
            gid: 0,
            link_count: 0,

            size: 0,
            blocks: 0,

            atime: 0,
            mtime: 0,
            ctime: 0,
            crtime: 0,

            extents: [Extent::empty(); MAX_EXTENTS],
            indirect1: 0,
            indirect2: 0,
            indirect3: 0,

            snapshot_gen: 0,
            cow_block: 0,

            compressed: 0,
            comp_algo: 0,
            uncomp_size: 0,

            checksum: 0,

            reserved: [0; 88],
        }
    }

    pub fn new_file(uid: u16, gid: u16, mode: u16) -> Self {
        let mut inode = Self::empty();
        inode.mode = S_IFREG | (mode & 0x0FFF);
        inode.uid = uid;
        inode.gid = gid;
        inode.link_count = 1;

        let now = get_time();
        inode.atime = now;
        inode.mtime = now;
        inode.ctime = now;
        inode.crtime = now;

        inode.update_checksum();
        inode
    }

    pub fn new_dir(uid: u16, gid: u16, mode: u16) -> Self {
        let mut inode = Self::empty();
        inode.mode = S_IFDIR | (mode & 0x0FFF);
        inode.uid = uid;
        inode.gid = gid;
        inode.link_count = 2; // . and ..

        let now = get_time();
        inode.atime = now;
        inode.mtime = now;
        inode.ctime = now;
        inode.crtime = now;

        inode.update_checksum();
        inode
    }

    pub fn is_dir(&self) -> bool {
        self.mode & 0xF000 == S_IFDIR
    }

    pub fn is_file(&self) -> bool {
        self.mode & 0xF000 == S_IFREG
    }

    pub fn add_extent(&mut self, extent: Extent) -> bool {
        for i in 0..MAX_EXTENTS {
            if self.extents[i].is_empty() {
                self.extents[i] = extent;
                self.blocks += extent.length as u64;
                self.update_checksum();
                return true;
            }
        }
        false // No free extent slots
    }

    pub fn get_block_for_offset(&self, offset: u64) -> Option<u64> {
        let block_offset = offset / BLOCK_SIZE as u64;
        let mut current_block = 0u64;

        for extent in &self.extents {
            if extent.is_empty() {
                break;
            }

            let extent_blocks = extent.length as u64;
            if block_offset < current_block + extent_blocks {
                let extent_offset = block_offset - current_block;
                return Some(extent.start_block + extent_offset);
            }

            current_block += extent_blocks;
        }

        None
    }

    fn update_checksum(&mut self) {
        let data = unsafe {
            core::slice::from_raw_parts(
                self as *const _ as *const u8,
                core::mem::size_of::<Self>() - 4 - 88, // Exclude checksum and reserved
            )
        };

        self.checksum = crate::superblock::crc32(data);
    }

    pub fn verify_checksum(&self) -> bool {
        let stored = self.checksum;
        let mut temp = *self;
        temp.update_checksum();
        stored == temp.checksum
    }
}

// Get current time (Unix epoch)
fn get_time() -> u64 {
    // TODO: Implement real time source
    // For now, return monotonic counter
    static mut TIME_COUNTER: u64 = 1700000000; // ~2023
    unsafe {
        TIME_COUNTER += 1;
        TIME_COUNTER
    }
}

pub struct InodeTable {
    pub inodes: [GuardFsInode; DEFAULT_INODES as usize],
    free_bitmap: [u64; 16], // 1024 bits / 64 = 16 u64s
}

impl InodeTable {
    pub const fn new() -> Self {
        Self {
            inodes: [GuardFsInode::empty(); DEFAULT_INODES as usize],
            free_bitmap: [0xFFFFFFFFFFFFFFFF; 16], // All free
        }
    }

    pub fn init(&mut self) {
        // Reserve inode 0 and 1 (not used)
        self.free_bitmap[0] &= !(1u64 << 0);
        self.free_bitmap[0] &= !(1u64 << 1);

        // Create root directory (inode 2)
        self.inodes[2] = GuardFsInode::new_dir(0, 0, 0o755);
        self.free_bitmap[0] &= !(1u64 << 2);
    }

    pub fn allocate(&mut self) -> Option<u32> {
        for (word_idx, word) in self.free_bitmap.iter_mut().enumerate() {
            if *word != 0 {
                // Find first set bit
                let bit = word.trailing_zeros() as usize;
                if bit < 64 {
                    *word &= !(1u64 << bit);
                    let inode_num = (word_idx * 64 + bit) as u32;
                    return Some(inode_num);
                }
            }
        }
        None
    }

    pub fn free(&mut self, inode_num: u32) {
        let word_idx = (inode_num / 64) as usize;
        let bit = (inode_num % 64) as usize;

        if word_idx < 16 {
            self.free_bitmap[word_idx] |= 1u64 << bit;
            self.inodes[inode_num as usize] = GuardFsInode::empty();
        }
    }

    pub fn get(&self, inode_num: u32) -> Option<&GuardFsInode> {
        if inode_num < DEFAULT_INODES {
            let inode = &self.inodes[inode_num as usize];
            if inode.link_count > 0 {
                return Some(inode);
            }
        }
        None
    }

    pub fn get_mut(&mut self, inode_num: u32) -> Option<&mut GuardFsInode> {
        if inode_num < DEFAULT_INODES {
            let inode = &mut self.inodes[inode_num as usize];
            if inode.link_count > 0 {
                return Some(inode);
            }
        }
        None
    }
}
