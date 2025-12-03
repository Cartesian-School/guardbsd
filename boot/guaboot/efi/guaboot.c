/*
 * GuaBoot EFI Loader
 * BSD 3-Clause License
 * 
 * UEFI bootloader for GuardBSD
 */

#include <efi.h>
#include <efilib.h>

EFI_STATUS
EFIAPI
efi_main(EFI_HANDLE ImageHandle, EFI_SYSTEM_TABLE *SystemTable) {
    EFI_STATUS Status;
    EFI_INPUT_KEY Key;
    
    InitializeLib(ImageHandle, SystemTable);
    
    Print(L"GuaBoot EFI Loader v1.0.0\r\n");
    Print(L"BSD 3-Clause License\r\n\r\n");
    
    /* Load kernel from /boot/kernel.elf */
    Print(L"Loading kernel from /boot/kernel.elf...\r\n");
    
    /* TODO: Implement EFI file loading */
    /* For now, just display message */
    
    Print(L"\r\nPress any key to continue...\r\n");
    Status = SystemTable->ConIn->Reset(SystemTable->ConIn, FALSE);
    if (EFI_ERROR(Status))
        return Status;
    
    while ((Status = SystemTable->ConIn->ReadKeyStroke(SystemTable->ConIn, &Key)) == EFI_NOT_READY);
    
    return EFI_SUCCESS;
}
