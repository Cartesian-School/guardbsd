//! kernel/boot_stub/build.rs
//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: boot_stub
//! Copyright © 2025 Cartesian School.
//! License: BSD-3-Clause
//!
//! Build script for assembling x86_64 boot_stub entry/IRQ/context stubs.
//!
//! NOTE:
//! - exception_stubs.S is included via Rust `global_asm!` in
//!   `src/interrupt/mod.rs` and must NOT be compiled here (to avoid duplicate symbols).

use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    // Only build these ASM files for x86_64.
    if target_arch != "x86_64" {
        println!(
            "cargo:warning=boot_stub/build.rs: skipping x86_64 asm for arch={}",
            target_arch
        );
        return;
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Compile individual asm units into static archives.
    // Keep names stable: lib{name}.a
    //
    // HERE is exactly where you add your three files:
    // - src/interrupt/keyboard_irq.S
    // - src/interrupt/timer_irq.S
    // - src/interrupt/syscall_entry.S
    let libs = [
        compile_asm(&manifest_dir, "src/guaboot_entry.S", "guaboot_entry"),
        compile_asm(&manifest_dir, "src/interrupt/keyboard_irq.S", "keyboard_irq"),
        compile_asm(&manifest_dir, "src/interrupt/timer_irq.S", "timer_irq"),
        compile_asm(&manifest_dir, "src/interrupt/syscall_entry.S", "syscall_entry"),
        compile_asm(&manifest_dir, "src/process/context_amd64.S", "context"),
    ];

    // Tell rustc where to find produced static libs and link them.
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    for name in libs {
        println!("cargo:rustc-link-lib=static={}", name);
    }
}

/// Compile a single `.S` file into `lib{name}.a` in OUT_DIR.
/// Returns `name` which is used as `cargo:rustc-link-lib=static={name}`.
fn compile_asm(manifest_dir: &Path, rel_src: &str, name: &str) -> String {
    let src_path = manifest_dir.join(rel_src);

    if !src_path.exists() {
        panic!("build.rs: asm source not found: {}", src_path.display());
    }

    // Rebuild if source changes.
    println!("cargo:rerun-if-changed={}", src_path.display());

    let mut build = cc::Build::new();

    // Optional debug macro passthrough
    if env::var("DEBUG_HANDOFF").is_ok() {
        build.define("DEBUG_HANDOFF", None);
    }

    build
        .cargo_metadata(true)
        .file(&src_path)
        .flag("-m64")
        .compile(name);

    name.to_string()
}
