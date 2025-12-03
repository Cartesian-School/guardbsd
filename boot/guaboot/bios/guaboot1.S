/*
 * GuaBoot Stage 1 - BIOS Bootloader
 * BSD 3-Clause License
 * 
 * First stage bootloader for GuardBSD
 * Fits in 512-byte boot sector
 */

.code16
.section .text
.globl _start

_start:
    cli
    xorw %ax, %ax
    movw %ax, %ds
    movw %ax, %es
    movw %ax, %ss
    movw $0x7c00, %sp
    sti
    
    /* Print boot message */
    movw $msg_boot, %si
    call print_string
    
    /* Load guaboot2 from disk */
    movw $0x0201, %ax       /* Read 1 sector */
    movw $0x0002, %cx       /* Cylinder 0, Sector 2 */
    movw $0x0000, %dx       /* Head 0, Drive 0 */
    movw $0x7e00, %bx       /* Load to 0x7e00 */
    int $0x13
    jc disk_error
    
    /* Jump to stage 2 */
    ljmp $0x0000, $0x7e00

disk_error:
    movw $msg_error, %si
    call print_string
    jmp halt

print_string:
    lodsb
    orb %al, %al
    jz print_done
    movb $0x0e, %ah
    int $0x10
    jmp print_string
print_done:
    ret

halt:
    hlt
    jmp halt

msg_boot:
    .asciz "GuaBoot1...\r\n"
msg_error:
    .asciz "Disk error\r\n"

/* Boot signature */
.org 510
.word 0xaa55
