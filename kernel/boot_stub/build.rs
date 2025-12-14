use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let mut libs = Vec::new();
    libs.push(compile_asm(
        &out_dir,
        "src/guaboot_entry.S",
        "guaboot_entry",
    ));
    libs.push(compile_asm(
        &out_dir,
        "../interrupt/keyboard_irq.S",
        "keyboard_irq",
    ));
    libs.push(compile_asm(
        &out_dir,
        "../interrupt/timer_irq.S",
        "timer_irq",
    ));
    libs.push(compile_asm(
        &out_dir,
        "../interrupt/syscall_entry.S",
        "syscall_entry",
    ));
    libs.push(compile_asm(
        &out_dir,
        "../process/context_amd64.S",
        "context",
    ));

    println!("cargo:rustc-link-arg=--whole-archive");
    for lib in libs {
        println!("cargo:rustc-link-arg={}", lib.display());
    }
    println!("cargo:rustc-link-arg=--no-whole-archive");
}

fn compile_asm(out_dir: &Path, src: &str, name: &str) -> PathBuf {
    let path = Path::new(src);
    let mut build = cc::Build::new();
    if env::var("DEBUG_HANDOFF").is_ok() {
        build.define("DEBUG_HANDOFF", None);
    }
    build
        .cargo_metadata(false)
        .file(path)
        .flag("-m64")
        .compile(name);

    println!("cargo:rerun-if-changed={}", path.display());

    out_dir.join(format!("lib{}.a", name))
}
