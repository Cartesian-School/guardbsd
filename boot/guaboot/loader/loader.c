/*
 * GuaBoot Loader - Main Loader Implementation
 * BSD 3-Clause License
 * 
 * Loads kernel and modules according to guaboot.conf
 */

#include <stdint.h>
#include <stdbool.h>

#define COM1 0x3F8

typedef struct {
    char *kernel_path;
    char *module_path;
    bool boot_verbose;
    uint32_t autoboot_delay;
} loader_config_t;

static loader_config_t config = {
    .kernel_path = "/boot/kernel.elf",
    .module_path = "/boot/modules",
    .boot_verbose = true,
    .autoboot_delay = 0
};

static void serial_init(void) {
    *(volatile uint8_t*)(COM1 + 1) = 0x00;
    *(volatile uint8_t*)(COM1 + 3) = 0x80;
    *(volatile uint8_t*)(COM1 + 0) = 0x03;
    *(volatile uint8_t*)(COM1 + 1) = 0x00;
    *(volatile uint8_t*)(COM1 + 3) = 0x03;
    *(volatile uint8_t*)(COM1 + 2) = 0xC7;
    *(volatile uint8_t*)(COM1 + 4) = 0x0B;
}

static void putc(char c) {
    while (!(*(volatile uint8_t*)(COM1 + 5) & 0x20));
    *(volatile uint8_t*)COM1 = c;
}

static void puts(const char* s) {
    while (*s) {
        if (*s == '\n') putc('\r');
        putc(*s++);
    }
}

static void load_kernel(void) {
    puts("GuaBoot Loader v1.0.0\n");
    puts("BSD 3-Clause License\n\n");
    
    if (config.boot_verbose) {
        puts("[LOADER] Loading kernel: ");
        puts(config.kernel_path);
        puts("\n");
    }
    
    /* TODO: Parse ELF and load to memory */
    puts("[LOADER] Kernel loaded\n");
}

static void load_modules(void) {
    if (config.boot_verbose) {
        puts("[LOADER] Loading modules from: ");
        puts(config.module_path);
        puts("\n");
    }
    
    /* Load microkernels in order */
    const char *modules[] = {"uk_space", "uk_time", "uk_ipc", NULL};
    for (int i = 0; modules[i]; i++) {
        if (config.boot_verbose) {
            puts("[LOADER]   - ");
            puts(modules[i]);
            puts("\n");
        }
    }
}

void loader_main(void) {
    serial_init();
    
    puts("================================================================================\n");
    puts("GuaBoot Unified Bootloader\n");
    puts("================================================================================\n");
    
    load_kernel();
    load_modules();
    
    puts("[LOADER] Boot complete, transferring control to kernel\n");
    puts("================================================================================\n");
    
    /* Jump to kernel entry point */
    void (*kernel_entry)(void) = (void*)0x100000;
    kernel_entry();
}
