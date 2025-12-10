fn main() {
    // Build GuaBoot entry point (FreeBSD-style, NO multiboot)
    cc::Build::new()
        .file("src/guaboot_entry.S")
        .compile("guaboot_entry");
    println!("cargo:rerun-if-changed=src/guaboot_entry.S");

    // Long mode transition not yet implemented
    // cc::Build::new()
    //     .file("src/long_mode.S")
    //     .compile("long_mode");
    // println!("cargo:rerun-if-changed=src/long_mode.S");

    // Syscall entry is 64-bit only, not needed for 32-bit boot stub
    // cc::Build::new()
    //     .file("../interrupt/syscall_entry.S")
    //     .compile("syscall_entry");
    // println!("cargo:rerun-if-changed=../interrupt/syscall_entry.S");

    cc::Build::new()
        .file("../interrupt/keyboard_irq.S")
        .compile("keyboard_irq");
    println!("cargo:rerun-if-changed=../interrupt/keyboard_irq.S");

    cc::Build::new()
        .file("../interrupt/timer_irq.S")
        .compile("timer_irq");
    println!("cargo:rerun-if-changed=../interrupt/timer_irq.S");

    cc::Build::new()
        .file("../interrupt/syscall_entry.S")
        .flag("-m64")
        .compile("syscall_entry");
    println!("cargo:rerun-if-changed=../interrupt/syscall_entry.S");

    cc::Build::new()
        .file("../process/context_amd64.S")
        .flag("-m64")
        .compile("context");
    println!("cargo:rerun-if-changed=../process/context_amd64.S");
}
