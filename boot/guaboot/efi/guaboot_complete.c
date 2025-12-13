/*
 * Project: GuardBSD Winter Saga version 1.0.0
 * Package: guaboot_efi
 * Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
 * License: BSD-3-Clause
 *
 * UEFI loader GuaBoot ładujący jądro GuardBSD (minimalny).
 */

#include <efi.h>
#include <efilib.h>

/* ========================================================================
 * GuardBSD Boot Protocol Definitions
 * ======================================================================== */

#define GBSD_MAGIC 0x42534447  /* "GBSD" */

struct BootInfo {
    UINT32 magic;           /* 0x42534447 */
    UINT32 version;         /* 0x00010000 */
    UINT32 size;            /* sizeof(struct BootInfo) */
    UINT32 kernel_crc32;    /* CRC32 of loaded kernel PT_LOAD segments */
    UINT64 mem_lower;       /* Memory below 1MB (KB) */
    UINT64 mem_upper;       /* Memory above 1MB (KB) */
    UINT32 boot_device;     /* Boot device ID */
    CHAR8 *cmdline;         /* Kernel command line */
    UINT32 mods_count;      /* Number of modules */
    struct Module *mods;    /* Module array */
    struct BootMmapEntry *mmap; /* Memory map */
    UINT32 mmap_count;      /* Memory map entries */
} __attribute__((packed));

struct Module {
    UINT64 mod_start;
    UINT64 mod_end;
    CHAR8 *string;
    UINT32 reserved;
} __attribute__((packed));

struct BootMmapEntry {
    UINT64 base;
    UINT64 length;
    UINT32 typ;      /* 1 = usable */
    UINT32 reserved;
} __attribute__((packed));

/* ========================================================================
 * ELF Definitions
 * ======================================================================== */

#define EI_NIDENT 16
#define PT_LOAD 1

typedef struct {
    UINT8 e_ident[EI_NIDENT];
    UINT16 e_type;
    UINT16 e_machine;
    UINT32 e_version;
    UINT64 e_entry;
    UINT64 e_phoff;
    UINT64 e_shoff;
    UINT32 e_flags;
    UINT16 e_ehsize;
    UINT16 e_phentsize;
    UINT16 e_phnum;
    UINT16 e_shentsize;
    UINT16 e_shnum;
    UINT16 e_shstrndx;
} Elf64_Ehdr;

typedef struct {
    UINT32 p_type;
    UINT32 p_flags;
    UINT64 p_offset;
    UINT64 p_vaddr;
    UINT64 p_paddr;
    UINT64 p_filesz;
    UINT64 p_memsz;
    UINT64 p_align;
} Elf64_Phdr;

/* ========================================================================
 * Global Variables
 * ======================================================================== */

static EFI_HANDLE ImageHandle;
static EFI_SYSTEM_TABLE *SystemTable;
static EFI_BOOT_SERVICES *BS;

/* ========================================================================
 * Utility Functions
 * ======================================================================== */

static VOID *memcpy(VOID *dest, const VOID *src, UINTN n) {
    UINT8 *d = dest;
    const UINT8 *s = src;
    while (n--) *d++ = *s++;
    return dest;
}

static VOID *memset(VOID *s, INT32 c, UINTN n) {
    UINT8 *p = s;
    while (n--) *p++ = (UINT8)c;
    return s;
}

/* CRC32 (IEEE 802.3) */
static UINT32 crc32(const VOID *data, UINTN len) {
    UINT32 crc = 0xFFFFFFFF;
    const UINT8 *p = (const UINT8 *)data;
    for (UINTN i = 0; i < len; i++) {
        crc ^= p[i];
        for (int b = 0; b < 8; b++) {
            UINT32 mask = (UINT32)-(INT32)(crc & 1u);
            crc = (crc >> 1) ^ (0xEDB88320 & mask);
        }
    }
    return crc ^ 0xFFFFFFFF;
}

/* ========================================================================
 * File Loading
 * ======================================================================== */

static EFI_STATUS load_file(const CHAR16 *path, VOID **buffer, UINTN *size) {
    EFI_STATUS status;
    EFI_LOADED_IMAGE *loaded_image;
    EFI_SIMPLE_FILE_SYSTEM_PROTOCOL *fs;
    EFI_FILE *root, *file;
    EFI_FILE_INFO *file_info;
    UINTN info_size;
    
    /* Get loaded image protocol */
    status = BS->HandleProtocol(ImageHandle,
                                 &LoadedImageProtocol,
                                 (VOID **)&loaded_image);
    if (EFI_ERROR(status)) {
        Print(L"ERROR: Cannot get LoadedImageProtocol: %r\n", status);
        return status;
    }
    
    /* Get file system protocol */
    status = BS->HandleProtocol(loaded_image->DeviceHandle,
                                 &FileSystemProtocol,
                                 (VOID **)&fs);
    if (EFI_ERROR(status)) {
        Print(L"ERROR: Cannot get FileSystemProtocol: %r\n", status);
        return status;
    }
    
    /* Open volume */
    status = fs->OpenVolume(fs, &root);
    if (EFI_ERROR(status)) {
        Print(L"ERROR: Cannot open volume: %r\n", status);
        return status;
    }
    
    /* Open file */
    status = root->Open(root, &file, (CHAR16 *)path,
                        EFI_FILE_MODE_READ, 0);
    if (EFI_ERROR(status)) {
        Print(L"ERROR: Cannot open file %s: %r\n", path, status);
        root->Close(root);
        return status;
    }
    
    /* Get file size */
    info_size = sizeof(EFI_FILE_INFO) + 512;
    file_info = AllocatePool(info_size);
    if (!file_info) {
        file->Close(file);
        root->Close(root);
        return EFI_OUT_OF_RESOURCES;
    }
    
    status = file->GetInfo(file, &gEfiFileInfoGuid, &info_size, file_info);
    if (EFI_ERROR(status)) {
        Print(L"ERROR: Cannot get file info: %r\n", status);
        FreePool(file_info);
        file->Close(file);
        root->Close(root);
        return status;
    }
    
    *size = file_info->FileSize;
    FreePool(file_info);
    
    /* Allocate buffer */
    *buffer = AllocatePool(*size);
    if (!*buffer) {
        file->Close(file);
        root->Close(root);
        return EFI_OUT_OF_RESOURCES;
    }
    
    /* Read file */
    status = file->Read(file, size, *buffer);
    if (EFI_ERROR(status)) {
        Print(L"ERROR: Cannot read file: %r\n", status);
        FreePool(*buffer);
        *buffer = NULL;
        file->Close(file);
        root->Close(root);
        return status;
    }
    
    file->Close(file);
    root->Close(root);
    
    return EFI_SUCCESS;
}

/* ========================================================================
 * ELF Loader
 * ======================================================================== */

static INT32 verify_elf(Elf64_Ehdr *ehdr) {
    if (ehdr->e_ident[0] != 0x7F ||
        ehdr->e_ident[1] != 'E' ||
        ehdr->e_ident[2] != 'L' ||
        ehdr->e_ident[3] != 'F') {
        return 0;
    }
    
    if (ehdr->e_ident[4] != 2) {  /* ELFCLASS64 */
        return 0;
    }
    
    if (ehdr->e_machine != 0x3E) {  /* EM_X86_64 */
        return 0;
    }
    
    return 1;
}

static UINT32 compute_kernel_crc(void *elf_data) {
    Elf64_Ehdr *ehdr = (Elf64_Ehdr *)elf_data;
    if (!verify_elf(ehdr)) {
        return 0;
    }

    Elf64_Phdr *phdr = (Elf64_Phdr *)((UINT8 *)elf_data + ehdr->e_phoff);
    UINT32 crc = 0;

    for (UINT16 i = 0; i < ehdr->e_phnum; i++) {
        if (phdr[i].p_type != PT_LOAD) continue;
        UINT8 *seg = (UINT8 *)(UINTN)phdr[i].p_paddr;
        UINTN seg_len = (UINTN)phdr[i].p_memsz;
        crc ^= crc32(seg, seg_len);
    }

    return crc;
}

static UINT64 load_elf(VOID *elf_data, UINTN elf_size) {
    Elf64_Ehdr *ehdr = (Elf64_Ehdr *)elf_data;
    
    if (!verify_elf(ehdr)) {
        Print(L"ERROR: Invalid ELF file\n");
        return 0;
    }
    
    Print(L"Loading ELF segments...\n");
    
    Elf64_Phdr *phdr = (Elf64_Phdr *)((UINT8 *)elf_data + ehdr->e_phoff);
    
    for (UINT16 i = 0; i < ehdr->e_phnum; i++) {
        if (phdr[i].p_type != PT_LOAD) continue;
        
        Print(L"  Segment %u: 0x%lx -> 0x%lx (%lu bytes)\n",
              i, phdr[i].p_paddr, phdr[i].p_paddr + phdr[i].p_memsz,
              phdr[i].p_memsz);
        
        /* Copy segment */
        VOID *dest = (VOID *)(UINTN)phdr[i].p_paddr;
        VOID *src = (UINT8 *)elf_data + phdr[i].p_offset;
        
        memcpy(dest, src, phdr[i].p_filesz);
        
        /* Zero BSS */
        if (phdr[i].p_memsz > phdr[i].p_filesz) {
            UINTN bss_size = phdr[i].p_memsz - phdr[i].p_filesz;
            memset((UINT8 *)dest + phdr[i].p_filesz, 0, bss_size);
        }
    }
    
    Print(L"Entry point: 0x%lx\n", ehdr->e_entry);
    
    return ehdr->e_entry;
}

/* ========================================================================
 * Boot Info Construction
 * ======================================================================== */

static struct BootInfo *build_bootinfo(EFI_MEMORY_DESCRIPTOR *mmap,
                                        UINTN mmap_size,
                                        UINTN desc_size,
                                        UINT32 kernel_crc32) {
    /* Allocate BootInfo at fixed address */
    struct BootInfo *bi = AllocatePool(sizeof(struct BootInfo));
    if (!bi) return NULL;
    
    bi->magic = GBSD_MAGIC;
    bi->version = 0x00010000;
    bi->size = sizeof(struct BootInfo);
    bi->kernel_crc32 = kernel_crc32;
    bi->boot_device = 0;
    bi->cmdline = (CHAR8 *)"console=ttyS0";
    bi->mods_count = 0;
    bi->mods = NULL;
    
    /* Calculate memory from UEFI memory map */
    bi->mem_lower = 0;
    bi->mem_upper = 0;
    
    UINTN entry_count = mmap_size / desc_size;
    EFI_MEMORY_DESCRIPTOR *desc = mmap;
    struct BootMmapEntry *translated = AllocatePool(entry_count * sizeof(struct BootMmapEntry));
    if (!translated) {
        FreePool(bi);
        return NULL;
    }
    
    for (UINTN i = 0; i < entry_count; i++) {
        translated[i].base = desc->PhysicalStart;
        translated[i].length = desc->NumberOfPages * 4096;
        translated[i].typ = (desc->Type == EfiConventionalMemory) ? 1 : 2;
        translated[i].reserved = 0;

        if (desc->Type == EfiConventionalMemory) {
            UINT64 size = desc->NumberOfPages * 4096;
            
            if (desc->PhysicalStart < 0x100000) {
                bi->mem_lower += size / 1024;
            } else {
                bi->mem_upper += size / 1024;
            }
        }
        
        desc = (EFI_MEMORY_DESCRIPTOR *)((UINT8 *)desc + desc_size);
    }
    
    /* Save memory map */
    bi->mmap = translated;
    bi->mmap_count = entry_count;
    
    Print(L"Memory: %lu KB low, %lu KB high\n", bi->mem_lower, bi->mem_upper);
    
    return bi;
}

/* ========================================================================
 * Main UEFI Entry Point
 * ======================================================================== */

EFI_STATUS EFIAPI efi_main(EFI_HANDLE image_handle,
                            EFI_SYSTEM_TABLE *system_table) {
    EFI_STATUS status;
    VOID *kernel_buffer;
    UINTN kernel_size;
    UINT64 kernel_entry;
    UINT32 kernel_crc = 0;
    
    /* Initialize globals */
    ImageHandle = image_handle;
    SystemTable = system_table;
    BS = system_table->BootServices;
    
    InitializeLib(image_handle, system_table);
    
    /* Print banner */
    Print(L"\n");
    Print(L"================================================================================\n");
    Print(L"GuaBoot 1.0 UEFI (BSD 3-Clause License)\n");
    Print(L"================================================================================\n");
    Print(L"\n");
    
    /* Load kernel */
    Print(L"Loading \\boot\\kernel.elf...\n");
    status = load_file(L"\\boot\\kernel.elf", &kernel_buffer, &kernel_size);
    if (EFI_ERROR(status)) {
        Print(L"FATAL: Cannot load kernel\n");
        return status;
    }
    
    Print(L"Kernel size: %lu bytes\n", kernel_size);
    
    /* Load ELF */
    kernel_entry = load_elf(kernel_buffer, kernel_size);
    if (kernel_entry == 0) {
        Print(L"FATAL: Cannot load ELF\n");
        FreePool(kernel_buffer);
        return EFI_LOAD_ERROR;
    }
    kernel_crc = compute_kernel_crc(kernel_buffer);
    
    /* Get memory map */
    Print(L"Getting memory map...\n");
    UINTN map_key;
    UINTN map_size = 0;
    UINTN desc_size;
    UINT32 desc_version;
    EFI_MEMORY_DESCRIPTOR *mmap;
    
    /* Get size */
    status = BS->GetMemoryMap(&map_size, NULL, &map_key, &desc_size, &desc_version);
    if (status != EFI_BUFFER_TOO_SMALL) {
        Print(L"ERROR: Cannot get memory map size: %r\n", status);
        FreePool(kernel_buffer);
        return status;
    }
    
    /* Add extra space */
    map_size += 2 * desc_size;
    mmap = AllocatePool(map_size);
    if (!mmap) {
        Print(L"ERROR: Cannot allocate memory map\n");
        FreePool(kernel_buffer);
        return EFI_OUT_OF_RESOURCES;
    }
    
    /* Get actual map */
    status = BS->GetMemoryMap(&map_size, mmap, &map_key, &desc_size, &desc_version);
    if (EFI_ERROR(status)) {
        Print(L"ERROR: Cannot get memory map: %r\n", status);
        FreePool(mmap);
        FreePool(kernel_buffer);
        return status;
    }
    
    /* Build BootInfo */
    Print(L"Building boot information...\n");
    struct BootInfo *bi = build_bootinfo(mmap, map_size, desc_size, kernel_crc);
    if (!bi) {
        Print(L"ERROR: Cannot build BootInfo\n");
        FreePool(mmap);
        FreePool(kernel_buffer);
        return EFI_OUT_OF_RESOURCES;
    }
    
    /* Exit boot services */
    Print(L"Exiting boot services...\n");
    status = BS->ExitBootServices(image_handle, map_key);
    if (EFI_ERROR(status)) {
        Print(L"ERROR: Cannot exit boot services: %r\n", status);
        /* Try again with fresh memory map */
        status = BS->GetMemoryMap(&map_size, mmap, &map_key, &desc_size, &desc_version);
        if (!EFI_ERROR(status)) {
            status = BS->ExitBootServices(image_handle, map_key);
        }
        
        if (EFI_ERROR(status)) {
            Print(L"FATAL: Cannot exit boot services\n");
            return status;
        }
    }
    
    /* Jump to kernel */
    /* From this point, no UEFI services are available */
    
    typedef VOID (*kernel_entry_t)(UINT64 magic, struct BootInfo *bi);
    kernel_entry_t entry = (kernel_entry_t)kernel_entry;
    
    entry(GBSD_MAGIC, bi);
    
    /* Should never return */
    while (1) {
        __asm__ volatile("hlt");
    }
    
    return EFI_SUCCESS;
}
