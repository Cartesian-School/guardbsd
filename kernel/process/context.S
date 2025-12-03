// Context Switch - x86 Assembly (32-bit)
// BSD 3-Clause License

.section .text
.global context_switch
.global jump_to_user

// context_switch(old_regs: *mut Registers, new_regs: *const Registers)
context_switch:
    // Get arguments from stack
    mov %eax, 4(%esp)   // old_regs
    mov %edx, 8(%esp)   // new_regs
    
    // Save old context
    mov (%eax), %ecx
    mov %ecx, 0(%eax)
    mov %ebx, 4(%eax)
    mov %ecx, 8(%eax)
    mov %edx, 12(%eax)
    mov %esi, 16(%eax)
    mov %edi, 20(%eax)
    mov %ebp, 24(%eax)
    mov %esp, 28(%eax)
    
    // Save eip (return address)
    mov (%esp), %ecx
    mov %ecx, 32(%eax)
    
    // Save eflags
    pushf
    pop %ecx
    mov %ecx, 36(%eax)
    
    // Load new context
    mov 4(%edx), %ebx
    mov 8(%edx), %ecx
    mov 24(%edx), %ebp
    mov 28(%edx), %esp
    
    // Load eflags
    mov 36(%edx), %eax
    push %eax
    popf
    
    // Load eip
    mov 32(%edx), %eax
    push %eax
    
    // Load remaining regs
    mov 20(%edx), %edi
    mov 16(%edx), %esi
    mov 12(%edx), %edx
    mov 0(%edx), %eax
    
    ret

// jump_to_user(entry: u32, stack: u32)
jump_to_user:
    mov 4(%esp), %eax   // entry
    mov 8(%esp), %ecx   // stack
    
    mov %ecx, %esp
    push $0x23          // User data segment
    push %ecx           // User stack
    pushf               // Flags
    push $0x1B          // User code segment
    push %eax           // Entry point
    iret
