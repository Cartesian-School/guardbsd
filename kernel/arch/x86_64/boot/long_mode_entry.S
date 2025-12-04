// kernel/arch/x86_64/boot/long_mode_entry.S
// 64-bit long mode entry: enable paging/PAE/long mode, install GDT, jump to Rust.

.intel_syntax noprefix
.section .text
.global long_mode_entry
.type long_mode_entry, @function

// This entry assumes we were entered from 32-bit protected mode with paging off.
// It enables PAE, long mode, paging, sets up a temporary stack, and jumps to
// kernel_main_x86_64 in 64-bit mode.

long_mode_entry:
    cli

    // Enable PAE
    mov eax, cr4
    or eax, 1 << 5          // CR4.PAE
    mov cr4, eax

    // Point CR3 to an identity-mapped PML4 (assumed provided by bootloader)
    // Placeholder: bootloader must set up minimal page tables; use symbol if exported.
    // For now, expect bootloader to place PML4 physical address in ebx.
    mov eax, ebx
    mov cr3, eax

    // Enable Long Mode by setting EFER.LME
    mov ecx, 0xC0000080     // IA32_EFER
    rdmsr
    or eax, 1 << 8          // LME
    wrmsr

    // Enable paging and protection
    mov eax, cr0
    or eax, 1 << 31         // PG
    or eax, 1 << 0          // PE
    mov cr0, eax

    // Load 64-bit GDT
    lgdt [gdt64_descriptor]

    // Far jump to 64-bit code segment to switch to long mode
    pushq $0x08             // 64-bit code segment selector
    lea rax, [rip + long_mode_start]
    push rax
    retfq

.code64
long_mode_start:
    // Set up data segments
    mov ax, 0x10            // 64-bit data segment selector
    mov ds, ax
    mov es, ax
    mov ss, ax

    // Set up stack (bootloader should supply stack phys in a register; fallback to symbol)
    lea rsp, [rip + long_mode_stack_top]

    // Call Rust entry
    extern kernel_main_x86_64
    call kernel_main_x86_64

    // If returns, halt forever
1:  hlt
    jmp 1b

.align 8
gdt64:
    // Null descriptor
    .quad 0x0000000000000000
    // Code: base=0 limit=0xfffff, flags: present, ring0, executable, readable, long mode
    .quad 0x00af9a000000ffff
    // Data: base=0 limit=0xfffff, flags: present, ring0, writable
    .quad 0x00af92000000ffff

gdt64_descriptor:
    .word gdt64_descriptor_end - gdt64 - 1
    .quad gdt64
gdt64_descriptor_end:

.comm long_mode_stack, 0x4000, 16
long_mode_stack_top:
    .quad long_mode_stack + 0x4000

.size long_mode_entry, .-long_mode_entry
