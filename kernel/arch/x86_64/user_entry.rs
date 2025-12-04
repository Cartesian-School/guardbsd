// kernel/arch/x86_64/user_entry.rs
#![no_std]

/// User mode segment selectors (temporary values; align with GDT layout)
pub const USER_CS: u64 = 0x1B;
pub const USER_DS: u64 = 0x23;

/// Enter user mode at `entry` with user stack pointer `user_sp`.
/// Safety: caller must ensure valid user mappings and GDT/TSS setup.
pub unsafe fn enter_user_mode(entry: u64, user_sp: u64) -> ! {
    const RFLAGS_IF: u64 = 1 << 9;
    core::arch::asm!(
        "push {user_ds}",
        "push {user_sp}",
        "push {rflags}",
        "push {user_cs}",
        "push {entry}",
        "iretq",
        user_ds = const USER_DS,
        user_cs = const USER_CS,
        user_sp = in(reg) user_sp,
        rflags  = in(reg) RFLAGS_IF,
        entry   = in(reg) entry,
        options(noreturn, preserves_flags),
    );
}
