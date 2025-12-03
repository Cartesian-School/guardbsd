// Virtual Memory Manager - x86_64 Page Tables
// BSD 3-Clause License

#![no_std]

use super::pmm;

const PAGE_SIZE: usize = 4096;
const ENTRIES: usize = 512;

#[repr(C, align(4096))]
pub struct PageTable {
    entries: [u64; ENTRIES],
}

impl PageTable {
    pub fn new() -> Self {
        PageTable { entries: [0; ENTRIES] }
    }
}

bitflags::bitflags! {
    pub struct PageFlags: u64 {
        const PRESENT = 1 << 0;
        const WRITABLE = 1 << 1;
        const USER = 1 << 2;
        const ACCESSED = 1 << 5;
        const DIRTY = 1 << 6;
    }
}

pub struct AddressSpace {
    pml4_phys: usize,
}

impl AddressSpace {
    pub fn new() -> Option<Self> {
        let pml4_phys = pmm::alloc_page()?;
        unsafe {
            let pml4 = pml4_phys as *mut PageTable;
            (*pml4).entries.fill(0);
        }
        Some(AddressSpace { pml4_phys })
    }

    pub fn map(&mut self, virt: usize, phys: usize, flags: PageFlags) -> bool {
        let pml4_idx = (virt >> 39) & 0x1FF;
        let pdpt_idx = (virt >> 30) & 0x1FF;
        let pd_idx = (virt >> 21) & 0x1FF;
        let pt_idx = (virt >> 12) & 0x1FF;

        unsafe {
            let pml4 = self.pml4_phys as *mut PageTable;
            
            // Get or create PDPT
            if (*pml4).entries[pml4_idx] & 1 == 0 {
                let pdpt_phys = pmm::alloc_page().unwrap_or(0);
                if pdpt_phys == 0 { return false; }
                let pdpt = pdpt_phys as *mut PageTable;
                (*pdpt).entries.fill(0);
                (*pml4).entries[pml4_idx] = pdpt_phys as u64 | 0x3;
            }
            let pdpt = ((*pml4).entries[pml4_idx] & !0xFFF) as *mut PageTable;

            // Get or create PD
            if (*pdpt).entries[pdpt_idx] & 1 == 0 {
                let pd_phys = pmm::alloc_page().unwrap_or(0);
                if pd_phys == 0 { return false; }
                let pd = pd_phys as *mut PageTable;
                (*pd).entries.fill(0);
                (*pdpt).entries[pdpt_idx] = pd_phys as u64 | 0x3;
            }
            let pd = ((*pdpt).entries[pdpt_idx] & !0xFFF) as *mut PageTable;

            // Get or create PT
            if (*pd).entries[pd_idx] & 1 == 0 {
                let pt_phys = pmm::alloc_page().unwrap_or(0);
                if pt_phys == 0 { return false; }
                let pt = pt_phys as *mut PageTable;
                (*pt).entries.fill(0);
                (*pd).entries[pd_idx] = pt_phys as u64 | 0x3;
            }
            let pt = ((*pd).entries[pd_idx] & !0xFFF) as *mut PageTable;

            // Map page
            (*pt).entries[pt_idx] = phys as u64 | flags.bits();
        }
        true
    }

    pub fn activate(&self) {
        unsafe {
            core::arch::asm!(
                "mov cr3, {}",
                in(reg) self.pml4_phys,
                options(nostack, preserves_flags)
            );
        }
    }
}
