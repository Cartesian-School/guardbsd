// ELF64 Loader - Minimal Implementation
// BSD 3-Clause License

#![no_std]

extern crate alloc;

use core::mem::size_of;

#[repr(C)]
pub struct Elf64Ehdr {
    pub e_ident: [u8; 16],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

#[repr(C)]
pub struct Elf64Phdr {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

const PT_LOAD: u32 = 1;
const ELFMAG: &[u8; 4] = b"\x7FELF";

// ELF constants
const ELFCLASS64: u8 = 2;  // 64-bit
const ELFDATA2LSB: u8 = 1;  // Little endian
const ET_EXEC: u16 = 2;     // Executable file
const ET_DYN: u16 = 3;      // Shared object (PIE)

// Architecture constants
const EM_X86_64: u16 = 62;  // x86-64
const EM_AARCH64: u16 = 183; // ARM64

pub struct LoadedElf {
    pub entry: u64,
    pub segments: usize,
}

/// Parse ELF header only (Day 18)
/// Returns entry point and number of loadable segments
pub fn parse_elf_header(data: &[u8]) -> Result<(u64, usize), &'static str> {
    if data.len() < size_of::<Elf64Ehdr>() {
        return Err("ELF too small");
    }

    let ehdr = unsafe { &*(data.as_ptr() as *const Elf64Ehdr) };
    
    // Validate magic number
    if &ehdr.e_ident[0..4] != ELFMAG {
        return Err("Invalid ELF magic");
    }

    // Validate 64-bit
    if ehdr.e_ident[4] != ELFCLASS64 {
        return Err("Not 64-bit ELF");
    }
    
    // Validate little-endian
    if ehdr.e_ident[5] != ELFDATA2LSB {
        return Err("Not little-endian ELF");
    }
    
    // Validate executable or PIE
    if ehdr.e_type != ET_EXEC && ehdr.e_type != ET_DYN {
        return Err("Not executable ELF");
    }
    
    // Validate architecture
    #[cfg(target_arch = "x86_64")]
    if ehdr.e_machine != EM_X86_64 {
        return Err("Wrong architecture (not x86-64)");
    }
    
    #[cfg(target_arch = "aarch64")]
    if ehdr.e_machine != EM_AARCH64 {
        return Err("Wrong architecture (not aarch64)");
    }

    // Count loadable segments
    let mut segments = 0;
    for i in 0..ehdr.e_phnum {
        let phdr_off = ehdr.e_phoff as usize + (i as usize * ehdr.e_phentsize as usize);
        if phdr_off + size_of::<Elf64Phdr>() > data.len() {
            return Err("Program header out of bounds");
        }
        
        let phdr = unsafe { &*(data.as_ptr().add(phdr_off) as *const Elf64Phdr) };
        if phdr.p_type == PT_LOAD {
            segments += 1;
        }
    }
    
    if segments == 0 {
        return Err("No loadable segments");
    }

    Ok((ehdr.e_entry, segments))
}

/// Get program segment info (Day 18)
pub struct SegmentInfo {
    pub vaddr: u64,
    pub memsz: u64,
    pub filesz: u64,
    pub offset: u64,
    pub flags: u32,
}

/// Extract all loadable segments from ELF (Day 18)
pub fn get_loadable_segments(data: &[u8]) -> Result<alloc::vec::Vec<SegmentInfo>, &'static str> {
    if data.len() < size_of::<Elf64Ehdr>() {
        return Err("ELF too small");
    }

    let ehdr = unsafe { &*(data.as_ptr() as *const Elf64Ehdr) };
    let mut segments = alloc::vec::Vec::new();
    
    for i in 0..ehdr.e_phnum {
        let phdr_off = ehdr.e_phoff as usize + (i as usize * ehdr.e_phentsize as usize);
        if phdr_off + size_of::<Elf64Phdr>() > data.len() {
            continue;
        }
        
        let phdr = unsafe { &*(data.as_ptr().add(phdr_off) as *const Elf64Phdr) };
        if phdr.p_type == PT_LOAD {
            segments.push(SegmentInfo {
                vaddr: phdr.p_vaddr,
                memsz: phdr.p_memsz,
                filesz: phdr.p_filesz,
                offset: phdr.p_offset,
                flags: phdr.p_flags,
            });
        }
    }
    
    Ok(segments)
}

pub fn load_elf_from_file(path: &str, addr_space: &mut crate::mm::AddressSpace) -> Result<LoadedElf, &'static str> {
    // Read ELF from filesystem
    let data = crate::fs::iso9660::read_file(path)
        .ok_or("File not found")?;
    
    parse_and_load_elf(data, addr_space)
}

pub fn parse_and_load_elf(data: &[u8], addr_space: &mut crate::mm::AddressSpace) -> Result<LoadedElf, &'static str> {
    if data.len() < size_of::<Elf64Ehdr>() {
        return Err("ELF too small");
    }

    let ehdr = unsafe { &*(data.as_ptr() as *const Elf64Ehdr) };
    
    if &ehdr.e_ident[0..4] != ELFMAG {
        return Err("Invalid ELF magic");
    }

    if ehdr.e_ident[4] != 2 {
        return Err("Not 64-bit ELF");
    }

    let mut segments = 0;
    for i in 0..ehdr.e_phnum {
        let phdr_off = ehdr.e_phoff as usize + (i as usize * ehdr.e_phentsize as usize);
        if phdr_off + size_of::<Elf64Phdr>() > data.len() {
            return Err("Program header out of bounds");
        }
        
        let phdr = unsafe { &*(data.as_ptr().add(phdr_off) as *const Elf64Phdr) };
        if phdr.p_type == PT_LOAD {
            segments += 1;
        }
    }

    // Load PT_LOAD segments into process memory
    for i in 0..ehdr.e_phnum {
        let phdr_off = ehdr.e_phoff as usize + (i as usize * ehdr.e_phentsize as usize);
        if phdr_off + size_of::<Elf64Phdr>() > data.len() {
            continue;
        }
        
        let phdr = unsafe { &*(data.as_ptr().add(phdr_off) as *const Elf64Phdr) };
        if phdr.p_type != PT_LOAD { continue; }

        let vaddr = phdr.p_vaddr as usize;
        let memsz = phdr.p_memsz as usize;
        let filesz = phdr.p_filesz as usize;
        let offset = phdr.p_offset as usize;

        // Allocate physical pages
        let pages = (memsz + 4095) / 4096;
        for p in 0..pages {
            let phys = crate::mm::alloc_page().ok_or("Out of memory")?;
            let virt = vaddr + p * 4096;
            
            // Copy data to physical page
            if p * 4096 < filesz {
                let copy_size = core::cmp::min(4096, filesz - p * 4096);
                let src = unsafe { data.as_ptr().add(offset + p * 4096) };
                let dst = phys as *mut u8;
                unsafe {
                    core::ptr::copy_nonoverlapping(src, dst, copy_size);
                }
            }
            
            // Map into process address space
            let mut flags = crate::mm::PageFlags::PRESENT | crate::mm::PageFlags::USER;
            if phdr.p_flags & 2 != 0 { flags |= crate::mm::PageFlags::WRITABLE; }
            
            addr_space.map(virt, phys, flags);
        }
    }

    Ok(LoadedElf {
        entry: ehdr.e_entry,
        segments,
    })
}
