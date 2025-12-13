//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: guardfs
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Zarządzanie snapshotami GuardFS.

pub const MAX_SNAPSHOTS: usize = 16;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Snapshot {
    pub id: u32,
    pub timestamp: u64,
    pub root_inode: u32,
    pub name: [u8; 64],
    pub parent_id: u32,
    pub refcount: u32,
    pub flags: u32,
    pub reserved: [u8; 40],
}

impl Snapshot {
    pub const fn empty() -> Self {
        Self {
            id: 0,
            timestamp: 0,
            root_inode: 0,
            name: [0; 64],
            parent_id: 0,
            refcount: 0,
            flags: 0,
            reserved: [0; 40],
        }
    }

    pub fn new(id: u32, name: &str, root_inode: u32) -> Self {
        let mut snap = Self::empty();
        snap.id = id;
        snap.root_inode = root_inode;
        snap.timestamp = get_time();
        snap.refcount = 1;

        let name_bytes = name.as_bytes();
        let copy_len = name_bytes.len().min(63);
        snap.name[..copy_len].copy_from_slice(&name_bytes[..copy_len]);

        snap
    }

    pub fn is_valid(&self) -> bool {
        self.id != 0 && self.refcount > 0
    }

    pub fn get_name(&self) -> &str {
        let len = self.name.iter().position(|&c| c == 0).unwrap_or(64);
        core::str::from_utf8(&self.name[..len]).unwrap_or("<invalid>")
    }
}

pub struct SnapshotManager {
    snapshots: [Snapshot; MAX_SNAPSHOTS],
    next_id: u32,
}

impl SnapshotManager {
    pub const fn new() -> Self {
        Self {
            snapshots: [Snapshot::empty(); MAX_SNAPSHOTS],
            next_id: 1,
        }
    }

    pub fn create(&mut self, name: &str, root_inode: u32) -> Option<u32> {
        // Find free slot
        for snap in &mut self.snapshots {
            if !snap.is_valid() {
                *snap = Snapshot::new(self.next_id, name, root_inode);
                let id = self.next_id;
                self.next_id += 1;
                return Some(id);
            }
        }
        None // No free slots
    }

    pub fn delete(&mut self, id: u32) -> bool {
        for snap in &mut self.snapshots {
            if snap.id == id {
                snap.refcount = snap.refcount.saturating_sub(1);
                if snap.refcount == 0 {
                    *snap = Snapshot::empty();
                }
                return true;
            }
        }
        false
    }

    pub fn get(&self, id: u32) -> Option<&Snapshot> {
        for snap in &self.snapshots {
            if snap.id == id && snap.is_valid() {
                return Some(snap);
            }
        }
        None
    }

    pub fn find_by_name(&self, name: &str) -> Option<&Snapshot> {
        for snap in &self.snapshots {
            if snap.is_valid() && snap.get_name() == name {
                return Some(snap);
            }
        }
        None
    }

    pub fn list(&self) -> &[Snapshot] {
        &self.snapshots
    }

    pub fn count(&self) -> usize {
        self.snapshots.iter().filter(|s| s.is_valid()).count()
    }
}

fn get_time() -> u64 {
    static mut TIME: u64 = 1700000000;
    unsafe {
        TIME += 1;
        TIME
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_create() {
        let mut mgr = SnapshotManager::new();
        let id = mgr.create("test_snap", 2);
        assert!(id.is_some());
        assert_eq!(id.unwrap(), 1);
    }

    #[test]
    fn test_snapshot_find() {
        let mut mgr = SnapshotManager::new();
        let id = mgr.create("my_snapshot", 2).unwrap();

        let snap = mgr.find_by_name("my_snapshot");
        assert!(snap.is_some());
        assert_eq!(snap.unwrap().id, id);
    }
}
