#!/bin/bash
set -e

# Ensure we are in root
cd "$(dirname "$0")/.."

BUILD_DIR="build/x86_64"
ISO_DIR="build/iso_root"
LIMINE_DIR="boot/limine"

# Check if Limine binaries exist, if not download them (Production convenience)
if [ ! -f "$LIMINE_DIR/limine-bios.sys" ]; then
    echo ">>> Downloading Limine bootloader..."
    mkdir -p "$LIMINE_DIR"
    # In real env, use a fixed version. Here we clone latest binary branch.
    git clone https://github.com/limine-bootloader/limine.git --branch=v7.x-binary --depth=1 limine-temp
    cp limine-temp/limine-bios.sys "$LIMINE_DIR/"
    cp limine-temp/limine-bios-cd.bin "$LIMINE_DIR/"
    cp limine-temp/limine-uefi-cd.bin "$LIMINE_DIR/"
    rm -rf limine-temp
fi

# Prepare ISO structure
echo ">>> Preparing ISO root directory..."
rm -rf "$ISO_DIR" # Clean previous runs
mkdir -p "$ISO_DIR"

# Dynamically copy all microkernel ELF files from the build directory
echo ">>> Copying microkernel binaries..."
cp "$BUILD_DIR"/uk_*.elf "$ISO_DIR/"

cp "boot/limine.conf" "$ISO_DIR/"
cp "$LIMINE_DIR/limine-bios.sys" "$ISO_DIR/"
cp "$LIMINE_DIR/limine-bios-cd.bin" "$ISO_DIR/"
cp "$LIMINE_DIR/limine-uefi-cd.bin" "$ISO_DIR/"

# Create ISO
xorriso -as mkisofs -b limine-bios-cd.bin \
        -no-emul-boot -boot-load-size 4 -boot-info-table \
        --efi-boot limine-uefi-cd.bin \
        -efi-boot-part --efi-boot-image --protective-msdos-label \
        "$ISO_DIR" -o "$BUILD_DIR/gbsd-x86_64.iso"

# Install Limine to ISO
./boot/limine/limine bios-install "$BUILD_DIR/gbsd-x86_64.iso"

echo ">>> ISO Created: $BUILD_DIR/gbsd-x86_64.iso"