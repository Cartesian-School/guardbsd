//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: guardfs
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! System journalingu GuardFS.

use crate::superblock::BLOCK_SIZE;

pub const JOURNAL_MAGIC: u32 = 0x474A524E; // "GJRN"
pub const JOURNAL_ENTRIES: usize = 8; // 8 blocks = 8 entries (one per block)

#[repr(u8)]
#[derive(Copy, Clone, PartialEq)]
pub enum JournalOp {
    InodeUpdate = 1,
    BlockAlloc = 2,
    BlockFree = 3,
    ExtentAdd = 4,
    DirectoryAdd = 5,
    DirectoryRemove = 6,
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq)]
pub enum JournalStatus {
    Pending = 0,
    Committed = 1,
    Aborted = 2,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct JournalEntry {
    pub magic: u32,
    pub seq: u64,
    pub timestamp: u64,

    pub op_type: u8,
    pub status: u8,
    pub reserved: [u8; 2],

    pub inode_num: u32,
    pub block_num: u64,

    // Old data for rollback
    pub old_data_len: u32,
    pub old_data: [u8; 3968],

    pub checksum: u32,
}

impl JournalEntry {
    pub const fn empty() -> Self {
        Self {
            magic: JOURNAL_MAGIC,
            seq: 0,
            timestamp: 0,
            op_type: 0,
            status: JournalStatus::Pending as u8,
            reserved: [0; 2],
            inode_num: 0,
            block_num: 0,
            old_data_len: 0,
            old_data: [0; 3968],
            checksum: 0,
        }
    }

    pub fn new(op_type: JournalOp, inode_num: u32, block_num: u64) -> Self {
        let mut entry = Self::empty();
        entry.op_type = op_type as u8;
        entry.inode_num = inode_num;
        entry.block_num = block_num;
        entry.timestamp = get_time();
        entry
    }

    pub fn set_old_data(&mut self, data: &[u8]) {
        let len = data.len().min(3968);
        self.old_data[..len].copy_from_slice(&data[..len]);
        self.old_data_len = len as u32;
        self.update_checksum();
    }

    pub fn commit(&mut self) {
        self.status = JournalStatus::Committed as u8;
        self.update_checksum();
    }

    pub fn abort(&mut self) {
        self.status = JournalStatus::Aborted as u8;
        self.update_checksum();
    }

    fn update_checksum(&mut self) {
        let data = unsafe {
            core::slice::from_raw_parts(
                self as *const _ as *const u8,
                core::mem::size_of::<Self>() - 4,
            )
        };
        self.checksum = crate::superblock::crc32(data);
    }

    pub fn verify(&self) -> bool {
        if self.magic != JOURNAL_MAGIC {
            return false;
        }

        let stored = self.checksum;
        let mut temp = *self;
        temp.update_checksum();
        stored == temp.checksum
    }
}

pub struct Journal {
    entries: [JournalEntry; JOURNAL_ENTRIES],
    head: usize,
    tail: usize,
    seq: u64,
}

impl Journal {
    pub const fn new() -> Self {
        Self {
            entries: [JournalEntry::empty(); JOURNAL_ENTRIES],
            head: 0,
            tail: 0,
            seq: 1,
        }
    }

    pub fn log(
        &mut self,
        op_type: JournalOp,
        inode_num: u32,
        block_num: u64,
        old_data: Option<&[u8]>,
    ) -> bool {
        if self.is_full() {
            return false;
        }

        let mut entry = JournalEntry::new(op_type, inode_num, block_num);
        entry.seq = self.seq;
        self.seq += 1;

        if let Some(data) = old_data {
            entry.set_old_data(data);
        }

        self.entries[self.tail] = entry;
        self.tail = (self.tail + 1) % JOURNAL_ENTRIES;

        true
    }

    pub fn commit(&mut self, seq: u64) {
        for entry in &mut self.entries {
            if entry.seq == seq {
                entry.commit();
                return;
            }
        }
    }

    pub fn recover(&mut self) -> usize {
        let mut recovered = 0;

        for entry in &self.entries {
            if !entry.verify() {
                continue;
            }

            if entry.status == JournalStatus::Pending as u8 {
                // Rollback uncommitted transaction
                // TODO: Implement rollback logic
                recovered += 1;
            }
        }

        // Clear journal after recovery
        self.clear();

        recovered
    }

    pub fn clear(&mut self) {
        self.entries = [JournalEntry::empty(); JOURNAL_ENTRIES];
        self.head = 0;
        self.tail = 0;
    }

    fn is_full(&self) -> bool {
        (self.tail + 1) % JOURNAL_ENTRIES == self.head
    }
}

fn get_time() -> u64 {
    static mut TIME: u64 = 1700000000;
    unsafe {
        TIME += 1;
        TIME
    }
}
