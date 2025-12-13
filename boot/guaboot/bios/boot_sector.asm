.intel_syntax noprefix
# Project: GuardBSD Winter Saga version 1.0.0
# Package: guaboot_bios
# Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
# License: BSD-3-Clause
#
# Boot sector start for GuaBoot BIOS.
.code16
.section .text
.globl _start

.set COM1,            0x3F8
.set STAGE2_SEGMENT,  0x0820
.set STAGE2_OFFSET,   0x0000
.set STAGE2_SECTORS,  1
.set STAGE2_LBA,      50          # xorriso LBA for /boot/guaboot/guaboot2.bin (2048-byte sector)

_start:
    cli
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00
    sti

    mov [boot_drive], dl          # original DL (e.g., 0xFC)

    call init_serial

    lea si, [msg_boot]
    call print_string

    # Print original DL
    mov al, 'o'
    call print_char
    mov al, 'D'
    call print_char
    mov al, 'L'
    call print_char
    mov al, '='
    call print_char
    mov al, dl
    call print_hex8
    call newline

    # Try LBA read with BIOS-provided drive, then common CD drive numbers
    mov dl, [boot_drive]
    call try_lba
    jnc stage2_jump

    mov dl, 0x00
    call try_lba
    jnc stage2_jump

    mov dl, 0x80
    call try_lba
    jnc stage2_jump

    mov dl, 0x9F
    call try_lba
    jnc stage2_jump

    mov dl, 0xE0
    call try_lba
    jnc stage2_jump

    jmp disk_error

stage2_jump:
    lea si, [msg_jump]
    call print_string
    jmp STAGE2_SEGMENT:STAGE2_OFFSET

disk_error:
    lea si, [msg_err]
    call print_string
    mov al, 'E'
    call print_char
    mov al, ah
    call print_hex8
    call newline
    jmp hang

hang:
    lea si, [msg_halt]
    call print_string
    cli
    hlt
    jmp hang

try_lba:
    lea si, [msg_lba]
    call print_string
    mov al, dl
    call print_hex8
    mov al, ':'
    call print_char

    lea si, [dap]
    mov ah, 0x42                 # extended disk read (no-emulation CD)
    int 0x13
    mov bl, ah
    jc .fail

    lea si, [msg_ok]
    call print_string
    clc
    ret

.fail:
    mov al, '!'
    call print_char
    mov al, bl
    call print_hex8
    call newline
    stc
    ret

init_serial:
    mov dx, COM1 + 1
    mov al, 0x00         # Disable interrupts
    out dx, al
    mov dx, COM1 + 3
    mov al, 0x80         # Enable DLAB
    out dx, al
    mov dx, COM1 + 0
    mov al, 0x03         # Divisor low (38400 baud)
    out dx, al
    mov dx, COM1 + 1
    mov al, 0x00         # Divisor high
    out dx, al
    mov dx, COM1 + 3
    mov al, 0x03         # 8N1, disable DLAB
    out dx, al
    mov dx, COM1 + 2
    mov al, 0xC7         # Enable FIFO
    out dx, al
    mov dx, COM1 + 4
    mov al, 0x0B         # IRQs enabled, RTS/DSR set
    out dx, al
    ret

print_string:
    pusha
.ps_loop:
    lodsb
    or al, al
    jz .ps_done
    call print_char
    jmp .ps_loop
.ps_done:
    popa
    ret

print_char:
    push dx
    push ax
.tx_wait:
    mov dx, COM1 + 5
    in al, dx
    test al, 0x20
    jz .tx_wait
    mov dx, COM1
    pop ax
    out dx, al
    pop dx
    ret

print_hex_nibble:
    and al, 0x0F
    cmp al, 0x0A
    jb .ph_digit
    add al, 'A' - 10
    jmp .ph_emit
.ph_digit:
    add al, '0'
.ph_emit:
    call print_char
    ret

print_hex8:
    push ax
    mov ah, al
    shr ah, 4
    mov al, ah
    call print_hex_nibble
    pop ax
    call print_hex_nibble
    ret

newline:
    mov al, 13
    call print_char
    mov al, 10
    call print_char
    ret

# Data
msg_boot:   .asciz "GB1\n"
msg_lba:    .asciz "L\n"
msg_jump:   .asciz "J\n"
msg_err:    .asciz "X "
msg_halt:   .asciz "H\n"
msg_ok:     .asciz "OK\n"
boot_drive: .byte 0

# Disk Address Packet for INT 13h extended read (stage2)
dap:
    .byte 0x10
    .byte 0
    .word STAGE2_SECTORS
    .word STAGE2_OFFSET
    .word STAGE2_SEGMENT
    .long STAGE2_LBA
    .long 0

# Boot signature must be at offset 510
.org 510
.word 0xAA55
