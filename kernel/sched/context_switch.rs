//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_sched
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Adapter FFI do asemblerowego przełączania kontekstu.

#![no_std]

use super::ArchContext;

extern "C" {
    fn arch_context_switch(old: *mut ArchContext, new: *const ArchContext);
}

#[inline(always)]
pub unsafe fn switch(old: &mut ArchContext, new_ctx: &ArchContext) {
    arch_context_switch(old as *mut _, new_ctx as *const _);
}
