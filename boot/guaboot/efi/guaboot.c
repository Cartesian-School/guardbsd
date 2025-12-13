/*
 * Project: GuardBSD Winter Saga version 1.0.0
 * Package: guaboot_efi
 * Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
 * License: BSD-3-Clause
 *
 * Minimalna aplikacja EFI GuaBoot (stub, zwraca sukces).
 */

typedef unsigned long long UINTN;
typedef UINTN EFI_STATUS;
typedef void *EFI_HANDLE;
typedef struct EFI_SYSTEM_TABLE EFI_SYSTEM_TABLE;

#define EFI_SUCCESS 0

// System table structure (minimal)
struct EFI_SYSTEM_TABLE {
    char _pad1[60];
    void *BootServices;
    char _pad2[24];
    void *ConOut;
};

EFI_STATUS efi_main(EFI_HANDLE image, EFI_SYSTEM_TABLE *system_table) {
    // Minimal EFI stub - just return success
    // Actual booting is handled by the BIOS bootloader chain
    (void)image;
    (void)system_table;

    return EFI_SUCCESS;
}
