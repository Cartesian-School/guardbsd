// kernel/sched/context_switch.rs
// Architecture-specific context switch helpers (minimal stubs)
// BSD 3-Clause License

#![no_std]

use super::Context;

/// Switch from `old` to `new_ctx`.
/// This placeholder copies the target context; real kernels should save/restore
/// full register state with assembly and swap address spaces.
#[inline(never)]
pub unsafe fn context_switch(old: &mut Context, new_ctx: &Context) {
    core::ptr::copy_nonoverlapping(new_ctx, old as *mut Context, 1);
}
