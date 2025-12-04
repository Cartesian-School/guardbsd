// kernel/arch/x86_64/mod.rs
// x86_64 architecture glue: long-mode entry stub and early init.

#![cfg(target_arch = "x86_64")]
#![allow(dead_code)]

pub mod interrupts;
pub mod boot;
pub mod time;
pub mod user_entry;

pub use user_entry::enter_user_mode;

/// Entry point reached from long_mode_entry.S after transitioning to 64-bit mode.
#[no_mangle]
pub extern "C" fn kernel_main_x86_64() -> ! {
    // Build & run user-mode smoke test:
    //   cargo build --release --features user_mode_test
    // Expected: jumps to dummy_user_entry() in user mode and spins.
    #[cfg(feature = "user_mode_test")]
    {
        crate::proc::test_enter_user_mode();
    }

    #[cfg(feature = "exec_boot_test")]
    {
        // This will call sys_exec("/bin/init") from kernel and drop to user mode.
        crate::syscall::kernel_exec_smoke_test();
    }

    #[cfg(not(feature = "exec_boot_test"))]
    {
        // Default boot path: exec /bin/init → /bin/gsh
        crate::syscall::sys_exec_entry();
    }

    // Initialize GDT/IDT (64-bit skeleton)
    interrupts::gdt64::init_gdt64();
    interrupts::idt64::init_idt64();

    // Spawn test threads
    #[cfg(target_arch = "x86_64")]
    {
        use crate::sched::{spawn_kernel_thread, start_first_thread};
        use crate::tests::preempt_threads::{thread_a, thread_b, thread_c, thread_d};

        spawn_kernel_thread(thread_a);
        spawn_kernel_thread(thread_b);
        spawn_kernel_thread(thread_c);
        spawn_kernel_thread(thread_d);

        start_first_thread();
    }

    loop {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)) };
    }
}
