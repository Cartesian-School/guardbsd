//! kernel/boot_stub/src/interrupt/idt.rs
//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: boot_stub
//! Copyright © 2025 Cartesian School.
//! License: BSD-3-Clause
//!
//! x86_64 Interrupt Descriptor Table (IDT) for the boot stub.
//!
//! Minimal policy:
//! - Install only: IRQ0 (timer), IRQ1 (keyboard), INT 0x80 (syscall gate).
//!
//! Optional debugging policy (feature = "idt_exceptions"):
//! - Install exception vectors 0..31 using per-vector ASM stubs (exc_stub_0..exc_stub_31)
//!   that jump to common_exception_handler. This avoids triple-fault and helps early debug.
//!
//! Notes:
//! - IST is set to 0 for all entries because boot stub does not yet maintain IST stacks.

#![cfg(target_arch = "x86_64")]

use core::mem;

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    flags: u8,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

#[repr(C, packed)]
struct IdtPtr {
    limit: u16,
    base: u64,
}

const IDT_ENTRIES: usize = 256;

// -----------------------------------------------------------------------------
// GDT selectors
// -----------------------------------------------------------------------------
// Hard-coded for now: matches GDT layout in `guaboot_entry.S`.
const KERNEL_CS: u16 = 0x08;

// -----------------------------------------------------------------------------
// Gate flags
// -----------------------------------------------------------------------------
// 0x8E = Present | DPL=0 | Interrupt Gate (type=0xE)
// 0xEE = Present | DPL=3 | Interrupt Gate (type=0xE)
const FLAG_INTGATE_DPL0: u8 = 0x8E;
const FLAG_INTGATE_DPL3: u8 = 0xEE;

const IRQ0_VECTOR: usize = 0x20;
const IRQ1_VECTOR: usize = 0x21;
const SYSCALL_VECTOR: usize = 0x80;

static mut IDT: [IdtEntry; IDT_ENTRIES] = [IdtEntry {
    offset_low: 0,
    selector: 0,
    ist: 0,
    flags: 0,
    offset_mid: 0,
    offset_high: 0,
    reserved: 0,
}; IDT_ENTRIES];

extern "C" {
    fn keyboard_irq_handler();
    fn timer_irq_handler();
    fn syscall_entry();

    // --- REPLACED BLOCK AS REQUESTED ---
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_0();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_1();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_2();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_3();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_4();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_5();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_6();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_7();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_8();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_9();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_10();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_11();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_12();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_13();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_14();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_15();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_16();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_17();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_18();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_19();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_20();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_21();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_22();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_23();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_24();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_25();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_26();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_27();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_28();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_29();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_30();
    #[cfg(feature = "idt_exceptions")]
    fn exc_stub_31();
}

/// Initialize IDT entries used in boot stub and load IDTR.
pub fn init_idt() {
    unsafe {
        #[cfg(feature = "idt_exceptions")]
        {
            set_idt_entry(0,  exc_stub_0  as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(1,  exc_stub_1  as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(2,  exc_stub_2  as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(3,  exc_stub_3  as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(4,  exc_stub_4  as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(5,  exc_stub_5  as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(6,  exc_stub_6  as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(7,  exc_stub_7  as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(8,  exc_stub_8  as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(9,  exc_stub_9  as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(10, exc_stub_10 as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(11, exc_stub_11 as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(12, exc_stub_12 as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(13, exc_stub_13 as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(14, exc_stub_14 as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(15, exc_stub_15 as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(16, exc_stub_16 as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(17, exc_stub_17 as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(18, exc_stub_18 as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(19, exc_stub_19 as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(20, exc_stub_20 as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(21, exc_stub_21 as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(22, exc_stub_22 as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(23, exc_stub_23 as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(24, exc_stub_24 as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(25, exc_stub_25 as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(26, exc_stub_26 as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(27, exc_stub_27 as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(28, exc_stub_28 as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(29, exc_stub_29 as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(30, exc_stub_30 as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
            set_idt_entry(31, exc_stub_31 as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);
        }

        // IRQ0 (timer)
        set_idt_entry(IRQ0_VECTOR, timer_irq_handler as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);

        // IRQ1 (keyboard)
        set_idt_entry(IRQ1_VECTOR, keyboard_irq_handler as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL0);

        // INT 0x80 syscall gate (DPL=3)
        set_idt_entry(SYSCALL_VECTOR, syscall_entry as usize as u64, KERNEL_CS, FLAG_INTGATE_DPL3);

        let idtr = IdtPtr {
            limit: (mem::size_of::<[IdtEntry; IDT_ENTRIES]>() - 1) as u16,
            base: (&IDT as *const _ as u64),
        };

        core::arch::asm!(
            "lidt [{}]",
            in(reg) &idtr,
            options(readonly, nostack, preserves_flags)
        );
    }
}

#[inline(always)]
unsafe fn set_idt_entry(index: usize, handler: u64, selector: u16, flags: u8) {
    if index >= IDT_ENTRIES {
        return;
    }

    IDT[index] = IdtEntry {
        offset_low: (handler & 0xFFFF) as u16,
        selector,
        // IST=0: boot stub does not use dedicated interrupt stacks yet.
        ist: 0,
        flags,
        offset_mid: ((handler >> 16) & 0xFFFF) as u16,
        offset_high: ((handler >> 32) & 0xFFFF_FFFF) as u32,
        reserved: 0,
    };
}
