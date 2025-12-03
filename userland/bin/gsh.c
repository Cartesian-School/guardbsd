// GuardBSD Shell - Minimal ELF Binary
// BSD 3-Clause License

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
