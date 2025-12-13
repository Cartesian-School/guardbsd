//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_arch_x86_64
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Minimalna 64-bitowa IDT z wpisem timera na wektorze 0x20.

#![cfg(target_arch = "x86_64")]

#[repr(C, packed)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attr: u8,
    offset_mid: u16,
    offset_high: u32,
    zero: u32,
}

#[repr(C, packed)]
struct IdtPtr {
    limit: u16,
    base: u64,
}

// 256-entry IDT
static mut IDT: [IdtEntry; 256] = [IdtEntry {
    offset_low: 0,
    selector: 0,
    ist: 0,
    type_attr: 0,
    offset_mid: 0,
    offset_high: 0,
    zero: 0,
}; 256];

extern "C" {
    fn timer_isr64();
    fn syscall_isr64();
}

pub fn init_idt64() {
    unsafe {
        set_gate(0x20, timer_isr64 as u64, 0x08, 0x8E); // present, ring0, interrupt gate
        set_gate(0x80, syscall_isr64 as u64, 0x08, 0xEE); // present, ring3, interrupt gate

        let idtr = IdtPtr {
            limit: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
            base: &IDT as *const _ as u64,
        };

        core::arch::asm!("lidt [{}]", in(reg) &idtr, options(readonly, nostack));
    }
}

unsafe fn set_gate(vec: usize, handler: u64, selector: u16, type_attr: u8) {
    IDT[vec] = IdtEntry {
        offset_low: handler as u16,
        selector,
        ist: 0,
        type_attr,
        offset_mid: (handler >> 16) as u16,
        offset_high: (handler >> 32) as u32,
        zero: 0,
    };
}
