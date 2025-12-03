// ELF64 Loader - Minimal Implementation
// BSD 3-Clause License

#![no_std]

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

pub struct LoadedElf {
    pub entry: u64,
    pub segments: usize,
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
