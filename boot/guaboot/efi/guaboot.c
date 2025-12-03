/*
 * GuaBoot EFI Loader (minimal stub)
 * BSD 3-Clause License
 *
 * This minimal implementation avoids external EFI library dependencies to
 * keep the build self-contained across architectures. It simply returns
 * EFI_SUCCESS; loading logic can be expanded when proper EFI headers/libs
 * are available in the environment.
 */

typedef unsigned long long UINTN;
typedef UINTN EFI_STATUS;
typedef void *EFI_HANDLE;
typedef struct EFI_SYSTEM_TABLE EFI_SYSTEM_TABLE;

#define EFI_SUCCESS 0

EFI_STATUS efi_main(EFI_HANDLE image, EFI_SYSTEM_TABLE *system_table) {
    (void)image;
    (void)system_table;
    return EFI_SUCCESS;
}
