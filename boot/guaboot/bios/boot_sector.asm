.intel_syntax noprefix
.intel_syntax noprefix
; Project: GuardBSD Winter Saga version 1.0.0
; Package: guaboot_bios
; Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
; License: BSD-3-Clause
;
; Boot sector start for GuaBoot BIOS.
.code16
.section .text
.globl _start

.set COM1,            0x3F8
.set STAGE2_SEGMENT,  0x0820
.set STAGE2_OFFSET,   0x0000
.set STAGE2_SECTORS,  1
.set STAGE2_LBA,      35          # isoinfo LBA for guaboot2.bin
.set SCRATCH_SEG,     0x9000
.set SCRATCH_OFF,     0x0000

_start:
    cli
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00
    sti

    call init_serial

    mov si, msg_boot
    call print_string

    mov [boot_drive], dl          # original DL (e.g., 0xFC)

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

    # TEMP: force CHS path immediately to validate CHS read works
    jmp chs_fallback

chs_fallback:
    mov si, msg_chs
    call print_string

    # CHS read on original DL
    mov dl, [boot_drive]
    mov ah, 0x08
    int 0x13
    jc disk_error

    mov bl, cl
    and bl, 0x3F            # sectors per track
    mov [spt], bl
    mov bl, dh
    inc bl                   # heads count = DH + 1
    mov [heads], bl
    cmp byte ptr [spt], 0
    jne .spt_ok
    mov byte ptr [spt], 32   # fallback geometry
.spt_ok:
    cmp byte ptr [heads], 0
    jne .heads_ok
    mov byte ptr [heads], 64
.heads_ok:

    # Convert LBA -> CHS (LBA fits in AX)
    mov ax, STAGE2_LBA
    xor dx, dx
    mov bl, [spt]
    div bl                   # AL=quotient (cyl*heads + head), AH=remainder (sector-1)
    mov cl, ah
    inc cl                   # sector = rem + 1
    mov ah, 0
    # AX now = quotient
    mov bl, [heads]
    xor dx, dx
    div bl                   # AL=cylinder, AH=head
    mov dh, ah               # DH=head
    mov ch, al               # CH=cylinder low
    xor ah, ah               # cylinder high bits zero for small LBA
    # CL already has sector

    # Read via AH=02h into scratch
    mov ax, SCRATCH_SEG
    mov es, ax
    mov bx, SCRATCH_OFF
    mov ah, 0x02
    mov al, STAGE2_SECTORS
    int 0x13
    jc disk_error
    cmp al, STAGE2_SECTORS
    jne disk_error

    mov si, msg_chs
    call print_string

    # Print first 4 bytes from scratch (expect 8B36BF82)
    mov si, msg_data
    call print_string
    mov bx, SCRATCH_OFF
    mov al, es:[bx]
    call print_hex8
    inc bx
    mov al, es:[bx]
    call print_hex8
    inc bx
    mov al, es:[bx]
    call print_hex8
    inc bx
    mov al, es:[bx]
    call print_hex8
    call newline

    # Copy scratch to stage2 destination
    push ds
    mov ax, SCRATCH_SEG
    mov ds, ax
    xor si, si
    mov ax, STAGE2_SEGMENT
    mov es, ax
    mov di, STAGE2_OFFSET
    mov cx, 256 * STAGE2_SECTORS   # words per sector
    rep movsw
    pop ds

stage2_jump:
    mov si, msg_jump
    call print_string
    jmp STAGE2_SEGMENT:STAGE2_OFFSET

disk_error:
    mov si, msg_err
    call print_string
    mov al, 'E'
    call print_char
    mov al, ah
    call print_hex8
    call newline
    jmp hang

hang:
    mov si, msg_halt
    call print_string
    cli
    hlt
    jmp hang

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
msg_data:   .asciz "D "
msg_chs:    .asciz "C\n"
msg_jump:   .asciz "J\n"
msg_err:    .asciz "X "
msg_halt:   .asciz "H\n"
boot_drive: .byte 0
spt:        .byte 0
heads:      .byte 0

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
