// Interrupt Descriptor Table - x86_64
// BSD 3-Clause License

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

static mut IDT: [IdtEntry; 256] = [IdtEntry {
    offset_low: 0,
    selector: 0,
    ist: 0,
    flags: 0,
    offset_mid: 0,
    offset_high: 0,
    reserved: 0,
}; 256];

extern "C" {
    fn syscall_entry();
    fn keyboard_irq_handler();
    fn timer_irq_handler();
}

pub fn init_idt() {
    unsafe {
        // Set timer interrupt (IRQ0 = 0x20)
        set_idt_entry(0x20, timer_irq_handler as u64, 0x08, 0x8E);
        
        // Set keyboard interrupt (IRQ1 = 0x21)
        set_idt_entry(0x21, keyboard_irq_handler as u64, 0x08, 0x8E);
        
        // Set syscall interrupt (0x80)
        set_idt_entry(0x80, syscall_entry as u64, 0x08, 0xEE);
        
        // Load IDT
        let idtr = IdtPtr {
            limit: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
            base: &IDT as *const _ as u64,
        };
        
        core::arch::asm!("lidt [{}]", in(reg) &idtr, options(readonly, nostack, preserves_flags));
    }
}

unsafe fn set_idt_entry(index: usize, handler: u64, selector: u16, flags: u8) {
    IDT[index] = IdtEntry {
        offset_low: (handler & 0xFFFF) as u16,
        selector,
        ist: 0,
        flags,
        offset_mid: ((handler >> 16) & 0xFFFF) as u16,
        offset_high: ((handler >> 32) & 0xFFFFFFFF) as u32,
        reserved: 0,
    };
}
