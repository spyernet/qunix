#!/bin/bash

# Qunix OS Interactive Boot Script
# This script boots the OS with proper serial I/O configuration for -nographic mode

BOOTIMAGE="target/x86_64-qunix/debug/bootimage-qunix.bin"

if [ ! -f "$BOOTIMAGE" ]; then
    echo "Error: Bootimage not found. Building..."
    cargo bootimage --target x86_64-qunix.json || exit 1
fi

echo "=================================="
echo "Qunix OS - Interactive Shell"
echo "=================================="
echo ""
echo "Booting Qunix OS..."
echo "Type 'help' for available commands"
echo ""

# Run QEMU with serial connected to stdio and monitor disabled
qemu-system-x86_64 \
  -drive format=raw,file=$BOOTIMAGE \
  -serial stdio \
  -nographic \
  -monitor none \
  -m 256
