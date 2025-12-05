; GuardBSD Stage 2 Bootloader
; BSD 3-Clause License
; Copyright (c) 2025, Cartesian School - Siergej Sobolewski
;
; Stage 2 bootloader - loads kernel and microkernels

BITS 16
ORG 0x8200

%define KERNEL_LOAD_SEG  0x1000
%define KERNEL_ENTRY     0x100000  ; 1MB (protected mode)
%define GDT_SEG          0x0800

start:
    ; Print stage 2 banner
    mov si, msg_stage2
    call print_string
    
    ; Load kernel to 0x10000
    mov si, msg_load_kernel
    call print_string
    call load_kernel_sectors
    
    ; Enable A20 line
    call enable_a20
    
    ; Load GDT
    call setup_gdt
    
    ; Switch to protected mode
    cli
    mov eax, cr0
    or al, 1
    mov cr0, eax
    
    ; Far jump to flush pipeline and enter 32-bit mode
    jmp 0x08:protected_mode

; Enable A20 gate using keyboard controller
enable_a20:
    mov si, msg_a20
    call print_string
    
    call wait_keyboard
    mov al, 0xAD        ; Disable keyboard
    out 0x64, al
    
    call wait_keyboard
    mov al, 0xD0        ; Read output port
    out 0x64, al
    
    call wait_keyboard_data
    in al, 0x60
    push ax
    
    call wait_keyboard
    mov al, 0xD1        ; Write output port
    out 0x64, al
    
    call wait_keyboard
    pop ax
    or al, 2            ; Enable A20
    out 0x60, al
    
    call wait_keyboard
    mov al, 0xAE        ; Enable keyboard
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

; Load kernel from CD-ROM
load_kernel_sectors:
    mov ax, KERNEL_LOAD_SEG
    mov es, ax
    xor bx, bx
    
    mov ah, 0x42        ; Extended read
    mov dl, 0x80        ; Drive
    mov si, dap         ; Disk address packet
    int 0x13
    jc load_error
    
    mov si, msg_ok
    call print_string
    ret

load_error:
    mov si, msg_error
    call print_string
    jmp $

; Setup GDT
setup_gdt:
    mov si, msg_gdt
    call print_string
    
    lgdt [gdt_descriptor]
    ret

; Print string (real mode)
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

BITS 32
protected_mode:
    ; Setup segments
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov esp, 0x90000
    
    ; Jump to kernel entry point
    jmp KERNEL_ENTRY

BITS 16

; Data
msg_stage2:      db 'Stage 2 Loader', 13, 10, 0
msg_load_kernel: db 'Loading kernel image...', 0
msg_a20:         db 'A20...', 0
msg_gdt:         db 'GDT...', 0
msg_ok:          db 'OK', 13, 10, 0
msg_error:       db 'ERROR', 13, 10, 0

; Disk Address Packet for INT 13h extended read
dap:
    db 0x10             ; Size of packet (16 bytes)
    db 0                ; Reserved (0)
    dw 128              ; Number of sectors to read (64KB)
    dw 0                ; Offset
    dw KERNEL_LOAD_SEG  ; Segment
    dq 16               ; Starting LBA (sector 16)

; GDT (Global Descriptor Table)
align 8
gdt_start:
    ; Null descriptor
    dq 0
    
    ; Code segment (0x08)
    dw 0xFFFF           ; Limit low
    dw 0x0000           ; Base low
    db 0x00             ; Base middle
    db 10011010b        ; Access: present, ring 0, code, executable, readable
    db 11001111b        ; Flags: 4KB granularity, 32-bit
    db 0x00             ; Base high
    
    ; Data segment (0x10)
    dw 0xFFFF           ; Limit low
    dw 0x0000           ; Base low
    db 0x00             ; Base middle
    db 10010010b        ; Access: present, ring 0, data, writable
    db 11001111b        ; Flags: 4KB granularity, 32-bit
    db 0x00             ; Base high

gdt_end:

gdt_descriptor:
    dw gdt_end - gdt_start - 1  ; Size
    dd gdt_start                ; Offset

; Pad to reasonable size (< 4KB)
times 760-($-$$) db 0

