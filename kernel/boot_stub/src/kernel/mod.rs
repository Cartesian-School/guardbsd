//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: boot_stub
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Minimalne stuby przestrzeni nazw kernel potrzebne w boot stubie.
pub mod mm {
    bitflags::bitflags! {
        pub struct PageFlags: u64 {
            const PRESENT = 1 << 0;
            const WRITABLE = 1 << 1;
            const USER = 1 << 2;
        }
    }

    pub struct AddressSpace {
        pml4: u64,
    }

    impl AddressSpace {
        pub fn new_with_kernel_mappings() -> Self {
            AddressSpace { pml4: 0 }
        }
        pub fn pml4_phys(&self) -> u64 {
            self.pml4
        }
        pub fn map_page(&mut self, _virt: u64, _phys: u64, _flags: PageFlags) -> Result<(), ()> {
            Ok(())
        }
        pub fn map(&mut self, _virt: u64, _phys: u64, _flags: PageFlags) -> bool {
            true
        }
    }

    pub fn alloc_page() -> Option<u64> {
        Some(0x1000)
    }
}

pub mod syscalls {
    pub mod process_jobctl {
        pub fn find_process_mut_for_signal(_pid: usize) -> Option<usize> {
            None
        }
    }
}

pub mod signal {
    pub struct SignalFrame;
}

pub mod process {
    pub mod process {
        pub fn switch_to(_pid: usize) {}
        pub fn get_current() -> Option<usize> {
            Some(1)
        }
        pub fn try_add_memory_usage(_pid: usize, _pages: usize) -> bool {
            true
        }
        pub fn mark_killed(_pid: usize) {}
        pub fn create_process(_entry: u64, _stack: u64, _pml4: u64) -> usize {
            1
        }
    }
}
