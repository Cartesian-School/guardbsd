//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: boot_stub
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Minimal ISO9660 reader for boot stub.
//!
//! Design goals:
//! - If init(base=0) is used during bring-up, ISO access is disabled,
//!   but embedded INIT_ELF "init" still loads.
//! - When ISO is mapped, we parse PVD at LBA 16 and root directory record.
//! - Supports basic directory traversal for arbitrary depth paths.

#![allow(dead_code)]

use core::slice;

const SECTOR_SIZE: usize = 2048;
const PVD_LBA: u32 = 16;
const PVD_ROOT_DIR_OFFSET: usize = 156;

// Conservative guardrails (avoid scanning absurd sizes if ISO_BASE is wrong).
const MAX_DIR_BYTES: usize = 8 * 1024 * 1024;   // 8 MiB max directory scan
const MAX_FILE_BYTES: usize = 64 * 1024 * 1024; // 64 MiB max file read

// ISO9660 directory record flags
const ISO_FLAG_DIRECTORY: u8 = 0x02;

#[repr(C, packed)]
struct DirEntryHeader {
    length: u8,              // 0
    ext_length: u8,          // 1
    extent_lba: [u8; 8],     // 2..9   (both-endian; we use LE part)
    data_length: [u8; 8],    // 10..17 (both-endian; we use LE part)
    datetime: [u8; 7],       // 18..24
    flags: u8,               // 25
    unit_size: u8,           // 26
    gap_size: u8,            // 27
    seq_num: [u8; 4],        // 28..31 (both-endian; not used)
    name_len: u8,            // 32
    // name follows at offset 33
}

static mut ISO_BASE: usize = 0;
static mut ROOT_LBA: u32 = 0;
static mut ROOT_SIZE: u32 = 0;
static mut ISO_READY: bool = false;

// Embedded init ELF for boot-time loading when no real ISO is present.
// Path is relative to boot_stub/src/.
static INIT_ELF: &[u8] = include_bytes!("../../../../servers/init/init_min.bin");

/// Safe-ish slice accessor for mapped ISO memory.
/// This prevents arithmetic overflow and absurd lengths. It cannot guarantee
/// the address is mapped; that's the caller's responsibility (bootloader/MMU).
unsafe fn iso_slice(at_lba: u32, len: usize) -> Option<&'static [u8]> {
    if ISO_BASE == 0 {
        return None;
    }
    if len == 0 || len > MAX_FILE_BYTES {
        return None;
    }

    let offset = (at_lba as usize).checked_mul(SECTOR_SIZE)?;
    let end = offset.checked_add(len)?;
    let base_end = ISO_BASE.checked_add(end)?;
    // base_end is computed only to validate overflow; we don't use it directly.
    let _ = base_end;

    let ptr = (ISO_BASE + offset) as *const u8;
    Some(slice::from_raw_parts(ptr, len))
}

/// Initialize ISO9660 reader with a memory-mapped ISO base address.
/// If `base == 0`, ISO access is disabled (but embedded "init" still works).
pub fn init(base: usize) {
    unsafe {
        ISO_BASE = base;
        ROOT_LBA = 0;
        ROOT_SIZE = 0;
        ISO_READY = false;

        // Bring-up mode: allow init(0) without touching memory.
        if base == 0 {
            return;
        }

        // Overflow check for PVD access: base + (PVD_LBA * SECTOR_SIZE) + SECTOR_SIZE
        let pvd_offset = (PVD_LBA as usize).checked_mul(SECTOR_SIZE);
        if pvd_offset.is_none() {
            return;
        }
        let pvd_end = pvd_offset
            .and_then(|o| o.checked_add(SECTOR_SIZE))
            .and_then(|o| base.checked_add(o));
        if pvd_end.is_none() {
            return;
        }

        // Read Primary Volume Descriptor sector.
        let pvd = match iso_slice(PVD_LBA, SECTOR_SIZE) {
            Some(s) => s,
            None => return,
        };

        // Validate PVD: Type = 1, Identifier = "CD001", Version = 1
        // Layout: [0]=Type, [1..6]=Identifier, [6]=Version
        if pvd.len() < 7 {
            return;
        }
        if pvd[0] != 1 {
            return;
        }
        if &pvd[1..6] != b"CD001" {
            return;
        }
        if pvd[6] != 1 {
            return;
        }

        // Root directory record in PVD at offset 156.
        if PVD_ROOT_DIR_OFFSET + core::mem::size_of::<DirEntryHeader>() >= SECTOR_SIZE {
            return;
        }

        let root_entry_ptr = pvd.as_ptr().add(PVD_ROOT_DIR_OFFSET) as *const DirEntryHeader;
        let root_len = (*root_entry_ptr).length as usize;
        if root_len < 34 {
            return;
        }

        let lba = read_u32_le(&(*root_entry_ptr).extent_lba);
        let size = read_u32_le(&(*root_entry_ptr).data_length);

        // Sane bounds.
        if lba == 0 || size == 0 || (size as usize) > MAX_DIR_BYTES {
            return;
        }

        ROOT_LBA = lba;
        ROOT_SIZE = size;
        ISO_READY = true;
    }
}

/// Read a file by path from ISO9660 image.
/// Returns a borrowed slice into the mapped ISO memory, or embedded init ELF.
pub fn read_file(path: &str) -> Option<&'static [u8]> {
    // Fast-path for embedded init ELF (works even if ISO is not mapped).
    if path == "init" || path.ends_with("/init") {
        return if INIT_ELF.is_empty() { None } else { Some(INIT_ELF) };
    }

    unsafe {
        if !ISO_READY || ISO_BASE == 0 || ROOT_LBA == 0 || ROOT_SIZE == 0 {
            return None;
        }

        // Normalize and split components, skipping empty segments (handles leading '/').
        let mut comps = PathComponents::new(path);
        let first = comps.next()?;
        let mut cur_lba = ROOT_LBA;
        let mut cur_size = ROOT_SIZE;

        // Traverse directories for all but last component.
        let mut current_name = first;
        loop {
            let next_name_opt = comps.next();

            // Find current component in current directory.
            let (e_lba, e_size, e_flags) = find_entry(cur_lba, cur_size, current_name)?;

            match next_name_opt {
                Some(next_name) => {
                    // Must be a directory to continue traversal.
                    if (e_flags & ISO_FLAG_DIRECTORY) == 0 {
                        return None;
                    }
                    cur_lba = e_lba;
                    cur_size = e_size;
                    current_name = next_name;
                }
                None => {
                    // Last component: must be a file (directory read not supported here).
                    if (e_flags & ISO_FLAG_DIRECTORY) != 0 {
                        return None;
                    }
                    let sz = e_size as usize;
                    if sz == 0 || sz > MAX_FILE_BYTES {
                        return None;
                    }
                    return iso_slice(e_lba, sz);
                }
            }
        }
    }
}

/// Iterator over path components without allocation.
struct PathComponents<'a> {
    s: &'a [u8],
    i: usize,
}

impl<'a> PathComponents<'a> {
    fn new(path: &'a str) -> Self {
        Self {
            s: path.as_bytes(),
            i: 0,
        }
    }

    fn next(&mut self) -> Option<&'a str> {
        while self.i < self.s.len() && self.s[self.i] == b'/' {
            self.i += 1;
        }
        if self.i >= self.s.len() {
            return None;
        }
        let start = self.i;
        while self.i < self.s.len() && self.s[self.i] != b'/' {
            self.i += 1;
        }
        core::str::from_utf8(&self.s[start..self.i]).ok()
    }
}

unsafe fn find_entry(dir_lba: u32, dir_size: u32, name: &str) -> Option<(u32, u32, u8)> {
    if dir_lba == 0 || dir_size == 0 || (dir_size as usize) > MAX_DIR_BYTES {
        return None;
    }

    // Read directory bytes as a slice to avoid manual pointer arithmetic for base checks.
    let dir_bytes = iso_slice(dir_lba, dir_size as usize)?;
    let mut offset: usize = 0;

    while offset < dir_bytes.len() {
        // Ensure we can read the header at least.
        if offset + core::mem::size_of::<DirEntryHeader>() > dir_bytes.len() {
            return None;
        }

        let entry_ptr = dir_bytes.as_ptr().add(offset) as *const DirEntryHeader;
        let len = (*entry_ptr).length as usize;

        if len == 0 {
            // Advance to the next sector boundary.
            offset = (offset + SECTOR_SIZE) & !(SECTOR_SIZE - 1);
            continue;
        }

        // Defensive: avoid infinite loops / malformed records.
        if len < 34 || offset + len > dir_bytes.len() {
            offset = offset.saturating_add(1);
            continue;
        }

        let name_len = (*entry_ptr).name_len as usize;
        if 33 + name_len > len {
            offset = offset.saturating_add(len);
            continue;
        }

        let entry_name = slice::from_raw_parts((entry_ptr as *const u8).add(33), name_len);

        if name_matches(entry_name, name) {
            let e_lba = read_u32_le(&(*entry_ptr).extent_lba);
            let e_size = read_u32_le(&(*entry_ptr).data_length);
            let e_flags = (*entry_ptr).flags;

            // Sane size bounds.
            if e_lba == 0 {
                return None;
            }
            if (e_size as usize) > MAX_FILE_BYTES {
                return None;
            }
            return Some((e_lba, e_size, e_flags));
        }

        offset = offset.saturating_add(len);
    }

    None
}

/// Match ISO9660 entry name with a target component.
/// - Strips version suffix ";1"
/// - Case-insensitive
/// - Handles special entries "." and ".." encoded as 0 and 1 in ISO9660.
fn name_matches(entry_name: &[u8], target: &str) -> bool {
    // Special entries: 0 => ".", 1 => ".."
    if target == "." {
        return entry_name.len() == 1 && entry_name[0] == 0;
    }
    if target == ".." {
        return entry_name.len() == 1 && entry_name[0] == 1;
    }

    let target_bytes = target.as_bytes();
    let mut i = 0usize;

    for &b in entry_name {
        if b == b';' {
            break;
        }
        if i >= target_bytes.len() {
            return false;
        }

        // ASCII case-fold to lower.
        let eb = if (b'A'..=b'Z').contains(&b) { b + 32 } else { b };
        let tb0 = target_bytes[i];
        let tb = if (b'A'..=b'Z').contains(&tb0) { tb0 + 32 } else { tb0 };

        if eb != tb {
            return false;
        }
        i += 1;
    }

    i == target_bytes.len()
}

/// Read u32 little-endian from a both-endian ISO9660 field.
/// The field is 8 bytes: [LE32][BE32]. We take the first 4 bytes.
fn read_u32_le(bytes8: &[u8; 8]) -> u32 {
    (bytes8[0] as u32)
        | (bytes8[1] as u32) << 8
        | (bytes8[2] as u32) << 16
        | (bytes8[3] as u32) << 24
}
