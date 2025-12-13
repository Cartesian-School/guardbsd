/*
 * Project: GuardBSD Winter Saga version 1.0.0
 * Package: guaboot_bios
 * Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
 * License: BSD-3-Clause
 *
 * GuaBoot stage 2 BIOS: ładuje główny loader z dysku.
 */

#include <stdint.h>

#define COM1 0x3F8
#define LOADER_LOAD_ADDR 0x8000
#define LOADER_SIZE_SECTORS 16  // Assume loader fits in 16 sectors (8KB)

static void serial_putc(char c) {
    while (!(*(volatile uint8_t*)(COM1 + 5) & 0x20));
    *(volatile uint8_t*)COM1 = c;
}

static void puts(const char* s) {
    while (*s) serial_putc(*s++);
}

static void put_hex(uint32_t val) {
    char hex_chars[] = "0123456789ABCDEF";
    for (int i = 28; i >= 0; i -= 4) {
        serial_putc(hex_chars[(val >> i) & 0xF]);
    }
}

static int load_loader(void) {
    puts("GuaBoot loader should be pre-loaded by ISO\r\n");
    // For ISO boot, the loader should already be loaded at LOADER_LOAD_ADDR
    // by the El Torito boot catalog or by the previous stage
    return 0;
}

void _start(void) {
    puts("GuaBoot Stage 2 v1.0.0\r\n");
    puts("BSD 3-Clause License\r\n\r\n");

    // Initialize serial port
    *(volatile uint8_t*)(COM1 + 1) = 0x00;  // Disable interrupts
    *(volatile uint8_t*)(COM1 + 3) = 0x80;  // Enable DLAB
    *(volatile uint8_t*)(COM1 + 0) = 0x03;  // Divisor low byte (9600 baud)
    *(volatile uint8_t*)(COM1 + 1) = 0x00;  // Divisor high byte
    *(volatile uint8_t*)(COM1 + 3) = 0x03;  // Disable DLAB, 8N1
    *(volatile uint8_t*)(COM1 + 2) = 0xC7;  // Enable FIFO
    *(volatile uint8_t*)(COM1 + 4) = 0x0B;  // Enable IRQs

    // Load the main loader
    if (load_loader() != 0) {
        puts("CRITICAL: Failed to load main loader\r\n");
        puts("System halted.\r\n");
        while (1) {
            __asm__ volatile ("hlt");
        }
    }

    // Transfer control to loader
    puts("Transferring control to GuaBoot loader...\r\n");

    void (*loader_entry)(void) = (void*)LOADER_LOAD_ADDR;
    loader_entry();

    // Should never reach here
    puts("ERROR: Unexpected return from loader\r\n");
    while (1) {
        __asm__ volatile ("hlt");
    }
}
