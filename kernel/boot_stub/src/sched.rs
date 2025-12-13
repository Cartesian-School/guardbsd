//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: boot_stub
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Minimalne stuby planisty wymagane do budowy (bez realnego schedulera).

#[derive(Clone, Copy)]
pub struct ArchContext {
    pub rip: u64,
    pub rsp: u64,
    pub rflags: u64,
    pub cs: u64,
    pub ss: u64,
    pub cr3: u64,
}

impl ArchContext {
    pub fn zeroed() -> Self {
        Self {
            rip: 0,
            rsp: 0,
            rflags: 0,
            cs: 0,
            ss: 0,
            cr3: 0,
        }
    }
}

pub fn yield_current() {}
pub fn on_tick(_cpu: u8) {}
pub fn init(_hz: u32) {}

pub fn register_thread(_pid: i32, _tid: i32, _prio: i32, _ctx: ArchContext) -> Option<usize> {
    None
}
