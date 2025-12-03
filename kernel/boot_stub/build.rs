fn main() {
    cc::Build::new()
        .file("src/multiboot.S")
        .compile("multiboot");
    println!("cargo:rerun-if-changed=src/multiboot.S");
    
    cc::Build::new()
        .file("../interrupt/syscall_entry.S")
        .compile("syscall_entry");
    println!("cargo:rerun-if-changed=../interrupt/syscall_entry.S");
    
    cc::Build::new()
        .file("../interrupt/keyboard_irq.S")
        .compile("keyboard_irq");
    println!("cargo:rerun-if-changed=../interrupt/keyboard_irq.S");
    
    cc::Build::new()
        .file("../interrupt/timer_irq.S")
        .compile("timer_irq");
    println!("cargo:rerun-if-changed=../interrupt/timer_irq.S");
    
    cc::Build::new()
        .file("../process/context.S")
        .compile("context");
    println!("cargo:rerun-if-changed=../process/context.S");
}
