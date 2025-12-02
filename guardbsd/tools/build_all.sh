#!/bin/bash
set -e

# Usage: ./tools/build_all.sh [x86_64|aarch64]

ARCH=$1
if [ -z "$ARCH" ]; then
    echo "Usage: $0 [x86_64|aarch64]"
    exit 1
fi

ROOT_DIR=$(pwd)
BUILD_DIR="$ROOT_DIR/build/$ARCH"
TARGET_JSON="$ROOT_DIR/targets/${ARCH}-cartesian-gbsd.json"

echo ">>> Building GuardBSD for $ARCH..."

# 1. Clean/Prepare
mkdir -p "$BUILD_DIR"

# 2. Build Microkernels (Workspace)
echo ">>> Compiling Microkernels..."
cargo build --release --workspace --target "$TARGET_JSON"

# 3. Dynamically Copy Artifacts
echo ">>> Copying ELF files..."

# Find all microkernel members ('uk_*') from the root Cargo.toml
UK_MEMBERS=$(grep 'uk_' "$ROOT_DIR/Cargo.toml" | sed 's/.*"\(.*\)"/\1/' | xargs -n1 basename)

for uk in $UK_MEMBERS; do
    SOURCE_PATH="$ROOT_DIR/target/${ARCH}-cartesian-gbsd/release/$uk"
    DEST_PATH="$BUILD_DIR/${uk}.elf"
    echo "Copying $SOURCE_PATH to $DEST_PATH"
    cp "$SOURCE_PATH" "$DEST_PATH"
done

# 4. Create Bootable Image
if [ "$ARCH" == "x86_64" ]; then
    echo ">>> Creating ISO (Limine)..."
    "$ROOT_DIR/tools/create_iso_x86_64.sh"
elif [ "$ARCH" == "aarch64" ]; then
    echo ">>> Creating Disk Image..."
    "$ROOT_DIR/tools/create_image_aarch64.sh"
fi

echo ">>> Build Success! Artifacts in $BUILD_DIR"