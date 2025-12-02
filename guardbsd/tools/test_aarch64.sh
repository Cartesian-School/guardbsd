#!/bin/bash
# ============================================================================
# GuardBSD: AArch64 QEMU Test Runner
# ============================================================================
set -e

# Определяем пути
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
# Для теста загружаем uk_space, так как это точка входа (_start)
KERNEL_PATH="$ROOT_DIR/build/aarch64/uk_space.elf"

# Проверка наличия ядра
if [ ! -f "$KERNEL_PATH" ]; then
    echo "Error: Kernel ELF not found at $KERNEL_PATH"
    echo "Please run: make aarch64"
    exit 1
fi

# Базовые флаги QEMU
QEMU_FLAGS=(
    -M virt                     # Виртуальная плата (Generic ARMv8)
    -m 512M                     # ОЗУ
    -cpu cortex-a57             # Эмуляция процессора ARMv8-A
    -kernel "$KERNEL_PATH"      # Прямая загрузка ELF
    -serial stdio               # Вывод логов
    -display none               # Без графического окна (только консоль)
    -no-reboot
)

# Проверка KVM (если мы запускаем это на ARM-хосте, например Raspberry Pi или Apple Silicon Linux)
if [ "$(uname -m)" = "aarch64" ] && [ -e /dev/kvm ]; then
    echo ">>> ARM Host detected: Enabling KVM."
    QEMU_FLAGS+=(-enable-kvm -cpu host)
fi

echo ">>> Starting QEMU (aarch64)..."
echo ">>> Press Ctrl+A, then X to exit."
echo "---------------------------------------------------"

# Запуск
qemu-system-aarch64 "${QEMU_FLAGS[@]}"