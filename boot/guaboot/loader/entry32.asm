; Project: GuardBSD Winter Saga version 1.0.0
; Package: guaboot_loader_entry
; Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
; License: BSD-3-Clause
;
; 32-bit entry stub placed at the start of loader.bin.
; Stage2 jumps here (0x8000), we immediately chain to loader_main().

[BITS 32]

global _start
extern loader_main

section .text
_start:
    call loader_main
.hang:
    hlt
    jmp .hang
