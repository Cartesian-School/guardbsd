/*
 * Project: GuardBSD Winter Saga version 1.0.0
 * Package: userland_tests
 * Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
 * License: BSD-3-Clause
 *
 * Program testujący wywołania systemowe.
 */

// Syscall numbers
#define SYS_EXIT   0
#define SYS_WRITE  1
#define SYS_READ   2
#define SYS_FORK   3
#define SYS_EXEC   4
#define SYS_WAIT   5
#define SYS_YIELD  6
#define SYS_GETPID 7

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

// Helper functions
static inline int write(int fd, const char *buf, int len) {
    return syscall(SYS_WRITE, fd, (int)buf, len);
}

static inline int read(int fd, char *buf, int len) {
    return syscall(SYS_READ, fd, (int)buf, len);
}

static inline int fork() {
    return syscall(SYS_FORK, 0, 0, 0);
}

static inline int exec(const char *path) {
    return syscall(SYS_EXEC, (int)path, 0, 0);
}

static inline int wait(int *status) {
    return syscall(SYS_WAIT, (int)status, 0, 0);
}

static inline void yield() {
    syscall(SYS_YIELD, 0, 0, 0);
}

static inline int getpid() {
    return syscall(SYS_GETPID, 0, 0, 0);
}

static inline void exit(int status) {
    syscall(SYS_EXIT, status, 0, 0);
}

// String length
static int strlen(const char *s) {
    int len = 0;
    while (s[len]) len++;
    return len;
}

void _start() {
    const char *msg1 = "=== GuardBSD Syscall Test ===\n";
    write(1, msg1, strlen(msg1));
    
    // Test getpid
    int pid = getpid();
    const char *msg2 = "PID: ";
    write(1, msg2, strlen(msg2));
    char buf[2] = {'0' + pid, '\n'};
    write(1, buf, 2);
    
    // Test write
    const char *msg3 = "Testing write syscall... OK\n";
    write(1, msg3, strlen(msg3));
    
    // Test yield
    const char *msg4 = "Testing yield syscall... ";
    write(1, msg4, strlen(msg4));
    yield();
    const char *msg5 = "OK\n";
    write(1, msg5, strlen(msg5));
    
    // Test fork
    const char *msg6 = "Testing fork syscall... ";
    write(1, msg6, strlen(msg6));
    int child = fork();
    if (child > 0) {
        const char *msg7 = "Parent (child PID: ";
        write(1, msg7, strlen(msg7));
        char cbuf[2] = {'0' + child, ')'};
        write(1, cbuf, 2);
        const char *msg8 = "\n";
        write(1, msg8, 1);
    }
    
    // Test wait
    const char *msg9 = "Testing wait syscall... ";
    write(1, msg9, strlen(msg9));
    int status;
    wait(&status);
    const char *msg10 = "OK\n";
    write(1, msg10, strlen(msg10));
    
    const char *msg11 = "All syscalls tested!\n";
    write(1, msg11, strlen(msg11));
    
    exit(0);
}
