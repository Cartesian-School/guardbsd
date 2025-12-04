// kernel/arch/x86_64/interrupts/timer_isr64.S
// Timer ISR (vector 0x20) that builds a TrapFrameX86_64 and calls into Rust.

.intel_syntax noprefix
.section .text
.global timer_isr64
.type timer_isr64, @function

.equ TF_R15,       0
.equ TF_R14,       8
.equ TF_R13,       16
.equ TF_R12,       24
.equ TF_R11,       32
.equ TF_R10,       40
.equ TF_R9,        48
.equ TF_R8,        56
.equ TF_RDI,       64
.equ TF_RSI,       72
.equ TF_RBP,       80
.equ TF_RBX,       88
.equ TF_RDX,       96
.equ TF_RCX,       104
.equ TF_RAX,       112
.equ TF_RSP,       120
.equ TF_RIP,       128
.equ TF_RFLAGS,    136
.equ TF_CS,        144
.equ TF_SS,        152
.equ TF_ERROR,     160
.equ TF_MODE,      168
.equ TF_SIZE,      176

timer_isr64:
    // Allocate space for TrapFrame on the current stack
    sub rsp, TF_SIZE

    // Save GPRs into TrapFrame
    mov [rsp + TF_R15], r15
    mov [rsp + TF_R14], r14
    mov [rsp + TF_R13], r13
    mov [rsp + TF_R12], r12
    mov [rsp + TF_R11], r11
    mov [rsp + TF_R10], r10
    mov [rsp + TF_R9],  r9
    mov [rsp + TF_R8],  r8
    mov [rsp + TF_RDI], rdi
    mov [rsp + TF_RSI], rsi
    mov [rsp + TF_RBP], rbp
    mov [rsp + TF_RBX], rbx
    mov [rsp + TF_RDX], rdx
    mov [rsp + TF_RCX], rcx
    mov [rsp + TF_RAX], rax

    // Capture hardware-pushed frame (RIP, CS, RFLAGS) and RSP at entry
    lea r10, [rsp + TF_SIZE]     // r10 -> hardware frame (RIP at [r10])
    lea r11, [r10 + 24]          // r11 -> RSP at entry (after hw push)
    mov rax, [r10]               // RIP
    mov [rsp + TF_RIP], rax
    mov rax, [r10 + 8]           // CS
    mov [rsp + TF_CS], rax
    mov rax, [r10 + 16]          // RFLAGS
    mov [rsp + TF_RFLAGS], rax
    mov [rsp + TF_RSP], r11
    mov rax, ss
    mov [rsp + TF_SS], rax
    mov qword ptr [rsp + TF_ERROR], 0
    mov qword ptr [rsp + TF_MODE], 0

    // Call Rust handler with &mut TrapFrameX86_64
    mov rdi, rsp
    call x86_64_timer_interrupt_handler

    // Update hardware frame from TrapFrame (RIP/CS/RFLAGS)
    mov rax, [rsp + TF_RIP]
    mov [r10], rax
    mov rax, [rsp + TF_CS]
    mov [r10 + 8], rax
    mov rax, [rsp + TF_RFLAGS]
    mov [r10 + 16], rax

    // Restore registers from TrapFrame (except RSP which is set to hardware frame)
    mov r15, [rsp + TF_R15]
    mov r14, [rsp + TF_R14]
    mov r13, [rsp + TF_R13]
    mov r12, [rsp + TF_R12]
    mov r11, [rsp + TF_R11]
    mov r10, [rsp + TF_R10]
    mov r9,  [rsp + TF_R9]
    mov r8,  [rsp + TF_R8]
    mov rdi, [rsp + TF_RDI]
    mov rsi, [rsp + TF_RSI]
    mov rbp, [rsp + TF_RBP]
    mov rbx, [rsp + TF_RBX]
    mov rdx, [rsp + TF_RDX]
    mov rcx, [rsp + TF_RCX]
    mov rax, [rsp + TF_RAX]

    // Send PIC EOI (placeholder until APIC wired)
    mov al, 0x20
    out 0x20, al

    // Drop TrapFrame allocation; set RSP to hardware frame for iretq
    lea rsp, [rsp + TF_SIZE]
    iretq

.size timer_isr64, .-timer_isr64
