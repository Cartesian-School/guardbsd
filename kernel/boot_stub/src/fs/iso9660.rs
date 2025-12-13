//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: boot_stub
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Minimalny czytnik systemu plików ISO 9660 używany przez boot stub.

const SECTOR_SIZE: usize = 2048;

#[repr(C, packed)]
struct DirEntry {
    length: u8,
    ext_length: u8,
    extent_lba: [u8; 8],
    data_length: [u8; 8],
    datetime: [u8; 7],
    flags: u8,
    unit_size: u8,
    gap_size: u8,
    seq_num: [u8; 4],
    name_len: u8,
}

static mut ISO_BASE: usize = 0;
static mut ROOT_LBA: u32 = 0;
static mut ROOT_SIZE: u32 = 0;

// Embedded init ELF for boot-time loading when no real ISO is present.
// Path is relative to boot_stub/src/.
static INIT_ELF: &[u8] = include_bytes!("../../../../servers/init/init_min.bin");

pub fn init(base: usize) {
    unsafe {
        ISO_BASE = base;
        let pvd = (base + 16 * SECTOR_SIZE) as *const u8;
        let root_entry = pvd.add(156) as *const DirEntry;
        ROOT_LBA = read_u32(&(*root_entry).extent_lba);
        ROOT_SIZE = read_u32(&(*root_entry).data_length);
    }
}

pub fn read_file(path: &str) -> Option<&'static [u8]> {
    // Fast-path for embedded init ELF
    if path == "init" || path.ends_with("/init") {
        if INIT_ELF.is_empty() {
            return None;
        }
        // Safety: INIT_ELF is 'static
        return Some(INIT_ELF);
    }

    unsafe {
        if ISO_BASE == 0 { return None; }
        
        let parts: [&str; 2] = if path.starts_with('/') {
            let p = &path[1..];
            if let Some(pos) = p.find('/') {
                [&p[..pos], &p[pos+1..]]
            } else {
                ["", p]
            }
        } else {
            ["", path]
        };
        
        let (dir_name, file_name) = (parts[0], parts[1]);
        let mut lba = ROOT_LBA;
        let mut size = ROOT_SIZE;
        
        if !dir_name.is_empty() {
            if let Some((d_lba, d_size)) = find_entry(lba, size, dir_name) {
                lba = d_lba;
                size = d_size;
            } else {
                return None;
            }
        }
        
        if let Some((f_lba, f_size)) = find_entry(lba, size, file_name) {
            let ptr = (ISO_BASE + f_lba as usize * SECTOR_SIZE) as *const u8;
            return Some(core::slice::from_raw_parts(ptr, f_size as usize));
        }
        
        None
    }
}

unsafe fn find_entry(lba: u32, size: u32, name: &str) -> Option<(u32, u32)> {
    let dir_data = (ISO_BASE + lba as usize * SECTOR_SIZE) as *const u8;
    let mut offset = 0;
    
    while offset < size as usize {
        let entry = dir_data.add(offset) as *const DirEntry;
        let len = (*entry).length as usize;
        
        if len == 0 {
            offset = (offset + SECTOR_SIZE) & !(SECTOR_SIZE - 1);
            continue;
        }
        
        let name_len = (*entry).name_len as usize;
        let entry_name = core::slice::from_raw_parts(
            (entry as *const u8).add(33),
            name_len
        );
        
        if name_matches(entry_name, name) {
            let e_lba = read_u32(&(*entry).extent_lba);
            let e_size = read_u32(&(*entry).data_length);
            return Some((e_lba, e_size));
        }
        
        offset += len;
    }
    
    None
}

fn name_matches(entry_name: &[u8], target: &str) -> bool {
    let target_bytes = target.as_bytes();
    let mut i = 0;
    
    for &b in entry_name {
        if b == b';' { break; }
        if i >= target_bytes.len() { return false; }
        
        let eb = if b >= b'A' && b <= b'Z' { b + 32 } else { b };
        let tb = if target_bytes[i] >= b'A' && target_bytes[i] <= b'Z' {
            target_bytes[i] + 32
        } else {
            target_bytes[i]
        };
        
        if eb != tb { return false; }
        i += 1;
    }
    
    i == target_bytes.len()
}

fn read_u32(bytes: &[u8]) -> u32 {
    bytes[0] as u32 |
    (bytes[1] as u32) << 8 |
    (bytes[2] as u32) << 16 |
    (bytes[3] as u32) << 24
}
