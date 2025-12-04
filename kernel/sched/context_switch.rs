// kernel/sched/context_switch.rs
// FFI adapter to architecture assembly context switch
// BSD 3-Clause License

#![no_std]

use super::ArchContext;

extern "C" {
    fn arch_context_switch(old: *mut ArchContext, new: *const ArchContext);
}

#[inline(always)]
pub unsafe fn switch(old: &mut ArchContext, new_ctx: &ArchContext) {
    arch_context_switch(old as *mut _, new_ctx as *const _);
}
