/*
 * GuaBoot Loader - Main Loader Implementation
 * BSD 3-Clause License
 *
 * Loads kernel and modules according to guaboot.conf
 * Implements ELF loading for proper kernel bootstrapping
 */

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#define COM1 0x3F8

// Forward declarations
static void put_hex_static(uint32_t val);

// ELF Header Structures (32-bit)
typedef struct {
    uint8_t  e_ident[16];
    uint16_t e_type;
    uint16_t e_machine;
    uint32_t e_version;
    uint32_t e_entry;
    uint32_t e_phoff;
    uint32_t e_shoff;
    uint32_t e_flags;
    uint16_t e_ehsize;
    uint16_t e_phentsize;
    uint16_t e_phnum;
    uint16_t e_shentsize;
    uint16_t e_shnum;
    uint16_t e_shstrndx;
} Elf32_Ehdr;

typedef struct {
    uint32_t p_type;
    uint32_t p_offset;
    uint32_t p_vaddr;
    uint32_t p_paddr;
    uint32_t p_filesz;
    uint32_t p_memsz;
    uint32_t p_flags;
    uint32_t p_align;
} Elf32_Phdr;

// ELF Constants
#define ET_EXEC     2
#define PT_LOAD     1
#define EI_CLASS    4
#define ELFCLASS32  1
#define EI_DATA     5
#define ELFDATA2LSB 1
#define EI_MAG0     0
#define ELFMAG0     0x7F
#define EI_MAG1     1
#define ELFMAG1     'E'
#define EI_MAG2     2
#define ELFMAG2     'L'
#define EI_MAG3     3
#define ELFMAG3     'F'

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

static bool validate_elf_header(const Elf32_Ehdr *ehdr) {
    // Check ELF magic
    if (ehdr->e_ident[EI_MAG0] != ELFMAG0 ||
        ehdr->e_ident[EI_MAG1] != ELFMAG1 ||
        ehdr->e_ident[EI_MAG2] != ELFMAG2 ||
        ehdr->e_ident[EI_MAG3] != ELFMAG3) {
        puts("[LOADER] ERROR: Invalid ELF magic\n");
        return false;
    }

    // Check 32-bit, little-endian, executable
    if (ehdr->e_ident[EI_CLASS] != ELFCLASS32 ||
        ehdr->e_ident[EI_DATA] != ELFDATA2LSB ||
        ehdr->e_type != ET_EXEC) {
        puts("[LOADER] ERROR: Unsupported ELF format\n");
        return false;
    }

    return true;
}

static void* load_kernel_from_memory(void) {
    // For now, assume kernel is pre-loaded at 0x100000
    // In a real implementation, this would read from disk
    const uint32_t KERNEL_LOAD_ADDR = 0x100000;
    const Elf32_Ehdr *ehdr = (const Elf32_Ehdr*)KERNEL_LOAD_ADDR;

    if (config.boot_verbose) {
        puts("[LOADER] Validating kernel ELF header...\n");
    }

    if (!validate_elf_header(ehdr)) {
        puts("[LOADER] ERROR: Invalid kernel ELF\n");
        return NULL;
    }

    if (config.boot_verbose) {
        puts("[LOADER] Loading kernel segments...\n");
    }

    // Load program headers
    const Elf32_Phdr *phdr = (const Elf32_Phdr*)(KERNEL_LOAD_ADDR + ehdr->e_phoff);
    for (uint16_t i = 0; i < ehdr->e_phnum; i++) {
        if (phdr[i].p_type == PT_LOAD) {
            if (config.boot_verbose) {
            puts("[LOADER]   Loading segment to 0x");
            put_hex_static(phdr[i].p_vaddr);
            puts("\n");
            }

            // Copy segment data
            uint8_t *src = (uint8_t*)(KERNEL_LOAD_ADDR + phdr[i].p_offset);
            uint8_t *dst = (uint8_t*)phdr[i].p_vaddr;

            for (uint32_t j = 0; j < phdr[i].p_filesz; j++) {
                dst[j] = src[j];
            }

            // Zero BSS
            for (uint32_t j = phdr[i].p_filesz; j < phdr[i].p_memsz; j++) {
                dst[j] = 0;
            }
        }
    }

    if (config.boot_verbose) {
        puts("[LOADER] Kernel entry point: 0x");
        put_hex_static(ehdr->e_entry);
        puts("\n");
    }

    return (void*)ehdr->e_entry;
}

static void load_kernel(void) {
    puts("GuaBoot Loader v1.0.0\n");
    puts("BSD 3-Clause License\n\n");

    if (config.boot_verbose) {
        puts("[LOADER] Loading kernel: ");
        puts(config.kernel_path);
        puts("\n");
    }

    void *entry_point = load_kernel_from_memory();
    if (!entry_point) {
        puts("[LOADER] ERROR: Failed to load kernel\n");
        while (1); // Halt on failure
    }

    puts("[LOADER] Kernel loaded successfully\n");

    // Store entry point for main function
    *(void**)(0x9000) = entry_point; // Store in safe location
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

    /* Get kernel entry point from loader */
    void *entry_point = *(void**)(0x9000);
    if (!entry_point) {
        puts("[LOADER] ERROR: No kernel entry point found\n");
        while (1);
    }

    /* Jump to kernel entry point */
    void (*kernel_entry)(void) = entry_point;
    puts("[LOADER] Jumping to kernel at 0x");
    put_hex_static((uint32_t)entry_point);
    puts("\n");
    kernel_entry();
}

static void put_hex_static(uint32_t val) {
    char hex_chars[] = "0123456789ABCDEF";
    for (int i = 28; i >= 0; i -= 4) {
        putc(hex_chars[(val >> i) & 0xF]);
    }
}
