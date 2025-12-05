; GuardBSD Boot Sector
; BSD 3-Clause License
; Copyright (c) 2025, Cartesian School - Siergej Sobolewski
;
; Simple boot sector that loads stage 2 bootloader
; Designed for El Torito CD-ROM booting

BITS 16
ORG 0x7C00

%define STAGE2_SEGMENT 0x0820
%define STAGE2_OFFSET  0x0000
%define KERNEL_SEGMENT 0x1000

start:
    ; Disable interrupts during setup
    cli
    
    ; Set up segments
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00
    
    ; Enable interrupts
    sti
    
    ; Clear screen and print banner
    call clear_screen
    mov si, msg_boot
    call print_string
    
    ; Check for CD-ROM
    mov ah, 0x41
    mov bx, 0x55AA
    mov dl, 0x80  ; First hard disk or CD
    int 0x13
    jc no_cdrom
    
    ; Load stage 2 (if it exists)
    mov si, msg_loading
    call print_string
    
    ; For now, just load kernel directly
    ; In a real implementation, this would load guaboot2.bin
    call load_kernel
    
    ; Jump to kernel
    mov si, msg_jump
    call print_string
    
    jmp KERNEL_SEGMENT:0x0000

no_cdrom:
    mov si, msg_no_cd
    call print_string
    jmp hang

load_kernel:
    ; This is a simplified loader
    ; In production, use INT 13h extended read
    
    ; For El Torito, kernel should be in specific location
    mov ax, KERNEL_SEGMENT
    mov es, ax
    xor bx, bx
    
    ; Read sectors (simplified - assumes kernel at LBA 16)
    mov ah, 0x02    ; Read sectors
    mov al, 64      ; 64 sectors (32KB)
    mov ch, 0       ; Cylinder 0
    mov cl, 17      ; Sector 17 (skip boot sector + catalog)
    mov dh, 0       ; Head 0
    mov dl, 0x80    ; Drive
    int 0x13
    jc load_error
    
    ret

load_error:
    mov si, msg_error
    call print_string
    jmp hang

clear_screen:
    mov ax, 0x0003  ; Set video mode 3 (80x25 text)
    int 0x10
    ret

print_string:
    pusha
.loop:
    lodsb
    or al, al
    jz .done
    mov ah, 0x0E
    mov bx, 0x0007  ; Page 0, light gray
    int 0x10
    jmp .loop
.done:
    popa
    ret

hang:
    mov si, msg_halt
    call print_string
    cli
    hlt
    jmp hang

; Data
msg_boot:     db 'GuardBSD Boot Loader v1.0', 13, 10, 0
msg_loading:  db 'Loading kernel...', 13, 10, 0
msg_jump:     db 'Starting GuardBSD...', 13, 10, 0
msg_no_cd:    db 'Error: CD-ROM not detected', 13, 10, 0
msg_error:    db 'Error: Failed to load kernel', 13, 10, 0
msg_halt:     db 'System halted.', 13, 10, 0

; Boot signature must be at offset 510
times 510-($-$$) db 0
dw 0xAA55

