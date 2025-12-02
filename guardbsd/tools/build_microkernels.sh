# ============================================================================
# Build script: tools/build_microkernels.sh
# ============================================================================
#!/bin/bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARCH="${1:-x86_64}"
TARGET="${ARCH}-cartesian-gbsd"
BUILD_TYPE="${2:-release}"

echo "╔══════════════════════════════════════════════════╗"
echo "║          Building GuardBSD Microkernels          ║"
echo "║         Architecture: ${ARCH}                    ║"
echo "║         Build Type: ${BUILD_TYPE}                ║"
echo "╚══════════════════════════════════════════════════╝"
echo ""

cd "$PROJECT_ROOT"

# Build flags
if [ "$BUILD_TYPE" = "release" ]; then
    BUILD_FLAG="--release"
    BUILD_DIR="release"
else
    BUILD_FLAG=""
    BUILD_DIR="debug"
fi

# Build each microkernel
for uk in uk_space uk_time uk_ipc; do
    echo "→ Building ${uk}..."
    (cd "kernel/microkernels/${uk}" && \
     cargo build $BUILD_FLAG --target "../../../targets/${TARGET}.json")
done

echo ""
echo "✓ Build complete!"
echo ""
echo "Binary locations:"
echo "  target/${TARGET}/${BUILD_DIR}/uk_space"
echo "  target/${TARGET}/${BUILD_DIR}/uk_time"
echo "  target/${TARGET}/${BUILD_DIR}/uk_ipc"
echo ""
echo "Binary sizes:"
size "target/${TARGET}/${BUILD_DIR}/uk_"* 2>/dev/null || true

# Check TCB size
echo ""
echo "TCB Analysis:"
TOTAL_SIZE=0
for bin in "target/${TARGET}/${BUILD_DIR}/uk_"*; do
    if [ -f "$bin" ]; then
        SIZE=$(size "$bin" | tail -1 | awk '{print $1}')
        TOTAL_SIZE=$((TOTAL_SIZE + SIZE))
    fi
done

echo "  Total text section: ${TOTAL_SIZE} bytes"
if [ $TOTAL_SIZE -lt 8192 ]; then
    echo "  ✓ TCB < 8 KB target achieved!"
else
    echo "  ⚠ TCB exceeds 8 KB target"
fi
