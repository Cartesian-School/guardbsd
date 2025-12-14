//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_arch_x86_64
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause

#![cfg(target_arch = "x86_64")]
#![allow(dead_code)]

pub mod interrupts;
pub mod boot;
pub mod time;

use core::panic::PanicInfo;

/// Entry point reached from long_mode_entry.S after transitioning to 64-bit mode.
#[no_mangle]
pub extern "C" fn kernel_main_x86_64() -> ! {
    // =========== ETAP 1: DEBUG - ŻYJEMY! ===========
    unsafe {
        // Debug port 0xE9
        x86::io::outb(0xE9, b'[');
        x86::io::outb(0xE9, b'K');
        x86::io::outb(0xE9, b']');
        x86::io::outb(0xE9, b'\n');
        
        // Serial port COM1
        x86::io::outb(0x3F8 + 1, 0x00);
        x86::io::outb(0x3F8 + 3, 0x80);
        x86::io::outb(0x3F8 + 0, 0x03);
        x86::io::outb(0x3F8 + 1, 0x00);
        x86::io::outb(0x3F8 + 3, 0x03);
        x86::io::outb(0x3F8 + 2, 0xC7);
        x86::io::outb(0x3F8 + 4, 0x0B);
        
        for &byte in b"[GUARDBSD KERNEL]\n".iter() {
            x86::io::outb(0x3F8, byte);
        }
    }
    
    // =========== ETAP 2: GDT/IDT ===========
    interrupts::gdt64::init_gdt64();
    interrupts::idt64::init_idt64();
    
    unsafe { x86::io::outb(0xE9, b'G'); }
    
    // =========== ETAP 3: PRÓBA WYWOŁANIA MIKROJĄDER ===========
    // Bootloader ładuje je pod jakimiś adresami, ale nie wiemy gdzie...
    
    unsafe { x86::io::outb(0xE9, b'T'); }
    unsafe { x86::io::outb(0xE9, b'R'); }
    unsafe { x86::io::outb(0xE9, b'Y'); }
    unsafe { x86::io::outb(0xE9, b' '); }
    unsafe { x86::io::outb(0xE9, b'M'); }
    unsafe { x86::io::outb(0xE9, b'I'); }
    unsafe { x86::io::outb(0xE9, b'K'); }
    unsafe { x86::io::outb(0xE9, b'\n'); }
    
    // Próbujemy znaleźć mikrojądra w pamięci
    // Bootloader zwykle ładuje moduły po kernelu
    try_call_microkernels();
    
    // =========== ETAP 4: GŁÓWNA PĘTLA ===========
    unsafe { 
        x86::io::outb(0xE9, b'L');
        x86::io::outb(0xE9, b'O');
        x86::io::outb(0xE9, b'O');
        x86::io::outb(0xE9, b'P');
        x86::io::outb(0xE9, b'\n');
    }
    
    let mut counter = 0;
    loop {
        unsafe {
            if counter % 100_000_000 == 0 {
                x86::io::outb(0xE9, b'.');
                counter = 0;
            }
            counter += 1;
            core::arch::asm!("pause");
        }
    }
}

/// Próbuje znaleźć i wywołać mikrojądra
fn try_call_microkernels() {
    unsafe {
        // Bootloader powinien przekazać informacje o modułach
        // Zobaczmy czy mamy strukturę multiboot/modules
        
        // Tymczasowo: po prostu wypisz że próbujemy
        for &byte in b"Looking for microkernels...\n".iter() {
            x86::io::outb(0x3F8, byte);
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        for &byte in b"KERNEL PANIC: ".iter() {
            x86::io::outb(0xE9, byte);
        }
        
        if let Some(msg) = info.message() {
            for byte in msg.as_str().bytes() {
                x86::io::outb(0xE9, byte);
            }
        }
        
        x86::io::outb(0xE9, b'\n');
    }
    
    loop {
        unsafe { core::arch::asm!("hlt") };
    }
}