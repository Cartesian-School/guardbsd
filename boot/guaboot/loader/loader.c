/*
 * Project: GuardBSD Winter Saga version 1.0.0
 * Package: guaboot_loader
 * Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
 * License: BSD-3-Clause
 *
 * Główny loader GuaBoot (ładowanie ELF, guaboot.conf).
 */

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#define COM1 0x3F8
#define GBSD_MAGIC 0x42534447
#define BOOTINFO_PTR_SLOT ((uint64_t*)0x7010)

static inline void outb(uint16_t port, uint8_t val) {
    __asm__ volatile("outb %0, %1" : : "a"(val), "Nd"(port));
}

static inline uint8_t inb(uint16_t port) {
    uint8_t ret;
    __asm__ volatile("inb %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}


// Boot protocol handoff locations populated by stage2
#define KERNEL_PHYS_PTR   ((uint32_t*)0x7000)
#define KERNEL_ENTRY_SLOT ((uint64_t*)0x7004)
#define ENTRY64_LINEAR    0x0000F000  // stage2 loads entry64.bin here (separate from loader/page tables)

// Forward declarations
static void put_hex_static(uint32_t val);

// ELF Header Structures (64-bit)
typedef struct {
    uint8_t  e_ident[16];
    uint16_t e_type;
    uint16_t e_machine;
    uint32_t e_version;
    uint64_t e_entry;
    uint64_t e_phoff;
    uint64_t e_shoff;
    uint32_t e_flags;
    uint16_t e_ehsize;
    uint16_t e_phentsize;
    uint16_t e_phnum;
    uint16_t e_shentsize;
    uint16_t e_shnum;
    uint16_t e_shstrndx;
} Elf64_Ehdr;

typedef struct {
    uint32_t p_type;
    uint32_t p_flags;
    uint64_t p_offset;
    uint64_t p_vaddr;
    uint64_t p_paddr;
    uint64_t p_filesz;
    uint64_t p_memsz;
    uint64_t p_align;
} Elf64_Phdr;

typedef struct {
    int64_t  d_tag;
    uint64_t d_un;
} Elf64_Dyn;

typedef struct {
    uint64_t r_offset;
    uint64_t r_info;
    int64_t  r_addend;
} Elf64_Rela;

// ELF Constants
#define ET_EXEC      2
#define ET_DYN       3
#define PT_LOAD      1
#define PT_DYNAMIC   2
#define EI_CLASS     4
#define ELFCLASS64   2
#define EI_DATA      5
#define ELFDATA2LSB  1
#define EI_MAG0      0
#define ELFMAG0      0x7F
#define EI_MAG1      1
#define ELFMAG1      'E'
#define EI_MAG2      2
#define ELFMAG2      'L'
#define EI_MAG3      3
#define ELFMAG3      'F'
#define EM_X86_64    62
#define DT_NULL      0
#define DT_RELA      7
#define DT_RELASZ    8
#define DT_RELAENT   9
#define R_X86_64_RELATIVE 8
#define ELF64_R_TYPE(info) ((uint32_t)(info))

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
    outb(COM1 + 1, 0x00);
    outb(COM1 + 3, 0x80);
    outb(COM1 + 0, 0x03);
    outb(COM1 + 1, 0x00);
    outb(COM1 + 3, 0x03);
    outb(COM1 + 2, 0xC7);
    outb(COM1 + 4, 0x0B);
}

static void putc(char c) {
    while (!(inb(COM1 + 5) & 0x20));
    outb(COM1, c);
}

static void puts(const char* s) {
    while (*s) {
        if (*s == '\n') putc('\r');
        putc(*s++);
    }
}

static inline void io_wait(void) {
    outb(0x80, 0);
}

static void disable_pic(void) {
    outb(0x21, 0xFF);
    outb(0xA1, 0xFF);
    io_wait();
}

// GuaBoot BootInfo structures (minimal stub for kernel bring-up)
typedef struct {
    uint64_t base;
    uint64_t length;
    uint32_t typ;
    uint32_t reserved;
} BootMmapEntry;

typedef struct {
    uint32_t magic;
    uint32_t version;
    uint32_t size;
    uint32_t kernel_crc32;
    uint64_t kernel_base;
    uint64_t kernel_size;
    uint64_t mem_lower;
    uint64_t mem_upper;
    uint32_t boot_device;
    uint32_t pad0;      // align to 8 bytes
    uint64_t cmdline;
    uint32_t mods_count;
    uint32_t pad1;      // align to 8 bytes
    uint64_t mods;
    uint64_t mmap;
    uint32_t mmap_count;
    uint32_t pad2;      // align to 8 bytes / 80-byte struct
} BootInfo;

static BootMmapEntry boot_mmap[2];
static BootInfo bootinfo;
static const char cmdline_str[] = "";
static uint32_t kernel_crc32_value = 0;
static uint64_t kernel_load_base = 0;
static uint64_t kernel_load_size = 0;

static uint32_t crc32_calc(const uint8_t *data, uint32_t len) {
    uint32_t crc = 0xFFFFffff;
    for (uint32_t i = 0; i < len; i++) {
        crc ^= data[i];
        for (int b = 0; b < 8; b++) {
            uint32_t mask = 0u - (crc & 1u);
            crc = (crc >> 1) ^ (0xEDB88320u & mask);
        }
    }
    return crc ^ 0xFFFFffff;
}

static inline void debug_e9(char ch) {
    __asm__ volatile (
        "push %%ax\n"
        "movb %0, %%al\n"
        "out  $0xE9, %%al\n"
        "pop  %%ax\n"
        :
        : "r"(ch)
        : "al");
}

static void build_bootinfo(void) {
    bootinfo.magic = GBSD_MAGIC;
    bootinfo.version = 1;
    bootinfo.size = sizeof(BootInfo);
    bootinfo.kernel_crc32 = kernel_crc32_value;
    bootinfo.kernel_base = kernel_load_base;
    bootinfo.kernel_size = kernel_load_size;
    bootinfo.mem_lower = 640;    // KB below 1MiB (legacy value)
    bootinfo.mem_upper = 128 * 1024; // 128 MiB in KB units
    bootinfo.boot_device = 0;
    bootinfo.pad0 = 0;
    bootinfo.cmdline = (uint64_t)(uintptr_t)cmdline_str;
    bootinfo.mods_count = 0;
    bootinfo.pad1 = 0;
    bootinfo.mods = 0;

    // Mark low 1MiB as reserved, rest of first 128MiB as usable
    boot_mmap[0].base = 0x00000000;
    boot_mmap[0].length = 0x000100000;
    boot_mmap[0].typ = 2; // reserved
    boot_mmap[0].reserved = 0;

    boot_mmap[1].base = 0x000100000;
    boot_mmap[1].length = 0x07F00000; // ~127MiB usable
    boot_mmap[1].typ = 1; // usable
    boot_mmap[1].reserved = 0;

    bootinfo.mmap = (uint64_t)(uintptr_t)boot_mmap;
    bootinfo.mmap_count = 2;
    bootinfo.pad2 = 0;

    *BOOTINFO_PTR_SLOT = (uint64_t)(uintptr_t)&bootinfo;
}

// Simple identity paging tables (1 GiB via 2 MiB pages)
__attribute__((aligned(4096))) static uint64_t pml4[512];
__attribute__((aligned(4096))) static uint64_t pdpt[512];
__attribute__((aligned(4096))) static uint64_t pd[512];

static void setup_identity_paging(void) {
    for (int i = 0; i < 512; i++) {
        pml4[i] = 0;
        pdpt[i] = 0;
        pd[i] = 0;
    }

    pml4[0] = ((uint32_t)(uintptr_t)pdpt) | 0x03; // present | rw
    pdpt[0] = ((uint32_t)(uintptr_t)pd) | 0x03;
    for (int i = 0; i < 512; i++) {
        uint64_t addr = (uint64_t)i * 0x200000ULL;
        pd[i] = addr | 0x83; // present | rw | huge
    }
}

// Minimal GDT for long mode
struct gdt_entry {
    uint16_t limit_low;
    uint16_t base_low;
    uint8_t  base_mid;
    uint8_t  access;
    uint8_t  gran;
    uint8_t  base_hi;
} __attribute__((packed));

struct gdt_ptr {
    uint16_t limit;
    uint32_t base;
} __attribute__((packed));

static struct gdt_entry gdt[3] __attribute__((aligned(8))) = {
    {0, 0, 0, 0, 0, 0},
    {0xFFFF, 0x0000, 0x00, 0x9A, 0xA0, 0x00}, // 64-bit code (L=1, G=1)
    {0xFFFF, 0x0000, 0x00, 0x92, 0xA0, 0x00}, // data
};

static struct gdt_ptr gdtp = {
    .limit = sizeof(gdt) - 1,
    .base = (uint32_t)(uintptr_t)gdt,
};

struct far_ptr {
    uint32_t offset;
    uint16_t selector;
} __attribute__((packed));

static void enable_long_mode_and_jump(uint64_t entry_point) {
    // Load GDT
    __asm__ volatile("lgdt %[g]" : : [g]"m"(gdtp));

    // Load data segments
    __asm__ volatile(
        "movw $0x10, %%ax\n"
        "movw %%ax, %%ds\n"
        "movw %%ax, %%es\n"
        "movw %%ax, %%ss\n"
        :
        :
        : "ax");

    // Set a 64-bit friendly stack (aligned)
    __asm__ volatile("movl $0x200000, %%esp" : : );

    debug_e9('A');

    // Enable PAE
    uint32_t cr4;
    __asm__ volatile("mov %%cr4, %0" : "=r"(cr4));
    cr4 |= (1 << 5);
    __asm__ volatile("mov %0, %%cr4" : : "r"(cr4));

    debug_e9('C');

    // Load PML4
    __asm__ volatile("mov %0, %%cr3" : : "r"(pml4));

    debug_e9('B');

    // Enable LME and NXE
    uint32_t eax, edx;
    __asm__ volatile("mov $0xC0000080, %%ecx; rdmsr" : "=a"(eax), "=d"(edx) : : "ecx");
    eax |= (1 << 8);   // LME
    eax |= (1 << 11);  // NXE
    __asm__ volatile("mov $0xC0000080, %%ecx; wrmsr" : : "a"(eax), "d"(edx), "c"(0xC0000080));

    debug_e9('D');

    // Enable paging
    uint32_t cr0;
    __asm__ volatile("mov %%cr0, %0" : "=r"(cr0));
    cr0 |= (1 << 31); // PG
    __asm__ volatile("mov %0, %%cr0" : : "r"(cr0));

    debug_e9('E');

    // Jump to 64-bit transition stub (entry64.bin loaded at ENTRY64_LINEAR)
    struct far_ptr target = {
        .offset = ENTRY64_LINEAR,
        .selector = 0x08,
    };
    __asm__ volatile("ljmp *%0" : : "m"(target));
}

static bool validate_elf_header(const Elf64_Ehdr *ehdr) {
    // Check ELF magic
    if (ehdr->e_ident[EI_MAG0] != ELFMAG0 ||
        ehdr->e_ident[EI_MAG1] != ELFMAG1 ||
        ehdr->e_ident[EI_MAG2] != ELFMAG2 ||
        ehdr->e_ident[EI_MAG3] != ELFMAG3) {
        puts("[LOADER] ERROR: Invalid ELF magic\n");
        return false;
    }

    // Check 64-bit, little-endian, executable, x86_64
    if (ehdr->e_ident[EI_CLASS] != ELFCLASS64 ||
        ehdr->e_ident[EI_DATA] != ELFDATA2LSB ||
        ehdr->e_type != ET_EXEC ||
        ehdr->e_machine != EM_X86_64) {
        puts("[LOADER] ERROR: Unsupported ELF format (expecting ET_EXEC x86_64)\n");
        return false;
    }

    return true;
}

static void* load_kernel_from_memory(void) {
    // Kernel is pre-loaded by stage2; stage2 writes its physical address here.
    uint32_t KERNEL_LOAD_ADDR = *KERNEL_PHYS_PTR;
    if (KERNEL_LOAD_ADDR == 0) {
        // Fallback (legacy assumption)
        KERNEL_LOAD_ADDR = 0x10000;
    }
    const Elf64_Ehdr *ehdr = (const Elf64_Ehdr*)(uintptr_t)KERNEL_LOAD_ADDR;

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
    const Elf64_Phdr *phdr = (const Elf64_Phdr*)((uint8_t*)ehdr + ehdr->e_phoff);
    uint64_t load_base = 0xFFFFFFFFFFFFFFFFULL;
    uint64_t load_end = 0;
    for (uint16_t i = 0; i < ehdr->e_phnum; i++) {
        if (phdr[i].p_type == PT_LOAD) {
            if (phdr[i].p_paddr < 0x00100000ULL) {
                puts("[LOADER] ERROR: Refusing to load below 1MiB\n");
                return NULL;
            }
            if (phdr[i].p_paddr < load_base) {
                load_base = phdr[i].p_paddr;
            }
            if (config.boot_verbose) {
            puts("[LOADER]   Loading segment to 0x");
            put_hex_static((uint32_t)phdr[i].p_paddr);
            puts("\n");
            }

            // Copy segment data
            uint8_t *src = (uint8_t*)((uintptr_t)ehdr + phdr[i].p_offset);
            uint8_t *dst = (uint8_t*)(uintptr_t)phdr[i].p_paddr;

            for (uint64_t j = 0; j < phdr[i].p_filesz; j++) {
                dst[j] = src[j];
            }

            // Zero BSS
            for (uint64_t j = phdr[i].p_filesz; j < phdr[i].p_memsz; j++) {
                dst[j] = 0;
            }

            uint64_t seg_end = phdr[i].p_paddr + phdr[i].p_memsz;
            if (seg_end > load_end) {
                load_end = seg_end;
            }
        }
    }

    // Compute kernel CRC over the loaded image range
    if (load_base != 0xFFFFFFFFFFFFFFFFULL && load_end > load_base) {
        kernel_load_base = load_base;
        kernel_load_size = load_end - load_base;
        uint32_t crc = crc32_calc((const uint8_t *)(uintptr_t)load_base, (uint32_t)(load_end - load_base));
        kernel_crc32_value = crc;
    } else {
        kernel_load_base = 0;
        kernel_load_size = 0;
        kernel_crc32_value = 0;
    }

    if (config.boot_verbose) {
        puts("[LOADER] Kernel entry point: 0x");
        put_hex_static(ehdr->e_entry);
        puts("\n");
    }

    return (void*)(uintptr_t)ehdr->e_entry;
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
    *KERNEL_ENTRY_SLOT = (uint64_t)(uintptr_t)entry_point; // Store in safe location

    // Build minimal BootInfo for the kernel
    build_bootinfo();
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

    // Mask PIC before any interrupts in 64-bit space
    disable_pic();

    puts("[LOADER] Boot complete, transferring control to kernel\n");
    puts("================================================================================\n");

    /* Get kernel entry point from loader */
    void *entry_point = *KERNEL_ENTRY_SLOT;
    if (!entry_point) {
        puts("[LOADER] ERROR: No kernel entry point found\n");
        while (1);
    }

    puts("[LOADER] Enabling long mode and jumping to kernel at 0x");
    put_hex_static((uint32_t)(uintptr_t)entry_point);
    puts("\n");

    // Build identity paging and enter long mode
    setup_identity_paging();
    enable_long_mode_and_jump((uint64_t)(uintptr_t)entry_point);
}

static void put_hex_static(uint32_t val) {
    char hex_chars[] = "0123456789ABCDEF";
    for (int i = 28; i >= 0; i -= 4) {
        putc(hex_chars[(val >> i) & 0xF]);
    }
}
