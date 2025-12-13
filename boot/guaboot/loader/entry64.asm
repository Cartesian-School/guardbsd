; Project: GuardBSD Winter Saga version 1.0.0
; Package: guaboot_loader_stub
; Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
; License: BSD-3-Clause
;
; 64-bit transition stub. Loaded by stage2 at 0x0000F000 (ENTRY64_SEG:0).
; Executed after long mode is enabled; sets up 64-bit environment and
; jumps to kernel entry stored by loader at 0x9004.

[BITS 64]

%define GBSD_MAGIC 0x42534447
%define BOOTINFO_PTR 0x7010

global entry64_stub

section .text

entry64_stub:
    ; Load 64-bit data selectors (same GDT as loader)
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    ; Align and set stack (2 MiB)
    mov rsp, 0x200000
    and rsp, ~0xF
    xor rbp, rbp

    ; Clear direction flag
    cld

    ; Zero GPRs for a clean ABI state
    xor rax, rax
    xor rbx, rbx
    xor rcx, rcx
    xor rdx, rdx
    xor rsi, rsi
    xor rdi, rdi
    xor r8, r8
    xor r9, r9
    xor r10, r10
    xor r11, r11
    xor r12, r12
    xor r13, r13
    xor r14, r14
    xor r15, r15

    ; Pass boot protocol registers
    mov rdi, GBSD_MAGIC
    mov rsi, [abs BOOTINFO_PTR]

    ; Load kernel entry pointer written by loader at 0x7004
    mov rax, [abs 0x7004]
    jmp rax
