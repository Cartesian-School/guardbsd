// Timer IRQ Handler - x86 Assembly (legacy 32-bit)
// BSD 3-Clause License

#ifndef X86_LEGACY
#define X86_LEGACY
#endif
#ifdef X86_LEGACY

.section .text
.global timer_irq_handler

// IRQ0 (timer) handler
timer_irq_handler:
    pushal
    
    call timer_interrupt_handler
    
    // Send EOI to PIC
    mov $0x20, %al
    out %al, $0x20
    
    popal
    iret
#endif
