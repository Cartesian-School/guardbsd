/*
 * GuaBoot EFI Loader (Simplified Stub)
 * BSD 3-Clause License
 *
 * Minimal EFI application that returns success.
 * Actual loading is handled by the BIOS bootloader chain.
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
