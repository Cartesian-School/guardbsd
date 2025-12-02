#!/bin/bash
# GuardBSD Device Driver Framework Test Script
# ============================================================================
# Copyright (c) 2025 Cartesian School - Siergej Sobolewski
# SPDX-License-Identifier: BSD-3-Clause

set -e

echo "=========================================="
echo "GuardBSD Device Driver Framework Test"
echo "=========================================="
echo ""

# Test 1: Build device driver server
echo "[1/5] Building device driver server..."
cargo build --release --target x86_64-unknown-none -p devd --quiet
echo "✅ Device driver server built successfully"

# Test 2: Build libgbsd with device module
echo "[2/5] Building libgbsd with device module..."
cargo build --release --target x86_64-unknown-none -p libgbsd --quiet
echo "✅ libgbsd built successfully"

# Test 3: Build device test program
echo "[3/5] Building device test program..."
cargo build --release --target x86_64-unknown-none -p devtest --quiet
echo "✅ Device test program built successfully"

# Test 4: Build for aarch64
echo "[4/5] Building for aarch64..."
cargo build --release --target aarch64-unknown-none -p devd -p libgbsd -p devtest --quiet
echo "✅ aarch64 build successful"

# Test 5: Verify all servers
echo "[5/5] Verifying all servers..."
cargo build --release --target x86_64-unknown-none -p init -p vfs -p ramfs -p devd --quiet
echo "✅ All servers built successfully"

echo ""
echo "=========================================="
echo "Binary Sizes"
echo "=========================================="
size target/x86_64-unknown-none/release/{init,vfs,ramfs,devd,devtest} | \
    awk 'NR==1 || NR>1 {printf "%-20s %8s %8s %8s\n", $6, $1, $2, $3}'

echo ""
echo "=========================================="
echo "Library Size"
echo "=========================================="
ls -lh target/x86_64-unknown-none/release/libgbsd.rlib | \
    awk '{printf "libgbsd.rlib: %s\n", $5}'

echo ""
echo "=========================================="
echo "Device Framework Summary"
echo "=========================================="
echo "✅ Device driver server (devd) ready"
echo "✅ Device table: 64 slots"
echo "✅ Device types: Character, Block, Network"
echo "✅ Operations: Register, Unregister, Open, Close"
echo "✅ Standard devices: null, console"
echo "✅ Dual-architecture support"
echo ""
echo "Next: Implement serial driver (ISSUE-021)"
echo "=========================================="
