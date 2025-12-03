# GuardBSD

**A Modern Capability-Based Microkernel Operating System**

[![License](https://img.shields.io/badge/license-BSD--3--Clause-blue.svg)](LICENSE)
[![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)](docs/build/building.md)
[![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)](CHANGELOG.md)

---

## Overview

GuardBSD is a modern operating system featuring:

- **Microkernel Architecture** - Minimal kernel, services in userspace
- **Capability-Based Security** - Fine-grained access control
- **Native Filesystem (GuardFS)** - Modern filesystem with snapshots and compression
- **Advanced Shell (gsh)** - Feature-rich shell inspired by zsh
- **GuaBoot Bootloader** - Unified BIOS/UEFI bootloader
- **100% BSD Licensed** - No GPL components

---

## Quick Start

```bash
# Build ISO
make iso

# Test in QEMU
qemu-system-x86_64 -cdrom build/x86_64/guardbsd-saga-x86_64.iso -serial stdio -m 256M

# Or with display
qemu-system-x86_64 -cdrom build/x86_64/guardbsd-saga-x86_64.iso -m 256M
```

---

## Features

### Core System
- **GuaBoot** - Fast bootloader (<1s boot time, ~60KB size)
- **Microkernel** - Minimal kernel with IPC
- **µK-Space** - Memory management microkernel
- **µK-Time** - Scheduler microkernel
- **µK-IPC** - Inter-process communication

### Filesystem
- **GuardFS** - Native filesystem
- **Journaling** - Crash recovery
- **Snapshots** - Point-in-time state (planned)
- **Compression** - Transparent compression (planned)
- **VFS** - Virtual filesystem layer

### Shell (gsh)
- **Command History** - Persistent history
- **Tab Completion** - Intelligent completion
- **Job Control** - Background jobs (fg/bg)
- **Pipes & Redirection** - Standard I/O
- **Scripting** - Shell script support
- **Aliases & Functions** - Customization

### Services
- **VFS** - Virtual filesystem server
- **DevD** - Device manager
- **NetD** - Network stack (planned)
- **Init** - System initialization

---

## Documentation

### User Documentation
- [Getting Started](docs/user-guide/getting-started.md) - Installation and first steps
- [Shell Manual](docs/shell/gsh-manual.md) - Complete gsh reference
- [Filesystem Guide](docs/filesystem/guardfs.md) - GuardFS documentation

### Developer Documentation
- [Building](docs/build/building.md) - Build from source
- [Architecture](docs/architecture/overview.md) - System design
- [API Reference](docs/api/COMPLETE-API-REFERENCE.md) - System calls
- [Contributing](CONTRIBUTING.md) - Contribution guidelines

### Reference
- [Issues](docs/ISSUES.md) - Issue tracker
- [Roadmap](docs/PROJECT-ROADMAP.md) - Future plans
- [Changelog](CHANGELOG.md) - Version history

---

## System Requirements

### Minimum
- CPU: x86_64 or AArch64
- RAM: 256 MB
- Disk: 1 GB
- Boot: BIOS or UEFI

### Recommended
- CPU: x86_64 dual-core
- RAM: 512 MB
- Disk: 4 GB
- Boot: UEFI

---

## Building

### Prerequisites
```bash
# Ubuntu/Debian
sudo apt install build-essential rust cargo qemu-system-x86 xorriso gnu-efi

# Install Rust targets
rustup target add x86_64-unknown-none aarch64-unknown-none
```

### Build
```bash
# Clone
git clone https://github.com/guardbsd/guardbsd
cd guardbsd

# Build ISO
make iso

# Test
make test-boot
```

See [Building Guide](docs/build/building.md) for details.

---

## Architecture

```
┌─────────────────────────────────────┐
│         User Applications           │
├─────────────────────────────────────┤
│  gsh  │  libgbsd  │  Utilities     │
├─────────────────────────────────────┤
│  VFS  │  DevD  │  NetD  │  Init    │  ← Servers
├─────────────────────────────────────┤
│  µK-Space │ µK-Time │ µK-IPC       │  ← Microkernels
├─────────────────────────────────────┤
│      Minimal Microkernel            │  ← Kernel
├─────────────────────────────────────┤
│          GuaBoot                    │  ← Bootloader
└─────────────────────────────────────┘
```

---

## Performance

- **Boot Time:** <1 second
- **Bootloader Size:** ~60 KB (vs GRUB ~5 MB)
- **Memory Footprint:** ~50 MB minimum
- **Context Switch:** <1 µs

---

## License

BSD 3-Clause License

Copyright (c) 2025, GuardBSD Project
All rights reserved.

See [LICENSE](LICENSE) for details.

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## Community

- **Website:** https://guardbsd.org
- **Forum:** https://forum.guardbsd.org
- **IRC:** #guardbsd on Libera.Chat
- **Mailing List:** users@guardbsd.org

---

## Status

**Version:** 1.0.0  
**Status:** Active Development  
**License:** BSD 3-Clause  
**Platforms:** x86_64, AArch64

---

**GuardBSD - Secure, Fast, Modern**
