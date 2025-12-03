// Keyboard IRQ Handler - x86 Assembly
// BSD 3-Clause License

.section .text
.global keyboard_irq_handler

// IRQ1 (keyboard) handler
keyboard_irq_handler:
    pushal
    
    call keyboard_interrupt_handler
    
    // Send EOI to PIC
    mov $0x20, %al
    out %al, $0x20
    
    popal
    iret
