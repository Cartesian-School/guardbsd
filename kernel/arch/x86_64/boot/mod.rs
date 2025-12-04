// kernel/arch/x86_64/boot/mod.rs
// Long-mode entry declarations for x86_64.

#![cfg(target_arch = "x86_64")]

extern "C" {
    /// Assembly entry that transitions from 32-bit protected mode into 64-bit long mode.
    pub fn long_mode_entry();
}
