// kernel/arch/x86_64/interrupts/gdt64.rs
// Minimal 64-bit GDT setup for long mode.

#![cfg(target_arch = "x86_64")]

#[repr(C, packed)]
struct DescriptorTablePtr {
    limit: u16,
    base: u64,
}

#[repr(C, align(16))]
struct Gdt {
    null: u64,
    code: u64,
    data: u64,
}

static mut GDT64: Gdt = Gdt {
    null: 0,
    // Code: base=0, limit ignored in long mode, flags: present, ring0, code, readable, long
    code: 0x00af9a000000ffff,
    // Data: present, ring0, writable
    data: 0x00af92000000ffff,
};

pub fn init_gdt64() {
    unsafe {
        let ptr = DescriptorTablePtr {
            limit: core::mem::size_of::<Gdt>() as u16 - 1,
            base: &GDT64 as *const _ as u64,
        };
        core::arch::asm!("lgdt [{}]", in(reg) &ptr, options(readonly, nostack));

        // Reload segment selectors
        core::arch::asm!(
            "mov ax, {data}",
            "mov ds, ax",
            "mov es, ax",
            "mov ss, ax",
            "push {code}",
            "lea rax, [rip + 1f]",
            "push rax",
            "retfq",
            "1:",
            code = const 0x08u16,
            data = const 0x10u16,
            out("rax") _,
            options(nostack)
        );
    }
}
