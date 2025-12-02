#!/bin/bash
# GuardBSD - Complete Quick Start Script
# This script sets up everything from scratch
# Run on Debian 12 / Ubuntu 22.04+

set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         GuardBSD Quick Start Setup Script                    ║"
echo "║         Version 1.0.0 - November 2025                        ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
GITHUB_USERNAME="${GITHUB_USERNAME:-your-username}"
REPO_NAME="guardbsd"
GITHUB_TOKEN="${GITHUB_TOKEN:-}"

# Functions
check_command() {
    if command -v $1 &> /dev/null; then
        echo -e "${GREEN}✓${NC} $1 is installed"
        return 0
    else
        echo -e "${RED}✗${NC} $1 is not installed"
        return 1
    fi
}

install_rust() {
    echo ""
    echo "=== Installing Rust ===="
    if ! check_command rustc; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source $HOME/.cargo/env
    fi
    
    rustup toolchain install nightly
    rustup default nightly
    rustup component add rust-src rustfmt clippy
    rustup target add x86_64-cartesian-none aarch64-cartesian-none
    
    echo -e "${GREEN}✓${NC} Rust installation complete"
}

install_dependencies() {
    echo ""
    echo "=== Installing System Dependencies ==="
    
    sudo apt-get update
    sudo apt-get install -y \
        build-essential \
        git \
        curl \
        qemu-system-x86 \
        qemu-system-aarch64 \
        xorriso \
        mtools \
        gcc-multilib \
        gcc-aarch64-linux-gnu \
        python3 \
        python3-pip \
        pkg-config \
        libssl-dev \
        dtc
    
    # Python packages
    pip3 install requests
    
    echo -e "${GREEN}✓${NC} Dependencies installed"
}

create_repository() {
    echo ""
    echo "=== Creating GitHub Repository ==="
    
    if [ -z "$GITHUB_TOKEN" ]; then
        echo -e "${YELLOW}⚠${NC}  GITHUB_TOKEN not set. Please create repository manually."
        echo "   Go to: https://github.com/new"
        echo "   Name: $REPO_NAME"
        read -p "Press Enter when repository is created..."
    else
        # Create via GitHub API
        curl -X POST https://api.github.com/user/repos \
            -H "Authorization: token $GITHUB_TOKEN" \
            -H "Accept: application/vnd.github.v3+json" \
            -d "{\"name\":\"$REPO_NAME\",\"description\":\"GuardBSD - Multi-Microkernel Operating System\",\"license_template\":\"bsd-3-clause\",\"auto_init\":false}"
        
        echo -e "${GREEN}✓${NC} Repository created"
    fi
}

clone_and_setup() {
    echo ""
    echo "=== Setting Up Project Structure ==="
    
    # Clone or init
    if [ -d "$REPO_NAME" ]; then
        echo "Directory $REPO_NAME already exists"
        cd $REPO_NAME
    else
        git clone https://github.com/$GITHUB_USERNAME/$REPO_NAME.git || {
            mkdir $REPO_NAME
            cd $REPO_NAME
            git init
            git branch -M main
        }
    fi
    
    # Create directory structure
    mkdir -p kernel/microkernels/{uk_time,uk_space,uk_ipc}/src/arch/{x86_64,aarch64}
    mkdir -p userland/{init,shell}/src
    mkdir -p servers/{netd,sshed,httpd,vfs,gpu}/src
    mkdir -p boot/limine
    mkdir -p tools
    mkdir -p docs/{architecture,api,guides}
    mkdir -p tests/{unit,integration}
    mkdir -p .github/workflows
    mkdir -p targets
    mkdir -p build
    
    echo -e "${GREEN}✓${NC} Project structure created"
}

create_gitignore() {
    cat > .gitignore <<'EOF'
# Rust
target/
Cargo.lock
**/*.rs.bk
*.pdb

# Build artifacts
build/
*.iso
*.bin
*.img
*.elf

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Logs
*.log
*.out

# QEMU
*.qcow2

# Python
__pycache__/
*.pyc
.pytest_cache/

# Secrets
.env
*.pem
*.key
EOF
    echo -e "${GREEN}✓${NC} .gitignore created"
}

create_cargo_toml() {
    cat > Cargo.toml <<'EOF'
[workspace]
resolver = "2"
members = [
    "kernel/microkernels/uk_time",
    "kernel/microkernels/uk_space",
    "kernel/microkernels/uk_ipc",
    "userland/init",
    "userland/shell",
    "servers/netd",
    "servers/sshed",
    "servers/httpd",
    "servers/vfs",
    "servers/gpu",
]

[workspace.package]
version = "1.0.0"
edition = "2021"
license = "BSD-3-Clause"
authors = ["Cartesian School"]

[profile.release]
opt-level = "z"
lto = true
panic = "abort"
codegen-units = 1
strip = true

[profile.dev]
opt-level = 0
debug = true
EOF
    echo -e "${GREEN}✓${NC} Cargo.toml created"
}

create_target_specs() {
    # x86_64 target
    cat > targets/x86_64-cartesian-gbsd.json <<'EOF'
{
  "llvm-target": "x86_64-cartesian-none",
  "data-layout": "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128",
  "arch": "x86_64",
  "target-endian": "little",
  "target-pointer-width": "64",
  "target-c-int-width": "32",
  "os": "none",
  "executables": true,
  "linker-flavor": "ld.lld",
  "linker": "rust-lld",
  "panic-strategy": "abort",
  "disable-redzone": true,
  "features": "-mmx,-sse,+soft-float"
}
EOF

    # aarch64 target
    cat > targets/aarch64-cartesian-gbsd.json <<'EOF'
{
  "llvm-target": "aarch64-cartesian-none",
  "data-layout": "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128",
  "arch": "aarch64",
  "target-endian": "little",
  "target-pointer-width": "64",
  "target-c-int-width": "32",
  "os": "none",
  "executables": true,
  "linker-flavor": "ld.lld",
  "linker": "rust-lld",
  "panic-strategy": "abort",
  "disable-redzone": true,
  "features": "+strict-align,+neon,+fp-armv8"
}
EOF
    echo -e "${GREEN}✓${NC} Target specifications created"
}

create_minimal_microkernels() {
    echo ""
    echo "=== Creating Minimal Microkernels ==="
    
    # uk_space
    cat > kernel/microkernels/uk_space/Cargo.toml <<'EOF'
[package]
name = "uk_space"
version.workspace = true
edition.workspace = true

[dependencies]
EOF

    cat > kernel/microkernels/uk_space/src/main.rs <<'EOF'
#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // µK-Space entry point
    unsafe {
        core::arch::asm!("hlt", options(noreturn));
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}
EOF

    # uk_time
    cat > kernel/microkernels/uk_time/Cargo.toml <<'EOF'
[package]
name = "uk_time"
version.workspace = true
edition.workspace = true

[dependencies]
EOF

    cat > kernel/microkernels/uk_time/src/main.rs <<'EOF'
#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // µK-Time entry point
    unsafe {
        core::arch::asm!("hlt", options(noreturn));
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}
EOF

    # uk_ipc
    cat > kernel/microkernels/uk_ipc/Cargo.toml <<'EOF'
[package]
name = "uk_ipc"
version.workspace = true
edition.workspace = true

[dependencies]
EOF

    cat > kernel/microkernels/uk_ipc/src/main.rs <<'EOF'
#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // µK-IPC entry point
    unsafe {
        core::arch::asm!("hlt", options(noreturn));
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}
EOF

    echo -e "${GREEN}✓${NC} Minimal microkernels created"
}

create_build_script() {
    cat > tools/quick_build.sh <<'EOF'
#!/bin/bash
set -e

ARCH="${1:-x86_64}"
TARGET="${ARCH}-cartesian-gbsd"

echo "Building GuardBSD for ${ARCH}..."

# Build each microkernel
for uk in uk_time uk_space uk_ipc; do
    echo "→ Building ${uk}..."
    (cd "kernel/microkernels/${uk}" && \
     cargo build --release --target "../../../targets/${TARGET}.json")
done

echo "✓ Build complete"
echo ""
echo "Binary sizes:"
size "target/${TARGET}/release/uk_"* || true
EOF
    chmod +x tools/quick_build.sh
    
    echo -e "${GREEN}✓${NC} Build script created"
}

create_readme() {
    cat > README.md <<'EOF'
# GuardBSD

Multi-Microkernel Operating System in Rust

## Quick Start

```bash
# Build
./tools/quick_build.sh x86_64

# Test (when bootloader ready)
./tools/test_x86_64.sh
```

## Documentation

See `docs/` directory.

## License

BSD 3-Clause
EOF
    echo -e "${GREEN}✓${NC} README created"
}

initial_commit() {
    echo ""
    echo "=== Creating Initial Commit ==="
    
    git add .
    git commit -m "chore: initial project structure

- Set up workspace with three microkernels
- Add target specifications for x86_64 and aarch64
- Create minimal build scripts
- Add project documentation structure
" || echo "Nothing to commit"
    
    echo -e "${GREEN}✓${NC} Initial commit created"
}

test_build() {
    echo ""
    echo "=== Testing Build ==="
    
    ./tools/quick_build.sh x86_64
    
    echo ""
    echo -e "${GREEN}✓${NC} Build test successful!"
}

# Main execution
main() {
    echo "Starting setup..."
    echo ""
    
    # Check if running on supported OS
    if [ ! -f /etc/debian_version ] && [ ! -f /etc/lsb-release ]; then
        echo -e "${YELLOW}⚠${NC}  This script is designed for Debian/Ubuntu."
        echo "   It may not work on your system."
        read -p "Continue anyway? (y/n) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
    
    # Prerequisites check
    echo "=== Checking Prerequisites ==="
    check_command git || { echo "Please install git first"; exit 1; }
    
    # Install everything
    install_dependencies
    install_rust
    
    # Setup project
    create_repository
    clone_and_setup
    
    # Create files
    create_gitignore
    create_cargo_toml
    create_target_specs
    create_minimal_microkernels
    create_build_script
    create_readme
    
    # Commit and test
    initial_commit
    test_build
    
    # Final message
    echo ""
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║              Setup Complete! 🎉                              ║"
    echo "╠══════════════════════════════════════════════════════════════╣"
    echo "║  Next steps:                                                 ║"
    echo "║  1. Push to GitHub: git push -u origin main                  ║"
    echo "║  2. Import issues: python3 import_issues.py                  ║"
    echo "║  3. Start development: see docs/getting-started.md           ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo ""
    echo "Project location: $(pwd)"
    echo ""
}

# Run main
main
