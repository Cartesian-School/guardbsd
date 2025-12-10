/*
 * GuaBoot Stage 2 - BIOS Loader
 * BSD 3-Clause License
 * Copyright (c) 2025, GuardBSD Project
 *
 * Minimal ELF loader for x86_64 kernel
 * Replaces GRUB/Multiboot with FreeBSD-style boot protocol
 */

#include <stdint.h>
#include <stddef.h>

/* ========================================================================
 * Type Definitions
 * ======================================================================== */

#define GBSD_MAGIC 0x42534447  /* "GBSD" */
#define BOOT_INFO_ADDR 0x100000  /* 1MB - safe location for bootinfo */

/* BootInfo structure - passed to kernel */
struct BootMmapEntry {
    uint64_t base;
    uint64_t length;
    uint32_t type;      /* 1 = usable, otherwise reserved */
    uint32_t reserved;
};

struct BootInfo {
    uint32_t magic;         /* 0x42534447 "GBSD" */
    uint32_t version;       /* 0x00010000 */
    uint32_t size;          /* sizeof(struct BootInfo) */
    uint32_t kernel_crc32;  /* CRC32 of loaded kernel image */
    uint64_t mem_lower;     /* Memory below 1MB (KB) */
    uint64_t mem_upper;     /* Memory above 1MB (KB) */
    uint32_t boot_device;   /* BIOS boot device */
    char *cmdline;          /* Kernel command line */
    uint32_t mods_count;    /* Number of modules */
    struct Module *mods;    /* Module array */
    struct BootMmapEntry *mmap; /* Memory map */
    uint32_t mmap_count;    /* Memory map entries */
};

struct Module {
    uint64_t mod_start;
    uint64_t mod_end;
    char *string;
    uint32_t reserved;
};

/* E820 memory map entry */
struct E820Entry {
    uint64_t base;
    uint64_t length;
    uint32_t type;
    uint32_t acpi_attrs;
} __attribute__((packed));

/* ELF structures */
#define EI_NIDENT 16
#define PT_LOAD 1

typedef struct {
    unsigned char e_ident[EI_NIDENT];
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

/* ========================================================================
 * BIOS Interface Functions (implemented in assembly)
 * ======================================================================== */

extern void bios_putchar(char c);
extern int bios_read_disk(uint32_t lba, uint16_t count, void *buffer);
extern int bios_detect_memory_e820(struct E820Entry *entries, int max_entries);
extern void switch_to_long_mode(uint64_t entry, uint64_t bootinfo);

/* ========================================================================
 * Console Functions
 * ======================================================================== */

static void puts(const char *s) {
    while (*s) {
        if (*s == '\n') bios_putchar('\r');
        bios_putchar(*s++);
    }
}

static void put_hex(uint64_t val) {
    static const char hex[] = "0123456789ABCDEF";
    char buf[17];
    buf[16] = 0;
    for (int i = 15; i >= 0; i--) {
        buf[i] = hex[val & 0xF];
        val >>= 4;
    }
    puts(buf);
}

/* ========================================================================
 * CRC32 (IEEE 802.3)
 * ======================================================================== */

static uint32_t crc32(const void *data, size_t len) {
    uint32_t crc = 0xFFFFFFFF;
    const uint8_t *p = (const uint8_t *)data;
    for (size_t i = 0; i < len; i++) {
        crc ^= p[i];
        for (int b = 0; b < 8; b++) {
            uint32_t mask = -(crc & 1u);
            crc = (crc >> 1) ^ (0xEDB88320 & mask);
        }
    }
    return crc ^ 0xFFFFFFFF;
}

static uint32_t compute_kernel_crc(void *elf_data) {
    Elf64_Ehdr *ehdr = (Elf64_Ehdr *)elf_data;
    if (!verify_elf(ehdr)) {
        return 0;
    }
    Elf64_Phdr *phdr = (Elf64_Phdr *)((uint8_t *)elf_data + ehdr->e_phoff);
    uint32_t crc = 0xFFFFFFFF;

    for (int i = 0; i < ehdr->e_phnum; i++) {
        if (phdr[i].p_type != PT_LOAD) continue;
        uint8_t *seg = (uint8_t *)(uintptr_t)phdr[i].p_paddr;
        size_t seg_len = (size_t)phdr[i].p_memsz;
        for (size_t j = 0; j < seg_len; j++) {
            crc ^= seg[j];
            for (int b = 0; b < 8; b++) {
                uint32_t mask = -(crc & 1u);
                crc = (crc >> 1) ^ (0xEDB88320 & mask);
            }
        }
    }
    return crc ^ 0xFFFFFFFF;
}

/* ========================================================================
 * Memory Functions
 * ======================================================================== */

static void *memcpy(void *dest, const void *src, size_t n) {
    uint8_t *d = dest;
    const uint8_t *s = src;
    while (n--) *d++ = *s++;
    return dest;
}

static void *memset(void *s, int c, size_t n) {
    uint8_t *p = s;
    while (n--) *p++ = (uint8_t)c;
    return s;
}

/* ========================================================================
 * Simplified Filesystem (assumes ISO9660 or simple layout)
 * ======================================================================== */

#define KERNEL_LBA_START 16   /* Hardcoded: kernel starts at LBA 16 */
#define SECTOR_SIZE 2048      /* ISO9660 sector size */

/* Simplified: just read kernel from known LBA */
static int read_kernel(void *buffer, size_t max_size) {
    /* Read up to 256 sectors (512KB) of kernel */
    int sectors = max_size / SECTOR_SIZE;
    if (sectors > 256) sectors = 256;
    
    return bios_read_disk(KERNEL_LBA_START, sectors, buffer);
}

/* ========================================================================
 * ELF Loader
 * ======================================================================== */

static int verify_elf(Elf64_Ehdr *ehdr) {
    /* Check ELF magic */
    if (ehdr->e_ident[0] != 0x7F ||
        ehdr->e_ident[1] != 'E' ||
        ehdr->e_ident[2] != 'L' ||
        ehdr->e_ident[3] != 'F') {
        return 0;
    }
    
    /* Check 64-bit */
    if (ehdr->e_ident[4] != 2) {  /* ELFCLASS64 */
        return 0;
    }
    
    /* Check x86_64 */
    if (ehdr->e_machine != 0x3E) {  /* EM_X86_64 */
        return 0;
    }
    
    return 1;
}

static uint64_t load_elf(void *elf_data) {
    Elf64_Ehdr *ehdr = (Elf64_Ehdr *)elf_data;
    
    if (!verify_elf(ehdr)) {
        puts("ERROR: Invalid ELF file\n");
        return 0;
    }
    
    puts("Loading ELF segments...\n");
    
    /* Load program headers */
    Elf64_Phdr *phdr = (Elf64_Phdr *)((uint8_t *)elf_data + ehdr->e_phoff);
    
    for (int i = 0; i < ehdr->e_phnum; i++) {
        if (phdr[i].p_type != PT_LOAD) continue;
        
        puts("  Segment ");
        put_hex(i);
        puts(" -> ");
        put_hex(phdr[i].p_paddr);
        puts("\n");
        
        /* Copy segment to physical address */
        void *dest = (void *)(uintptr_t)phdr[i].p_paddr;
        void *src = (uint8_t *)elf_data + phdr[i].p_offset;
        
        memcpy(dest, src, phdr[i].p_filesz);
        
        /* Zero BSS */
        if (phdr[i].p_memsz > phdr[i].p_filesz) {
            size_t bss_size = phdr[i].p_memsz - phdr[i].p_filesz;
            memset((uint8_t *)dest + phdr[i].p_filesz, 0, bss_size);
        }
    }
    
    puts("Entry point: ");
    put_hex(ehdr->e_entry);
    puts("\n");
    
    return ehdr->e_entry;
}

/* ========================================================================
 * Memory Detection
 * ======================================================================== */

static struct E820Entry e820_map[32];
static int e820_count = 0;

static void detect_memory(struct BootInfo *bi) {
    /* Try E820 first */
    e820_count = bios_detect_memory_e820(e820_map, 32);
    
    if (e820_count > 0) {
        bi->mmap = e820_map;
        bi->mmap_count = e820_count;
        
        /* Calculate mem_lower and mem_upper from E820 */
        bi->mem_lower = 0;
        bi->mem_upper = 0;
        
        for (int i = 0; i < e820_count; i++) {
            if (e820_map[i].type != 1) continue;  /* Type 1 = available */
            
            if (e820_map[i].base < 0x100000) {
                /* Below 1MB */
                bi->mem_lower = e820_map[i].length / 1024;
            } else {
                /* Above 1MB */
                bi->mem_upper += e820_map[i].length / 1024;
            }
        }
        
        puts("Memory detected: ");
        put_hex(bi->mem_lower);
        puts(" KB low, ");
        put_hex(bi->mem_upper);
        puts(" KB high\n");
    } else {
        /* Fallback: assume 640KB low, 31MB high */
        bi->mem_lower = 640;
        bi->mem_upper = 31 * 1024;
        bi->mmap = NULL;
        bi->mmap_count = 0;
        puts("WARNING: Using fallback memory detection\n");
    }
}

/* ========================================================================
 * Main GuaBoot Stage 2 Entry
 * ======================================================================== */

void guaboot2_main(void) {
    static const char cmdline[] = "root=/dev/ram0 debug=true";
    static struct Module modules[] = {
        {
            .mod_start = 0x00200000,
            .mod_end   = 0x00200000 + 4096,
            .string    = "test_module",
            .reserved  = 0,
        },
    };
    static struct BootMmapEntry mmap_entries[] = {
        /* [0x00000000 – 0x00100000] = RESERVED */
        { .base = 0x00000000, .length = 0x00100000, .type = 2, .reserved = 0 },
        /* [0x00100000 – 0x08000000] = USABLE (127 MB) */
        { .base = 0x00100000, .length = 0x07F00000, .type = 1, .reserved = 0 },
    };
    puts("\n");
    puts("================================================================================\n");
    puts("GuaBoot 1.0 - Stage 2 (BSD 3-Clause License)\n");
    puts("================================================================================\n");
    
    /* Allocate kernel buffer at 2MB */
    void *kernel_buffer = (void *)0x200000;
    
    /* Read kernel from disk */
    puts("Loading /boot/kernel.elf...\n");
    if (read_kernel(kernel_buffer, 512 * 1024) < 0) {
        puts("ERROR: Failed to read kernel\n");
        goto halt;
    }
    
    /* Load ELF */
    uint64_t entry = load_elf(kernel_buffer);
    if (entry == 0) {
        goto halt;
    }

    /* Compute kernel CRC over loaded segments */
    bi->kernel_crc32 = compute_kernel_crc(kernel_buffer);
    puts("Kernel CRC32: 0x");
    put_hex(bi->kernel_crc32);
    puts("\n");
    
    /* Build BootInfo */
    puts("Building boot information...\n");
    struct BootInfo *bi = (struct BootInfo *)BOOT_INFO_ADDR;
    
    bi->magic = GBSD_MAGIC;
    bi->version = 0x00010000;
    bi->size = sizeof(struct BootInfo);
    bi->boot_device = 0x80;  /* First hard disk */
    bi->cmdline = (char *)cmdline;
    bi->mods_count = sizeof(modules) / sizeof(modules[0]);
    bi->mods = modules;
    bi->mmap = mmap_entries;
    bi->mmap_count = sizeof(mmap_entries) / sizeof(mmap_entries[0]);
    bi->mem_lower = 1024;      /* 1MB below */
    bi->mem_upper = 127 * 1024;/* 127MB above 1MB */
    
    /* Intentionally skip BIOS/UEFI detection: use hard-coded map */
    
    puts("Switching to 64-bit mode...\n");
    
    /* Switch to long mode and jump to kernel */
    /* This function does NOT return */
    switch_to_long_mode(entry, (uint64_t)bi);
    
halt:
    puts("\nSystem halted.\n");
    __asm__ volatile("cli; hlt");
    while (1);
}

/* ========================================================================
 * Panic Handler
 * ======================================================================== */

void panic(const char *msg) {
    puts("\n\nPANIC: ");
    puts(msg);
    puts("\n");
    __asm__ volatile("cli; hlt");
    while (1);
}
