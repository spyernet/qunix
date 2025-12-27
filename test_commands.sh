#!/bin/bash

# Test script to verify Qunix OS shell commands work correctly

BOOTIMAGE="target/x86_64-qunix/debug/bootimage-qunix.bin"

echo "Testing Qunix OS Shell Commands"
echo "======================================"
echo ""
echo "Building test input..."

# Create test commands
cat > /tmp/qunix_test_input.txt << 'EOF'
help
whoami
uname
id
pwd
echo Hello World
ls
ps
clear
whoami
EOF

echo "Running QEMU with test input..."
echo ""

# Run QEMU with test input and capture output
timeout 8 qemu-system-x86_64 \
  -drive format=raw,file="$BOOTIMAGE" \
  -serial stdio \
  -nographic \
  -monitor none \
  < /tmp/qunix_test_input.txt 2>&1 | tee /tmp/qemu_output.txt

echo ""
echo "======================================"
echo "Test Summary"
echo "======================================"

# Check for command outputs
OUTPUT="/tmp/qemu_output.txt"

if grep -q "help" "$OUTPUT" && grep -q "whoami" "$OUTPUT"; then
    echo "✓ Commands are being parsed"
fi

if grep -q "root@qunix" "$OUTPUT"; then
    echo "✓ Shell prompt appears"
fi

if grep -q "Qunix Shell" "$OUTPUT"; then
    echo "✓ help command works"
fi

if grep -q "uid=0" "$OUTPUT"; then
    echo "✓ id command works"
fi

if grep -q "Hello World" "$OUTPUT"; then
    echo "✓ echo command works"
fi

# Count successful outputs
SUCCESS_COUNT=$(grep -c "✓" << 'SUMMARY'
Qunix Shell
root
uid=0
Hello World
SUMMARY
)

echo ""
echo "Commands working: $SUCCESS_COUNT/5"
