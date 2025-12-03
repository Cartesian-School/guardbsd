// Syscall Test Program
// BSD 3-Clause License

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
