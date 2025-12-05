<p align="center">
  <img src="https://www.guardbsd.org/images/logo-main.png" alt="GuardBSD Logo" width="180">
</p>

<h1 align="center">GuardBSD</h1>
<p align="center"><strong>Rust Multi-Microkernel OS • Capability Security • Production-Ready</strong></p>

<p align="center">
  <img src="https://img.shields.io/badge/license-BSD--3--Clause-blue.svg">
  <img src="https://img.shields.io/badge/build-passing-brightgreen.svg">
  <img src="https://img.shields.io/badge/release-Winter%20Saga-blue.svg">
  <img src="https://img.shields.io/badge/platform-x86_64-orange.svg">
  <img src="https://img.shields.io/badge/fosdem-2026%20premiere-blue.svg">
</p>

<p align="center">
  <a href="https://www.guardbsd.org">Website</a> •
  <a href="https://github.com/Cartesian-School/guardbsd">GitHub</a> •
  <a href="https://x.com/GuardBSD">X (@GuardBSD)</a>
</p>

<br>

---

## Overview

**GuardBSD** is a **multi-microkernel operating system in Rust** with **capability-based security**, **TCB < 8,000 lines**, **full storage stack**, and a **100% BSD-licensed bootloader**.

**Winter Saga** - the **first production-ready release**:
- **GuaBoot**: **100% BSD** bootloader (BIOS + UEFI), **<1 second**, **~60 KB**
- **gsh**: **zsh-level** shell (scripting, job control, tab completion)
- **GuardFS + GuardZFS**: native FS + ZFS features in **1,543 lines**
- **23 partition types**, **swap**, **ATA**, **block cache**
- **API Reference** and **Architecture Docs** - **fully documented**

<br>

---

## Key Features

| Component | Status | Description |
|---------|--------|--------|
| **GuaBoot** | Production-Ready | **100% BSD**, BIOS/UEFI, ELF64, FreeBSD protocol, **~2,650 LOC** |
| **gsh Shell** | Production-Ready | zsh-level: history, job control, scripts, aliases, functions |
| **GuardZFS** | Production-Ready | Pools, RAID-Z1/Z2, COW, snapshots, SHA-256, **1,543 LOC** |
| **GuardFS** | Production-Ready | Journaling, snapshots, LZ4, COW, **~3,200 LOC** |
| **Disk I/O** | Production-Ready | ATA, cache (128 blocks), 23 partition types, swap |
| **Swap** | Production-Ready | Auto-detection, up to 4, `swapon`/`swapoff` |
| **Three Microkernels** | **FOSDEM 2026** | `µK-Time`, `µK-Space`, `µK-IPC` - **source closed until premiere** |

> **Microkernels are not public** - **world premiere at FOSDEM 2026**

<br>

---

## Quick Start

```bash
# Clone
git clone https://github.com/Cartesian-School/guardbsd
cd guardbsd

# Build ISO
make iso

# Run
qemu-system-x86_64 -cdrom build/x86_64/guardbsd-winter-saga.iso -m 2G -smp 4 -serial stdio
```

ISO: [guardbsd.org/download](https://www.guardbsd.org)

<br>

---

## GuaBoot - 100% BSD Bootloader

**No GRUB. No Multiboot. Only BSD.**

```
BIOS → guaboot1 (512B) → guaboot2 (~32KB) → Long Mode → kernel.elf
UEFI → guaboot.efi (~50KB) → kernel.elf
```

- **BIOS + UEFI**
- **64-bit native**
- **ELF64 PT_LOAD**
- **E820 / UEFI memory map**
- **BootInfo** (FreeBSD-compatible)
- **Serial debug (COM1)**
- **~2,650 lines (C + ASM)**

```text
RDI = 0x42534447 ("GBSD")
RSI = *BootInfo
```

<br>

---

## gsh - Full-Featured Shell (zsh-level)

```text
$ zfs create tank/data
$ ls | grep txt > result.txt
$ sleep 10 &
$ jobs; fg %1
$ alias ll='ls -la'
$ greet() { echo "Hi, $1"; }; greet World
```

- **History**: `↑↓`, `Ctrl+R`, `!123`
- **Job Control**: `&`, `fg`, `bg`, `jobs`
- **Pipes**: `|`, `>`, `>>`, `<`, `2>`
- **Scripts**: `#!/usr/bin/gsh`
- **Tab Completion**: commands, paths, variables

<br>

---

## Filesystems

| Feature | GuardFS | GuardZFS |
|--------|--------|---------|
| Journaling | Yes | Yes (TXG) |
| Snapshots | Yes | Yes |
| Compression | Yes (LZ4) | Yes (LZ4) |
| Pool | No | Yes |
| RAID-Z | No | Yes (Z1/Z2) |
| Self-Healing | No | Yes (SHA-256) |
| Code | ~3.2K LOC | 1.5K LOC |

```bash
zfs create tank/data
mkfs.guardfs /dev/disk0p2
mount -t guardfs /dev/disk0p2 /mnt
```

<br>

---

## Disk & Storage

- **23 partition types**: FAT, NTFS, ext4, FreeBSD, HFS+, GPT, EFI
- **Swap**: up to 4, `swapon 0`, auto-detection
- **ATA**: LBA48, PIO, IDENTIFY
- **Cache**: 128 × 4KB, LRU, write-back

```text
Partition 1: Linux Swap - 2048 MB [Swap]
Partition 2: GuardZFS - 20480 MB
```

<br>

---

## Architecture

```
┌────────────────────────────────────────────┐
│             User Applications              │
├────────────────────────────────────────────┤
│    gsh │ zfs │ mkfs.guardfs │ top          │
├────────────────────────────────────────────┤
│  VFS │ GuardFS │ GuardZFS │ Init │ NetD    │
├────────────────────────────────────────────┤
│   µK-Space   │   µK-Time   │   µK-IPC      │
├────────────────────────────────────────────┤
│              System Call Layer             │
├────────────────────────────────────────────┤
│                  GuaBoot                   │
└────────────────────────────────────────────┘
```

<br>

---

## API Reference

**Full API** - [docs/api/REFERENCE.md](docs/api/REFERENCE.md)

### Examples

```rust
// IPC
let port = port_create()?;
ipc_send(port, &Message::new(1, b"ping"))?;

// Files
let fd = open("/data", O_RDWR)?;
write(fd, b"GuardBSD")?;
close(fd)?;

// Memory
let paddr = pmm_alloc()?;
vmm_map(0x100000, paddr, READ | WRITE)?;
```

<br>

---

## System Requirements

| | Minimum | Recommended |
|---|---|---|
| CPU | x86_64 | dual-core |
| RAM | 256 MB | 2 GB |
| Disk | 1 GB | 4 GB+ |
| Boot | UEFI/BIOS | UEFI |

<br>

---

## Building

```bash
# Debian/Ubuntu
sudo apt install build-essential rustc cargo qemu-system-x86 xorriso gnu-efi nasm

rustup target add x86_64-unknown-none

make iso
make test-boot
```

<br>

---

## Roadmap

| Milestone | Date | Status |
|------|------|--------|
| Winter Saga | Jan 2026 | Done |
| FOSDEM 2026 | Feb 2026 | Planned |
| Microkernels Open | Post-FOSDEM | Planned |
| ARM64 | Q2 2026 | In Progress |
| RISC-V | Q3 2026 | Planned |

<br>

---

## Documentation

- **[guardbsd.org](https://www.guardbsd.org)** – ISO, news
- **gsh Manual** – [docs/shell/gsh-manual.md](docs/shell/gsh-manual.md)
- **GuaBoot Reference** – [boot/guaboot/REFERENCE.md](boot/guaboot/REFERENCE.md)
- **API Reference** – [docs/api/REFERENCE.md](docs/api/REFERENCE.md)
- **Architecture** – [docs/architecture/OVERVIEW.md](docs/architecture/OVERVIEW.md)

<br>

---

## Performance

| Metric | Value |
|--------|--------|
| **GuaBoot** | < 1 s |
| Bootloader | ~60 KB |
| Memory | ~50 MB |
| Context | < 1 µs |
| IPC | ~180 cycles |
| GuardZFS | ~50 MB/s (SHA-256) |
| GuardFS | ~500 MB/s (LZ4) |

<br>

---

## License

```
BSD 3-Clause License
Copyright (c) 2025 Siergej Sobolewski, Cartesian School
```

[LICENSE](LICENSE)

<br>

---

## Contributing

- [CONTRIBUTING.md](CONTRIBUTING.md)
- Open an Issue / PR

<br>

---

## Community

- **Web**: https://www.guardbsd.org  
- **GitHub**: https://github.com/Cartesian-School/guardbsd  
- **X**: [@GuardBSD](https://x.com/GuardBSD)  
- **Email**: ssobo77@gmail.com  
- **FOSDEM 2026**: Microkernel Devroom - **microkernel premiere**

<br>

---

<p align="center">
  <strong>GuardBSD - Security. Minimalism. The Future.</strong>
</p>
```

<br>
