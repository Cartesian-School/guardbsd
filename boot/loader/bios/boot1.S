/*
 * GuardBSD Loader - BIOS Boot Stage 1
 * BSD 3-Clause License
 * Copyright (c) 2025, Cartesian School - Siergej Sobolewski
 */

.code16
.section .text
.global _start

_start:
    cli
    xor %ax, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %ss
    mov $0x7c00, %sp
    
    mov $msg_boot, %si
    call print_string
    
    # Load stage 2 from disk
    mov $0x02, %ah          # Read sectors
    mov $8, %al             # 8 sectors (4KB)
    mov $0x00, %ch          # Cylinder 0
    mov $0x02, %cl          # Sector 2
    mov $0x00, %dh          # Head 0
    mov $0x80, %dl          # Drive 0
    mov $0x7e00, %bx        # Load to 0x7e00
    int $0x13
    jc disk_error
    
    # Jump to stage 2
    ljmp $0x0000, $0x7e00

disk_error:
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

msg_boot:   .asciz "GuardBSD Loader...\r\n"
msg_error:  .asciz "Disk error\r\n"

.org 510
.word 0xaa55
