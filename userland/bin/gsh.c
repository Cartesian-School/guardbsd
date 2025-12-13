/*
 * Project: GuardBSD Winter Saga version 1.0.0
 * Package: userland_gsh
 * Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
 * License: BSD-3-Clause
 *
 * Minimalny ELF powłoki (gsh) dla testów ISO.
 */

void _start() {
    const char *msg = "Shell loaded from ISO!\n";
    
    // sys_write(1, msg, 23)
    __asm__ volatile (
        "mov $1, %%eax\n"
        "mov $1, %%ebx\n"
        "mov %0, %%ecx\n"
        "mov $23, %%edx\n"
        "int $0x80"
        :
        : "r" (msg)
        : "eax", "ebx", "ecx", "edx"
    );
    
    // Loop forever
    while(1) {
        __asm__ volatile ("hlt");
    }
}
