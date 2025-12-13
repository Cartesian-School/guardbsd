/*
 * Project: GuardBSD Winter Saga version 1.0.0
 * Package: userland_tests
 * Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
 * License: BSD-3-Clause
 *
 * Program testowy wywołań systemowych.
 */

// Syscall wrapper
static inline int syscall(int num, int arg1, int arg2, int arg3) {
    int ret;
    __asm__ volatile (
        "int $0x80"
        : "=a" (ret)
        : "a" (num), "b" (arg1), "c" (arg2), "d" (arg3)
        : "memory"
    );
    return ret;
}

#define SYS_WRITE 1
#define SYS_EXIT 0

void _start() {
    const char *msg = "Hello from userspace syscall!\n";
    int len = 0;
    while (msg[len]) len++;
    
    // sys_write(1, msg, len)
    syscall(SYS_WRITE, 1, (int)msg, len);
    
    // sys_exit(0)
    syscall(SYS_EXIT, 0, 0, 0);
    
    // Should never reach here
    while(1);
}
