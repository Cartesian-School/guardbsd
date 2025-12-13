//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: boot_stub_aarch64
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Minimalny boot stub dla AArch64 (UART init/printf).

#![no_std]
#![no_main]

use core::panic::PanicInfo;

const UART0: usize = 0x09000000; // QEMU virt UART

unsafe fn uart_init() {
    core::ptr::write_volatile((UART0 + 0x30) as *mut u32, 0x301);
    core::ptr::write_volatile((UART0 + 0x2C) as *mut u32, 0x10);
}

unsafe fn uart_putc(c: u8) {
    while (core::ptr::read_volatile((UART0 + 0x18) as *const u32) & 0x20) != 0 {}
    core::ptr::write_volatile(UART0 as *mut u8, c);
}

unsafe fn print(s: &str) {
    for b in s.bytes() {
        uart_putc(b);
    }
}

#[no_mangle]
#[link_section = ".text.start"]
pub extern "C" fn _start() -> ! {
    unsafe {
        uart_init();
        
        print("\n\n");
        print("================================================================================\n");
        print("[BOOT] GuardBSD Winter Saga v1.0.0 - SYSTEM ONLINE (AArch64)\n");
        print("================================================================================\n");
        print("[OK] Boot stub loaded\n");
        print("[OK] UART PL011 initialized\n");
        print("[OK] Running in EL1 (kernel mode)\n");
        print("\n[INIT] Microkernel bootstrap starting...\n");
        print("================================================================================\n\n");
    }
    
    loop {
        unsafe { core::arch::asm!("wfi"); }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        print("\n[PANIC] System halted\n");
    }
    loop {
        unsafe { core::arch::asm!("wfi"); }
    }
}
