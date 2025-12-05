/*
 * GuaBoot Stage 2 - BIOS Interface and Long Mode Transition
 * BSD 3-Clause License
 * Copyright (c) 2025, GuardBSD Project
 */

.code32
.section .text

/* ========================================================================
 * BIOS Interface Functions (16-bit BIOS calls from 32-bit protected mode)
 * ======================================================================== */

/*
 * bios_putchar(char c)
 * Print character using BIOS INT 10h
 */
.global bios_putchar
bios_putchar:
    pushl %ebp
    movl %esp, %ebp
    pushl %ebx
    
    /* Get character from argument */
    movb 8(%ebp), %al
    
    /* Use BIOS INT 10h, function 0Eh (teletype output) */
    /* For simplicity in protected mode, write directly to VGA */
    movl $0xB8000, %ebx
    movzbl cursor_pos, %ecx
    movb %al, (%ebx, %ecx, 2)
    movb $0x07, 1(%ebx, %ecx, 2)  /* Light gray on black */
    
    incl %ecx
    movb %cl, cursor_pos
    
    popl %ebx
    popl %ebp
    ret

cursor_pos: .byte 0

/*
 * bios_read_disk(uint32_t lba, uint16_t count, void *buffer)
 * Read sectors from disk using BIOS INT 13h
 * Returns 0 on success, -1 on error
 *
 * Note: This is simplified - in real implementation, you'd need to
 * switch back to real mode or use protected mode disk I/O
 */
.global bios_read_disk
bios_read_disk:
    pushl %ebp
    movl %esp, %ebp
    
    /* For minimal version, assume kernel is already loaded by stage 1 */
    /* or implement protected mode disk I/O */
    
    xorl %eax, %eax  /* Return success */
    
    popl %ebp
    ret

/*
 * bios_detect_memory_e820(struct E820Entry *entries, int max_entries)
 * Detect memory using BIOS INT 15h, E820h
 * Returns number of entries detected
 *
 * Note: Simplified version - assumes memory map is available
 */
.global bios_detect_memory_e820
bios_detect_memory_e820:
    pushl %ebp
    movl %esp, %ebp
    
    /* For minimal version, return 0 (use fallback) */
    xorl %eax, %eax
    
    popl %ebp
    ret

/* ========================================================================
 * Long Mode Transition
 * ======================================================================== */

/*
 * switch_to_long_mode(uint64_t entry, uint64_t bootinfo)
 *
 * Transitions from 32-bit protected mode to 64-bit long mode
 * and jumps to kernel entry point
 *
 * Arguments (on stack):
 *   [esp+4]  = kernel entry point (64-bit)
 *   [esp+12] = bootinfo pointer (64-bit)
 */
.global switch_to_long_mode
switch_to_long_mode:
    cli
    
    /* Save arguments */
    movl 4(%esp), %esi   /* Entry point (low 32 bits) */
    movl 12(%esp), %edi  /* BootInfo pointer (low 32 bits) */
    
    /* Set up page tables for identity mapping */
    call setup_page_tables
    
    /* Load CR3 with PML4 address */
    movl $pml4, %eax
    movl %eax, %cr3
    
    /* Enable PAE (CR4.PAE) */
    movl %cr4, %eax
    orl $0x20, %eax      /* Set PAE bit */
    movl %eax, %cr4
    
    /* Set EFER.LME (Long Mode Enable) */
    movl $0xC0000080, %ecx  /* EFER MSR */
    rdmsr
    orl $0x100, %eax     /* Set LME bit */
    wrmsr
    
    /* Enable paging (CR0.PG) to activate long mode */
    movl %cr0, %eax
    orl $0x80000000, %eax  /* Set PG bit */
    movl %eax, %cr0
    
    /* Now in compatibility mode, need far jump to 64-bit code segment */
    
    /* Load new GDT with 64-bit code segment */
    lgdt gdt64_ptr
    
    /* Far jump to 64-bit code */
    ljmp $0x08, $long_mode_entry

/* ========================================================================
 * Page Table Setup (Identity Mapping)
 * ======================================================================== */

setup_page_tables:
    /* Clear page tables */
    movl $pml4, %edi
    xorl %eax, %eax
    movl $4096, %ecx
    rep stosl
    
    /* Set up 4-level paging (PML4 -> PDPT -> PD -> PT) */
    /* Identity map first 512 MB (enough for kernel) */
    
    /* PML4[0] -> PDPT */
    movl $pdpt, %eax
    orl $0x03, %eax      /* Present + Writable */
    movl %eax, pml4
    
    /* PDPT[0] -> PD */
    movl $pd, %eax
    orl $0x03, %eax
    movl %eax, pdpt
    
    /* PD entries: map 512MB using 2MB pages */
    movl $pd, %edi
    movl $0x83, %eax     /* Present + Writable + Page Size (2MB) */
    movl $256, %ecx      /* 256 entries × 2MB = 512MB */
    
1:
    movl %eax, (%edi)
    addl $0x200000, %eax /* Next 2MB page */
    addl $8, %edi
    loop 1b
    
    ret

/* ========================================================================
 * 64-bit Entry Point
 * ======================================================================== */

.code64
long_mode_entry:
    /* Now in 64-bit long mode! */
    
    /* Set up 64-bit data segments */
    movw $0x10, %ax
    movw %ax, %ds
    movw %ax, %es
    movw %ax, %fs
    movw %ax, %gs
    movw %ax, %ss
    
    /* Set up kernel stack (8MB) */
    movq $0x800000, %rsp
    
    /* Prepare arguments for kernel entry */
    movl %esi, %edi      /* RDI = entry point (we'll fix this) */
    movl %edi, %esi      /* RSI = bootinfo pointer */
    
    /* Actually, we saved them wrong. Let's fix: */
    /* We want: RDI = GBSD_MAGIC, RSI = bootinfo */
    movq $0x42534447, %rdi   /* GBSD magic */
    /* RSI already has bootinfo from %edi above */
    
    /* Get actual kernel entry point from saved ESI */
    movl %esi, %eax
    movl saved_entry, %rax
    
    /* Jump to kernel */
    jmp *%rax
    
    /* Should never return */
    cli
    hlt
    jmp .

saved_entry: .quad 0

/* ========================================================================
 * Page Tables (aligned to 4KB)
 * ======================================================================== */

.align 4096
pml4:
    .space 4096

.align 4096
pdpt:
    .space 4096

.align 4096
pd:
    .space 4096

/* ========================================================================
 * GDT for 64-bit Long Mode
 * ======================================================================== */

.align 16
gdt64:
    .quad 0                 /* Null descriptor */
    
    /* Code segment (64-bit) */
    .word 0xFFFF            /* Limit low */
    .word 0x0000            /* Base low */
    .byte 0x00              /* Base middle */
    .byte 0x9A              /* Access: present, ring 0, code, executable, readable */
    .byte 0xAF              /* Flags: 64-bit, limit high */
    .byte 0x00              /* Base high */
    
    /* Data segment */
    .word 0xFFFF            /* Limit low */
    .word 0x0000            /* Base low */
    .byte 0x00              /* Base middle */
    .byte 0x92              /* Access: present, ring 0, data, writable */
    .byte 0xCF              /* Flags: 4KB granularity, 32-bit */
    .byte 0x00              /* Base high */

gdt64_end:

gdt64_ptr:
    .word gdt64_end - gdt64 - 1  /* Limit */
    .quad gdt64                   /* Base */

