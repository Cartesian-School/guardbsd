/*
 * GuaBoot Stage 2 - BIOS Loader
 * BSD 3-Clause License
 */

#include <stdint.h>

#define COM1 0x3F8

static void serial_putc(char c) {
    while (!(*(volatile uint8_t*)(COM1 + 5) & 0x20));
    *(volatile uint8_t*)COM1 = c;
}

static void puts(const char* s) {
    while (*s) serial_putc(*s++);
}

void _start(void) {
    puts("GuaBoot2 loading...\r\n");
    
    /* Load kernel from disk to 0x100000 (1MB) */
    /* Simplified: Jump to kernel entry point */
    
    puts("Jumping to kernel...\r\n");
    
    /* Jump to kernel at 1MB */
    void (*kernel_entry)(void) = (void*)0x100000;
    kernel_entry();
    
    /* Should never reach here */
    while(1);
}
