// Memory Management Module
// BSD 3-Clause License

#![no_std]

pub mod pmm;
pub mod vmm;

pub fn init() {
    pmm::init();
}

pub use pmm::{alloc_page, free_page};
pub use vmm::{AddressSpace, PageFlags};

/// Clone an address space for fork()
/// Returns physical address of new page table, or None if allocation fails
pub fn clone_address_space(source_pml4: usize) -> Option<usize> {
    AddressSpace::clone(source_pml4)
}

/// Create a new empty address space for exec()
/// Returns physical address of new PML4, or None if allocation fails
pub fn create_address_space() -> Option<usize> {
    let addr_space = AddressSpace::new()?;
    Some(addr_space.pml4_phys())
}

/// Destroy an address space and free all pages (for exec())
/// Frees all user-space pages but preserves kernel mappings
pub fn destroy_address_space(pml4_phys: usize) {
    // TODO: Implement proper cleanup
    // For now, just free the PML4 page
    // Full implementation should walk all page tables and free:
    // - All user-space page tables (PDPT, PD, PT)
    // - All user-space data pages
    // - But preserve kernel space mappings
    free_page(pml4_phys);
}
