# Project: GuardBSD Winter Saga version 1.0.0
# Package: guaboot_bios
# Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
# License: BSD-3-Clause
#
# GuaBoot stage2 loader (BIOS, 16-bit).
.intel_syntax noprefix
.code16
.section .text
.globl _start

.set COM1,             0x3F8
.set KERNEL_LOAD_SEG,  0x1000        # -> 0x10000 phys
.set KERNEL_BYTES,       19656            # AUTO_PATCH KERNEL_BYTES (16 * 2048 headroom for kernel image)
.set KERNEL_PTR_ADDR,  0x7000        # where we stash kernel phys for loader
.set KERNEL_LBA,       56            # AUTO_PATCH KERNEL_LBA updated after ISO build
.set KERNEL_SECTORS,       10            # AUTO_PATCH KERNEL_SECTORS read full kernel.elf (ceil(size/2048))
.set LOADER_SEG,       0x0800        # 0x8000 phys
.set LOADER_OFFSET,    0x0000
.set LOADER_LBA,       49            # AUTO_PATCH LOADER_LBA updated after ISO build
.set LOADER_SECTORS,       7            # AUTO_PATCH LOADER_SECTORS ~8KB loader.bin
.set LOADER_LINEAR,    (LOADER_SEG << 4)
.set ENTRY64_SEG,      0x0F00        # 0x0000F000 phys (loader handoff stub, away from loader/BSS)
.set ENTRY64_OFFSET,   0x0000
.set ENTRY64_LBA,       46            # AUTO_PATCH ENTRY64_LBA updated after ISO build
.set ENTRY64_SECTORS,       1            # AUTO_PATCH ENTRY64_SECTORS stub is tiny
.set GDT_SEG,          0x0800
.set BOOT_DRIVE_PTR,   0x7D49        # shared with stage1 boot_drive

_start:
    mov ax, cs
    mov ds, ax
    mov es, ax

    call init_serial

    # Mask PIC until IDT is ready in kernel
    mov al, 0xFF
    out 0x21, al
    out 0xA1, al

    lea si, [msg_stage2]
    call print_string

    lea si, [msg_load_kernel]
    call print_string
    call load_kernel_sectors

    lea si, [msg_load_loader]
    call print_string
    call load_loader

    lea si, [msg_load_entry64]
    call print_string
    call load_entry64

    # Enable A20 line
    call enable_a20

    # Load GDT
    call setup_gdt

    # Switch to protected mode
    cli
    mov eax, cr0
    or al, 1
    mov cr0, eax

    # Far return into protected mode using linear target (cs<<4 + offset)
    xor ebx, ebx
    mov bx, cs
    shl ebx, 4
    lea eax, [protected_mode]
    add eax, ebx
    push 0x08
    push ax
    lret

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

    push ds
    xor ax, ax
    mov ds, ax
    mov dl, byte ptr [BOOT_DRIVE_PTR]
    pop ds
    cmp dl, 0
    jne .have_drive
    mov dl, 0xE0
.have_drive:

    mov ah, 0x42        # Extended read
    lea si, [dap_kernel]       # Disk address packet
    int 0x13
    jc load_error

    # stash physical address for loader at 0x9000
    mov ax, KERNEL_LOAD_SEG
    mov bx, 16
    mul bx               # DX:AX = phys address
    push ds
    mov si, ax
    mov di, dx
    xor ax, ax
    mov ds, ax
    mov word ptr [KERNEL_PTR_ADDR], si
    mov word ptr [KERNEL_PTR_ADDR+2], di
    pop ds

    lea si, [msg_ok]
    call print_string
    ret

load_loader:
    mov ax, LOADER_SEG
    mov es, ax
    xor bx, bx

    push ds
    xor ax, ax
    mov ds, ax
    mov dl, byte ptr [BOOT_DRIVE_PTR]
    pop ds
    cmp dl, 0
    jne .have_drive2
    mov dl, 0xE0
.have_drive2:

    mov ah, 0x42
    lea si, [dap_loader]
    int 0x13
    jc load_error

    lea si, [msg_ok]
    call print_string
    ret

load_entry64:
    mov ax, ENTRY64_SEG
    mov es, ax
    xor bx, bx

    push ds
    xor ax, ax
    mov ds, ax
    mov dl, byte ptr [BOOT_DRIVE_PTR]
    pop ds
    cmp dl, 0
    jne .have_drive3
    mov dl, 0xE0
.have_drive3:

    mov ah, 0x42
    lea si, [dap_entry64]
    int 0x13
    jc load_error

    lea si, [msg_ok]
    call print_string
    ret

load_error:
    lea si, [msg_error]
    call print_string
    jmp $

# Setup GDT
setup_gdt:
    lea si, [msg_gdt]
    call print_string

    # Patch GDT base with current linear address (CS<<4 + offset)
    push eax
    push ebx
    xor ebx, ebx
    mov bx, cs
    shl ebx, 4
    lea eax, [gdt_start]
    add eax, ebx
    mov dword ptr [gdt_descriptor + 2], eax
    pop ebx
    pop eax

    lgdt [gdt_descriptor]
    ret

# Print string (real mode)
print_string:
    pusha
.loop:
    lodsb
    or al, al
    jz .done
    call print_char
    jmp .loop
.done:
    popa
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

.code32
protected_mode:
    # Setup flat segments
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov esp, 0x90000

    # Jump to loader (32-bit flat); loader will jump to kernel
    push 0x08
    push LOADER_LINEAR
    lret
.code16

# Data
msg_stage2:      .asciz "Stage 2 Loader\r\n"
msg_load_kernel: .asciz "Loading kernel image..."
msg_load_loader: .asciz "Loading loader..."
msg_load_entry64:.asciz "Loading 64-bit stub..."
msg_a20:         .asciz "A20..."
msg_gdt:         .asciz "GDT..."
msg_ok:          .asciz "OK\r\n"
msg_error:       .asciz "ERROR\r\n"

# Disk Address Packet for INT 13h extended reads
dap_kernel:
    .byte 0x10             # Size of packet (16 bytes)
    .byte 0                # Reserved (0)
    .word KERNEL_SECTORS   # Number of sectors to read (matches KERNEL_BYTES)
    .word 0                # Offset
    .word KERNEL_LOAD_SEG  # Segment
    .quad KERNEL_LBA       # Starting LBA for /boot/kernel.elf

dap_loader:
    .byte 0x10
    .byte 0
    .word LOADER_SECTORS
    .word LOADER_OFFSET
    .word LOADER_SEG
    .quad LOADER_LBA

dap_entry64:
    .byte 0x10
    .byte 0
    .word ENTRY64_SECTORS
    .word ENTRY64_OFFSET
    .word ENTRY64_SEG
    .quad ENTRY64_LBA

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
