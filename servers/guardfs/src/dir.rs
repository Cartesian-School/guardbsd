//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: guardfs
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Operacje na katalogach w GuardFS.

use crate::inode::*;

pub const MAX_FILENAME: usize = 60;
pub const DIR_ENTRY_SIZE: usize = 64;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DirEntry {
    pub inode: u32,
    pub name_len: u8,
    pub file_type: u8,
    pub reserved: [u8; 2],
    pub name: [u8; MAX_FILENAME],
}

impl DirEntry {
    pub const fn empty() -> Self {
        Self {
            inode: 0,
            name_len: 0,
            file_type: 0,
            reserved: [0; 2],
            name: [0; MAX_FILENAME],
        }
    }

    pub fn new(inode: u32, name: &str, file_type: u8) -> Self {
        let mut entry = Self::empty();
        entry.inode = inode;
        entry.file_type = file_type;

        let name_bytes = name.as_bytes();
        let len = name_bytes.len().min(MAX_FILENAME);
        entry.name[..len].copy_from_slice(&name_bytes[..len]);
        entry.name_len = len as u8;

        entry
    }

    pub fn get_name(&self) -> &str {
        let len = self.name_len as usize;
        core::str::from_utf8(&self.name[..len]).unwrap_or("<invalid>")
    }

    pub fn is_valid(&self) -> bool {
        self.inode != 0 && self.name_len > 0
    }
}

pub struct Directory {
    pub entries: [DirEntry; 64], // 4KB / 64 bytes = 64 entries per block
}

impl Directory {
    pub const fn new() -> Self {
        Self {
            entries: [DirEntry::empty(); 64],
        }
    }

    pub fn init_root(&mut self) {
        // Add . and ..
        self.entries[0] = DirEntry::new(ROOT_INODE, ".", S_IFDIR as u8);
        self.entries[1] = DirEntry::new(ROOT_INODE, "..", S_IFDIR as u8);
    }

    pub fn add_entry(&mut self, entry: DirEntry) -> bool {
        for slot in &mut self.entries {
            if !slot.is_valid() {
                *slot = entry;
                return true;
            }
        }
        false // Directory full
    }

    pub fn remove_entry(&mut self, name: &str) -> bool {
        for slot in &mut self.entries {
            if slot.is_valid() && slot.get_name() == name {
                *slot = DirEntry::empty();
                return true;
            }
        }
        false
    }

    pub fn find_entry(&self, name: &str) -> Option<&DirEntry> {
        for entry in &self.entries {
            if entry.is_valid() && entry.get_name() == name {
                return Some(entry);
            }
        }
        None
    }

    pub fn list_entries(&self) -> impl Iterator<Item = &DirEntry> {
        self.entries.iter().filter(|e| e.is_valid())
    }

    pub fn to_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(self as *const _ as *const u8, core::mem::size_of::<Self>())
        }
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        let mut dir = Self::new();
        if data.len() >= core::mem::size_of::<Self>() {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    data.as_ptr(),
                    &mut dir as *mut _ as *mut u8,
                    core::mem::size_of::<Self>(),
                );
            }
        }
        dir
    }
}

/// Path resolution helper - returns array of path components
/// Caller must provide a buffer to store the components
pub fn split_path<'a>(path: &'a str, components: &mut [&'a str]) -> usize {
    let mut count = 0;
    for part in path.split('/') {
        if !part.is_empty() && count < components.len() {
            components[count] = part;
            count += 1;
        }
    }
    count
}

pub fn get_filename(path: &str) -> Option<&str> {
    path.rsplit('/').next().filter(|s| !s.is_empty())
}

pub fn get_parent_path(path: &str) -> &str {
    if let Some(pos) = path.rfind('/') {
        if pos == 0 {
            "/"
        } else {
            &path[..pos]
        }
    } else {
        "/"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dir_entry() {
        let entry = DirEntry::new(5, "test.txt", S_IFREG as u8);
        assert_eq!(entry.get_name(), "test.txt");
        assert_eq!(entry.inode, 5);
    }

    #[test]
    fn test_directory() {
        let mut dir = Directory::new();
        let entry = DirEntry::new(10, "file.dat", S_IFREG as u8);

        assert!(dir.add_entry(entry));
        assert!(dir.find_entry("file.dat").is_some());
        assert!(dir.remove_entry("file.dat"));
        assert!(dir.find_entry("file.dat").is_none());
    }

    #[test]
    fn test_path_split() {
        let mut parts = [""; 16];
        let count = split_path("/usr/local/bin", &mut parts);
        assert_eq!(count, 3);
        assert_eq!(parts[0], "usr");
        assert_eq!(parts[1], "local");
        assert_eq!(parts[2], "bin");
    }
}
