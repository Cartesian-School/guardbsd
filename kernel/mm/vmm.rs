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

// Kernel PML4 template that holds the kernel half of the address space.
static mut KERNEL_PML4_TEMPLATE: usize = 0;
static mut KERNEL_TEMPLATE_INITIALIZED: bool = false;

impl AddressSpace {
    pub fn new() -> Option<Self> {
        let pml4_phys = pmm::alloc_page()?;
        unsafe {
            let pml4 = pml4_phys as *mut PageTable;
            (*pml4).entries.fill(0);
        }
        Some(AddressSpace { pml4_phys })
    }

    /// Initialize a new address space with kernel mappings cloned into the upper half.
    pub fn new_with_kernel_mappings() -> Option<Self> {
        let mut addr_space = Self::new()?;
        unsafe {
            if KERNEL_PML4_TEMPLATE == 0 {
                return None;
            }
            let dst = addr_space.pml4_phys as *mut PageTable;
            let src = KERNEL_PML4_TEMPLATE as *const PageTable;
            // Copy entries 256..512 (upper half) so kernel remains mapped.
            for i in 256..ENTRIES {
                (*dst).entries[i] = (*src).entries[i];
            }
        }
        Some(addr_space)
    }
    
    /// Get the physical address of the PML4
    pub fn pml4_phys(&self) -> usize {
        self.pml4_phys
    }
    
    /// Clone an address space (for fork())
    /// Creates a new page table hierarchy and copies all mappings
    /// 
    /// Note: This performs a full copy of all pages.
    /// Future enhancement: Implement Copy-on-Write (COW) for better performance
    /// 
    /// Returns: Physical address of new PML4, or None if allocation fails
    pub fn clone(source_pml4: usize) -> Option<usize> {
        // Allocate new PML4
        let new_pml4_phys = pmm::alloc_page()?;
        
        unsafe {
            let src_pml4 = source_pml4 as *const PageTable;
            let dst_pml4 = new_pml4_phys as *mut PageTable;
            
            // Initialize destination
            (*dst_pml4).entries.fill(0);
            
            // Copy each PML4 entry
            for pml4_idx in 0..ENTRIES {
                let src_entry = (*src_pml4).entries[pml4_idx];
                
                // Skip non-present entries and kernel space (top half)
                if (src_entry & 1) == 0 || pml4_idx >= 256 {
                    continue;
                }
                
                // Clone PDPT
                let src_pdpt_phys = (src_entry & !0xFFF) as usize;
                if let Some(new_pdpt_phys) = Self::clone_pdpt(src_pdpt_phys) {
                    (*dst_pml4).entries[pml4_idx] = new_pdpt_phys as u64 | (src_entry & 0xFFF);
                } else {
                    // Allocation failed, cleanup and return None
                    // TODO: Add proper cleanup of partially allocated structures
                    return None;
                }
            }
        }
        
        Some(new_pml4_phys)
    }
    
    /// Clone a PDPT (Page Directory Pointer Table)
    unsafe fn clone_pdpt(source_pdpt: usize) -> Option<usize> {
        let new_pdpt_phys = pmm::alloc_page()?;
        let src_pdpt = source_pdpt as *const PageTable;
        let dst_pdpt = new_pdpt_phys as *mut PageTable;
        
        (*dst_pdpt).entries.fill(0);
        
        for pdpt_idx in 0..ENTRIES {
            let src_entry = (*src_pdpt).entries[pdpt_idx];
            if (src_entry & 1) == 0 {
                continue;
            }
            
            let src_pd_phys = (src_entry & !0xFFF) as usize;
            if let Some(new_pd_phys) = Self::clone_pd(src_pd_phys) {
                (*dst_pdpt).entries[pdpt_idx] = new_pd_phys as u64 | (src_entry & 0xFFF);
            } else {
                return None;
            }
        }
        
        Some(new_pdpt_phys)
    }
    
    /// Clone a PD (Page Directory)
    unsafe fn clone_pd(source_pd: usize) -> Option<usize> {
        let new_pd_phys = pmm::alloc_page()?;
        let src_pd = source_pd as *const PageTable;
        let dst_pd = new_pd_phys as *mut PageTable;
        
        (*dst_pd).entries.fill(0);
        
        for pd_idx in 0..ENTRIES {
            let src_entry = (*src_pd).entries[pd_idx];
            if (src_entry & 1) == 0 {
                continue;
            }
            
            let src_pt_phys = (src_entry & !0xFFF) as usize;
            if let Some(new_pt_phys) = Self::clone_pt(src_pt_phys) {
                (*dst_pd).entries[pd_idx] = new_pt_phys as u64 | (src_entry & 0xFFF);
            } else {
                return None;
            }
        }
        
        Some(new_pd_phys)
    }
    
    /// Clone a PT (Page Table) and copy all page contents
    unsafe fn clone_pt(source_pt: usize) -> Option<usize> {
        let new_pt_phys = pmm::alloc_page()?;
        let src_pt = source_pt as *const PageTable;
        let dst_pt = new_pt_phys as *mut PageTable;
        
        (*dst_pt).entries.fill(0);
        
        for pt_idx in 0..ENTRIES {
            let src_entry = (*src_pt).entries[pt_idx];
            if (src_entry & 1) == 0 {
                continue;
            }
            
            // Allocate new physical page for content
            let new_page_phys = pmm::alloc_page()?;
            
            // Copy page content
            let src_page = ((src_entry & !0xFFF) as usize) as *const u8;
            let dst_page = new_page_phys as *mut u8;
            core::ptr::copy_nonoverlapping(src_page, dst_page, PAGE_SIZE);
            
            // Set entry with same flags
            (*dst_pt).entries[pt_idx] = new_page_phys as u64 | (src_entry & 0xFFF);
        }
        
        Some(new_pt_phys)
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

/// Initialize the kernel PML4 template from the currently active CR3.
/// This must be called after paging is enabled and kernel mappings are stable.
pub unsafe fn init_kernel_template() {
    let cr3_val: usize;
    core::arch::asm!("mov {}, cr3", out(reg) cr3_val, options(nomem, preserves_flags));
    KERNEL_PML4_TEMPLATE = cr3_val & !0xFFF;
    KERNEL_TEMPLATE_INITIALIZED = true;
}

/// Returns true if the kernel template has been recorded.
pub fn kernel_template_ready() -> bool {
    unsafe { KERNEL_TEMPLATE_INITIALIZED && KERNEL_PML4_TEMPLATE != 0 }
}
