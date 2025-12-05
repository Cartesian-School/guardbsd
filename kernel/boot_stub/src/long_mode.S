// Long Mode Transition Code
// BSD 3-Clause License

.code32

// GDT for long mode
.align 8
gdt64:
    .quad 0x0000000000000000  // Null descriptor
    .quad 0x0020980000000000  // Kernel code (64-bit)
    .quad 0x0000920000000000  // Kernel data (64-bit)

gdt64_ptr:
    .word gdt64_ptr - gdt64 - 1
    .long gdt64

// Page tables for identity mapping
.align 4096
pml4:
    .quad pml3 + 0x3
    .fill 511, 8, 0

pml3:
    .quad pml2 + 0x3
    .fill 511, 8, 0

pml2:
    .quad pml1 + 0x3
    .fill 511, 8, 0

pml1:
    .set addr, 0
    .rept 512
        .quad addr + 0x3
        .set addr, addr + 0x1000
    .endr

.global enter_long_mode
enter_long_mode:
    // Save return address
    pop %esi

    // Enable PAE
    mov %cr4, %eax
    or $(1 << 5), %eax
    mov %eax, %cr4

    // Load page table root
    mov $pml4, %eax
    mov %eax, %cr3

    // Enable long mode
    mov $0xC0000080, %ecx
    rdmsr
    or $(1 << 8), %eax
    wrmsr

    // Enable paging and protected mode
    mov %cr0, %eax
    or $(1 << 31 | 1 << 0), %eax
    mov %eax, %cr0

    // Load 64-bit GDT
    lgdt gdt64_ptr

    // Jump to 64-bit code
    ljmp $0x8, $long_mode_start

.code64
long_mode_start:
    // Set up segment registers
    mov $0x10, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %fs
    mov %ax, %gs
    mov %ax, %ss

    // Jump to Rust long mode entry
    push %rsi
    ret

.global is_long_mode_supported
is_long_mode_supported:
    // Check if CPUID is supported
    pushfl
    popl %eax
    movl %eax, %ecx
    xorl $(1 << 21), %eax
    pushl %eax
    popfl
    pushfl
    popl %eax
    pushl %ecx
    popfl
    xorl %ecx, %eax
    jz .no_long_mode

    // Check extended CPUID functions
    movl $0x80000000, %eax
    cpuid
    cmpl $0x80000001, %eax
    jb .no_long_mode

    // Check long mode support
    movl $0x80000001, %eax
    cpuid
    testl $(1 << 29), %edx
    jz .no_long_mode

    movl $1, %eax
    ret

.no_long_mode:
    xorl %eax, %eax
    ret
