/* 
 * Project: GuardBSD Winter Saga version 1.0.0
 * Package: kernel_arch_x86_64
 * Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
 * License: BSD-3-Clause
 *
 * Wejście do long mode: włącza PAE/paging, instaluje GDT i skacze do Rust.
 */

.intel_syntax noprefix
.section .text
.global long_mode_entry
.type long_mode_entry, @function

long_mode_entry:
    cli
    
    // =========== DEBUG: Powiadom że weszliśmy ===========
    mov dx, 0x3F8
    mov al, 'L'
    out dx, al
    mov al, 'M'
    out dx, al
    mov al, ' '
    out dx, al
    // ===================================================

    // Enable PAE
    mov eax, cr4
    or eax, 1 << 5          // CR4.PAE
    mov cr4, eax

    // Point CR3 to PML4 (bootloader powinien dać w EBX)
    mov eax, ebx
    mov cr3, eax

    // Enable Long Mode by setting EFER.LME
    mov ecx, 0xC0000080     // IA32_EFER
    rdmsr
    or eax, 1 << 8          // LME
    wrmsr

    // Enable paging and protection
    mov eax, cr0
    or eax, (1 << 31) | (1 << 0)  // PG | PE
    mov cr0, eax

    // Load 64-bit GDT
    lgdt [gdt64_descriptor]
    
    // =========== DEBUG przed skokiem ===========
    mov dx, 0x3F8
    mov al, 'J'
    out dx, al
    mov al, 'U'
    out dx, al
    mov al, 'M'
    out dx, al
    mov al, 'P'
    out dx, al
    mov al, 0x0A
    out dx, al
    // ===========================================

    // Far jump to 64-bit code segment
    jmp 0x08:long_mode_start64

// Teraz 64-bit code
.code64
long_mode_start64:
    // Set up data segments
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    // Set up stack
    mov rsp, offset long_mode_stack_top
    mov rbp, rsp
    
    // =========== DEBUG w 64-bit ===========
    mov dx, 0x3F8
    mov al, '6'
    out dx, al
    mov al, '4'
    out dx, al
    mov al, 'B'
    out dx, al
    mov al, 'I'
    out dx, al
    mov al, 'T'
    out dx, al
    mov al, 0x0A
    out dx, al
    // =====================================

    // Zero out .bss
    mov rdi, offset __bss_start
    mov rcx, offset __bss_end
    sub rcx, rdi
    xor rax, rax
    rep stosb

    // Call Rust entry
    extern kernel_main_x86_64
    call kernel_main_x86_64

    // If returns, halt forever
1:  hlt
    jmp 1b

.align 8
gdt64:
    .quad 0x0000000000000000
    .quad 0x00af9a000000ffff  // Code
    .quad 0x00af92000000ffff  // Data

gdt64_descriptor:
    .word gdt64_descriptor_end - gdt64 - 1
    .quad gdt64
gdt64_descriptor_end:

.comm long_mode_stack, 0x4000, 16
long_mode_stack_top:
    .quad long_mode_stack + 0x4000

.extern __bss_start
.extern __bss_end

.size long_mode_entry, .-long_mode_entry