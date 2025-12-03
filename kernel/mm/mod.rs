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
