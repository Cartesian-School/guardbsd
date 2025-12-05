// servers/guardzfs/src/zap.rs
// GuardZFS ZAP (ZFS Attribute Processor) - Directory Implementation
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

/// ZAP Entry - key/value pair for directory entries
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ZapEntry {
    pub key: [u8; 64],        // Filename
    pub key_len: u8,
    pub value: u64,           // Object ID
    pub valid: bool,
}

impl ZapEntry {
    pub const fn empty() -> Self {
        Self {
            key: [0; 64],
            key_len: 0,
            value: 0,
            valid: false,
        }
    }
    
    pub fn new(name: &str, object_id: u64) -> Self {
        let mut entry = Self::empty();
        let name_bytes = name.as_bytes();
        let len = name_bytes.len().min(63);
        entry.key[..len].copy_from_slice(&name_bytes[..len]);
        entry.key_len = len as u8;
        entry.value = object_id;
        entry.valid = true;
        entry
    }
    
    pub fn get_name(&self) -> &str {
        let len = self.key_len as usize;
        core::str::from_utf8(&self.key[..len]).unwrap_or("<invalid>")
    }
    
    pub fn matches(&self, name: &str) -> bool {
        self.valid && self.get_name() == name
    }
}

/// ZAP Directory - hash table for directory entries
pub struct ZapDirectory {
    pub entries: [ZapEntry; 64], // 64 entries per block
}

impl ZapDirectory {
    pub const fn new() -> Self {
        Self {
            entries: [ZapEntry::empty(); 64],
        }
    }
    
    pub fn insert(&mut self, name: &str, object_id: u64) -> bool {
        // Find empty slot
        for entry in &mut self.entries {
            if !entry.valid {
                *entry = ZapEntry::new(name, object_id);
                return true;
            }
        }
        false
    }
    
    pub fn lookup(&self, name: &str) -> Option<u64> {
        for entry in &self.entries {
            if entry.matches(name) {
                return Some(entry.value);
            }
        }
        None
    }
    
    pub fn remove(&mut self, name: &str) -> bool {
        for entry in &mut self.entries {
            if entry.matches(name) {
                *entry = ZapEntry::empty();
                return true;
            }
        }
        false
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
        let mut zap = Self::new();
        if data.len() >= core::mem::size_of::<Self>() {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    data.as_ptr(),
                    &mut zap as *mut _ as *mut u8,
                    core::mem::size_of::<Self>()
                );
            }
        }
        zap
    }
}

