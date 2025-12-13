; Project: GuardBSD Winter Saga version 1.0.0
; Package: guaboot_bios
; Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
; License: BSD-3-Clause
;
; GuaBoot stage2 loader (BIOS, 16-bit).
.intel_syntax noprefix
.code16
.section .text
.globl _start

.set KERNEL_LOAD_SEG,  0x1000
.set KERNEL_ENTRY,     0x100000  # 1MB (protected mode)
.set GDT_SEG,          0x0800

_start:
    mov si, msg_stage2
    call print_string

    mov si, msg_load_kernel
    call print_string
    call load_kernel_sectors

    # Enable A20 line
    call enable_a20

    # Load GDT
    call setup_gdt

    # Switch to protected mode
    cli
    mov eax, cr0
    or al, 1
    mov cr0, eax

    # Far jump to flush pipeline and enter 32-bit mode
    jmp 0x08:protected_mode

# Enable A20 gate using keyboard controller
enable_a20:
    mov si, msg_a20
    call print_string

    call wait_keyboard
    mov al, 0xAD        # Disable keyboard
    out 0x64, al

    call wait_keyboard
    mov al, 0xD0        # Read output port
    out 0x64, al

    call wait_keyboard_data
    in al, 0x60
    push ax

    call wait_keyboard
    mov al, 0xD1        # Write output port
    out 0x64, al

    call wait_keyboard
    pop ax
    or al, 2            # Enable A20
    out 0x60, al

    call wait_keyboard
    mov al, 0xAE        # Enable keyboard
    out 0x64, al

    call wait_keyboard
    ret

wait_keyboard:
    in al, 0x64
    test al, 2
    jnz wait_keyboard
    ret

wait_keyboard_data:
    in al, 0x64
    test al, 1
    jz wait_keyboard_data
    ret

# Load kernel from CD-ROM
load_kernel_sectors:
    mov ax, KERNEL_LOAD_SEG
    mov es, ax
    xor bx, bx

    mov ah, 0x42        # Extended read
    mov dl, 0x80        # Drive
    mov si, dap         # Disk address packet
    int 0x13
    jc load_error

    mov si, msg_ok
    call print_string
    ret

load_error:
    mov si, msg_error
    call print_string
    jmp $

# Setup GDT
setup_gdt:
    mov si, msg_gdt
    call print_string

    lgdt [gdt_descriptor]
    ret

# Print string (real mode)
print_string:
    pusha
.loop:
    lodsb
    or al, al
    jz .done
    mov ah, 0x0E
    int 0x10
    jmp .loop
.done:
    popa
    ret

.code32
protected_mode:
    # Setup segments
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov esp, 0x90000

    # Jump to kernel entry point
    jmp KERNEL_ENTRY

.code16

# Data
msg_stage2:      .asciz "Stage 2 Loader\r\n"
msg_load_kernel: .asciz "Loading kernel image..."
msg_a20:         .asciz "A20..."
msg_gdt:         .asciz "GDT..."
msg_ok:          .asciz "OK\r\n"
msg_error:       .asciz "ERROR\r\n"

# Disk Address Packet for INT 13h extended read
dap:
    .byte 0x10             # Size of packet (16 bytes)
    .byte 0                # Reserved (0)
    .word 128              # Number of sectors to read (64KB)
    .word 0                # Offset
    .word KERNEL_LOAD_SEG  # Segment
    .quad 16               # Starting LBA (sector 16)

# GDT (Global Descriptor Table)
.balign 8
gdt_start:
    .quad 0                # Null descriptor

    # Code segment (0x08)
    .word 0xFFFF           # Limit low
    .word 0x0000           # Base low
    .byte 0x00             # Base middle
    .byte 0x9A             # Access
    .byte 0xCF             # Flags
    .byte 0x00             # Base high

    # Data segment (0x10)
    .word 0xFFFF           # Limit low
    .word 0x0000           # Base low
    .byte 0x00             # Base middle
    .byte 0x92             # Access
    .byte 0xCF             # Flags
    .byte 0x00             # Base high

gdt_end:

gdt_descriptor:
    .word gdt_end - gdt_start - 1  # Size
    .long gdt_start                # Offset

# Pad to reasonable size (< 4KB)
.org 760
.byte 0
