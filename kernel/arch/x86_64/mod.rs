// kernel/arch/x86_64/mod.rs
// x86_64 architecture glue: long-mode entry stub and early init.

#![cfg(target_arch = "x86_64")]
#![allow(dead_code)]

pub mod interrupts;
pub mod boot;
pub mod time;

/// Entry point reached from long_mode_entry.S after transitioning to 64-bit mode.
#[no_mangle]
pub extern "C" fn kernel_main_x86_64() -> ! {
    // Initialize GDT/IDT (64-bit skeleton)
    interrupts::gdt64::init_gdt64();
    interrupts::idt64::init_idt64();

    // TODO: bring up devices, memory, scheduler. For now, halt loop.
    loop {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)) };
    }
}
