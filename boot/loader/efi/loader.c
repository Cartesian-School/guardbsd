/*
 * Project: GuardBSD Winter Saga version 1.0.0
 * Package: loader_efi
 * Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
 * License: BSD-3-Clause
 *
 * GuardBSD loader UEFI.
 */

typedef unsigned long long UINT64;
typedef unsigned long UINTN;
typedef unsigned short CHAR16;
typedef void VOID;

typedef struct {
    UINT64 Signature;
    UINT32 Revision;
    UINT32 HeaderSize;
    UINT32 CRC32;
    UINT32 Reserved;
} EFI_TABLE_HEADER;

typedef struct {
    EFI_TABLE_HEADER Hdr;
    VOID *OutputString;
} EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL;

typedef struct {
    EFI_TABLE_HEADER Hdr;
    VOID *FirmwareVendor;
    UINT32 FirmwareRevision;
    VOID *ConsoleInHandle;
    VOID *ConIn;
    VOID *ConsoleOutHandle;
    EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL *ConOut;
} EFI_SYSTEM_TABLE;

typedef UINTN (*EFI_TEXT_STRING)(EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL *This, CHAR16 *String);

VOID efi_main(VOID *ImageHandle, EFI_SYSTEM_TABLE *SystemTable) {
    EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL *ConOut = SystemTable->ConOut;
    EFI_TEXT_STRING OutputString = (EFI_TEXT_STRING)ConOut->OutputString;
    
    CHAR16 msg[] = L"GuardBSD Loader (UEFI)\r\n";
    OutputString(ConOut, msg);
    
    // Load kernel from disk
    // Jump to kernel entry point
    
    while(1);
}
