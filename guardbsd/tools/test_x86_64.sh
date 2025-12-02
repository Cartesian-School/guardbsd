#!/bin/bash
# ============================================================================
# GuardBSD: x86_64 QEMU Test Runner
# ============================================================================
set -e

# Определяем пути
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ISO_PATH="$ROOT_DIR/build/x86_64/gbsd-x86_64.iso"

# Проверка наличия ISO
if [ ! -f "$ISO_PATH" ]; then
    echo "Error: ISO file not found at $ISO_PATH"
    echo "Please run: make x86_64"
    exit 1
fi

# Базовые флаги QEMU
QEMU_FLAGS=(
    -M q35                      # Современный чипсет (PCIe, AHCI)
    -m 512M                     # ОЗУ
    -cdrom "$ISO_PATH"          # Загрузочный диск
    -serial stdio               # Вывод логов в терминал
    -no-reboot                  # Выход при панике/перезагрузке
    -no-shutdown
)

# Проверка поддержки KVM (Аппаратное ускорение)
if [ -e /dev/kvm ] && [ -w /dev/kvm ]; then
    echo ">>> KVM detected: Enabling hardware acceleration."
    QEMU_FLAGS+=(-enable-kvm -cpu host)
else
    echo ">>> KVM not available: Using software emulation (TCG)."
    QEMU_FLAGS+=(-cpu max)
fi

echo ">>> Starting QEMU (x86_64)..."
echo ">>> Press Ctrl+A, then X to exit."
echo "---------------------------------------------------"

# Запуск
qemu-system-x86_64 "${QEMU_FLAGS[@]}"