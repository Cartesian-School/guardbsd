/*
 * GuardBSD Loader - BIOS Boot Stage 2
 * BSD 3-Clause License
 * Copyright (c) 2025, Cartesian School - Siergej Sobolewski
 */

.code16
.section .text
.global _start

_start:
    mov $msg_stage2, %si
    call print_string
    
    # Enable A20 line
    call enable_a20
    
    # Load kernel
    mov $msg_loading, %si
    call print_string
    
    # Load kernel from disk (sector 16, 128 sectors = 64KB)
    mov $0x02, %ah
    mov $64, %al
    mov $0x00, %ch
    mov $16, %cl
    mov $0x00, %dh
    mov $0x80, %dl
    mov $0x1000, %bx
    mov %bx, %es
    xor %bx, %bx
    int $0x13
    jc load_error
    
    # Enter protected mode
    cli
    lgdt gdt_descriptor
    mov %cr0, %eax
    or $1, %eax
    mov %eax, %cr0
    ljmp $0x08, $protected_mode

enable_a20:
    in $0x92, %al
    or $2, %al
    out %al, $0x92
    ret

load_error:
    mov $msg_error, %si
    call print_string
    jmp halt

print_string:
    lodsb
    test %al, %al
    jz print_done
    mov $0x0e, %ah
    int $0x10
    jmp print_string
print_done:
    ret

halt:
    hlt
    jmp halt

.code32
protected_mode:
    mov $0x10, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %fs
    mov %ax, %gs
    mov %ax, %ss
    mov $0x90000, %esp
    
    # Jump to kernel
    ljmp $0x08, $0x10000

msg_stage2:  .asciz "Stage 2\r\n"
msg_loading: .asciz "Loading kernel...\r\n"
msg_error:   .asciz "Load error\r\n"

.align 8
gdt_start:
    .quad 0x0000000000000000    # Null descriptor
    .quad 0x00cf9a000000ffff    # Code segment
    .quad 0x00cf92000000ffff    # Data segment
gdt_end:

gdt_descriptor:
    .word gdt_end - gdt_start - 1
    .long gdt_start
