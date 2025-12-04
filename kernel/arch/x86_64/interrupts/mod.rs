// kernel/arch/x86_64/interrupts/mod.rs
// 64-bit interrupt support scaffolding.

#![cfg(target_arch = "x86_64")]

pub mod gdt64;
pub mod idt64;
pub mod syscall_isr;

use crate::sched::{self, ArchContext};
use crate::trapframe::TrapFrameX86_64;
pub use syscall_isr::x86_64_syscall_entry;

extern "C" {
    fn arch_context_switch(old: *mut ArchContext, new: *const ArchContext);
}

#[no_mangle]
pub extern "C" fn x86_64_timer_interrupt_handler(tf: &mut TrapFrameX86_64) {
    let mut ctx: ArchContext = tf.into();
    ctx.cr3 = read_cr3();

    let next = unsafe { sched::scheduler_handle_tick(0, &mut ctx as *mut _) };

    if next.is_null() {
        // No switch; propagate updated context back into trap frame
        *tf = TrapFrameX86_64::from(&ctx);
    } else {
        // Switch immediately; arch_context_switch will not return here
        unsafe {
            arch_context_switch(&mut ctx as *mut ArchContext, next);
        }
        unreachable!();
    }
}

#[inline(always)]
fn read_cr3() -> u64 {
    let val: u64;
    unsafe { core::arch::asm!("mov {}, cr3", out(reg) val, options(nomem, nostack, preserves_flags)) };
    val
}
