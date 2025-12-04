// arch/x86_64/interrupts/timer_isr.S
// Timer ISR hooking scheduler tick/preemption

.intel_syntax noprefix
.section .text
.global timer_isr
.type timer_isr,@function

// IDT handler entry
timer_isr:
    // CPU has pushed RIP, CS, RFLAGS, (RSP, SS if ring change)
    push rax
    push rcx
    push rdx
    push rbx
    push rbp
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15

    mov rdx, rsp              // pointer to saved regs
    sub rsp, 0xA8             // space for ArchContext
    mov rsi, rsp              // rsi = frame ptr for scheduler

    // Populate ArchContext from saved registers
    mov rax, [rdx + 0x70]     // r15
    mov [rsi + 0x00], rax
    mov rax, [rdx + 0x68]     // r14
    mov [rsi + 0x08], rax
    mov rax, [rdx + 0x60]     // r13
    mov [rsi + 0x10], rax
    mov rax, [rdx + 0x58]     // r12
    mov [rsi + 0x18], rax
    mov rax, [rdx + 0x50]     // r11
    mov [rsi + 0x20], rax
    mov rax, [rdx + 0x48]     // r10
    mov [rsi + 0x28], rax
    mov rax, [rdx + 0x40]     // r9
    mov [rsi + 0x30], rax
    mov rax, [rdx + 0x38]     // r8
    mov [rsi + 0x38], rax
    mov rax, [rdx + 0x30]     // rdi
    mov [rsi + 0x40], rax
    mov rax, [rdx + 0x28]     // rsi
    mov [rsi + 0x48], rax
    mov rax, [rdx + 0x20]     // rbp
    mov [rsi + 0x50], rax
    mov rax, [rdx + 0x18]     // rbx
    mov [rsi + 0x58], rax
    mov rax, [rdx + 0x10]     // rdx
    mov [rsi + 0x60], rax
    mov rax, [rdx + 0x08]     // rcx
    mov [rsi + 0x68], rax
    mov rax, [rdx + 0x00]     // rax
    mov [rsi + 0x70], rax

    // RSP is original interrupt stack (after CPU push)
    lea rax, [rdx + 0x78]     // points to interrupt frame (RIP at +0x00)
    mov [rsi + 0x78], rax
    mov rax, [rdx + 0x78]     // RIP from interrupt frame
    mov [rsi + 0x80], rax
    mov rax, [rdx + 0x88]     // RFLAGS
    mov [rsi + 0x88], rax
    mov rax, [rdx + 0x80]     // CS
    mov [rsi + 0x90], rax
    mov rax, [rdx + 0x90]     // SS (if present)
    mov [rsi + 0x98], rax
    mov rax, cr3
    mov [rsi + 0xA0], rax

    mov rdi, 0                // cpu_id = 0 (TODO: per-CPU ID)
    // rsi already context pointer
    call scheduler_handle_tick

    test rax, rax
    je 1f
    // Switch to new context
    mov rdi, rsi              // old ctx pointer
    mov rsi, rax              // new ctx pointer
    call arch_context_switch

1:
    add rsp, 0xA8
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rbp
    pop rbx
    pop rdx
    pop rcx
    pop rax

    // Acknowledge PIC/APIC
    mov al, 0x20
    out 0x20, al
    iretq

.size timer_isr,.-timer_isr
