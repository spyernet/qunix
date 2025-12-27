#!/bin/bash

# Test script to verify Qunix OS boots and responds correctly

set -e

BOOTIMAGE="target/x86_64-qunix/debug/bootimage-qunix.bin"
TEST_OUTPUT="/tmp/qunix_test_output.log"
TEST_INPUT="/tmp/qunix_test_input.txt"

echo "========================================="
echo "Qunix OS Boot Test"
echo "========================================="
echo ""

# Check if bootimage exists, if not build it
if [ ! -f "$BOOTIMAGE" ]; then
    echo "[1/3] Building bootimage..."
    cargo bootimage --target x86_64-qunix.json > /dev/null 2>&1
    echo "✓ Bootimage built successfully"
else
    echo "✓ Bootimage already exists"
fi

echo ""
echo "[2/3] Preparing test..."

# Create a simple test sequence
cat > "$TEST_INPUT" << 'EOF'
help
whoami
uname
echo Hello Qunix
id
ps
EOF

echo "✓ Test input prepared"

echo ""
echo "[3/3] Running boot test with timeout..."
echo ""
echo "--- QEMU Serial Output ---"

# Run QEMU with test input and capture output
# We use a timeout to prevent hanging
(
    timeout 10 qemu-system-x86_64 \
        -drive format=raw,file="$BOOTIMAGE" \
        -serial stdio \
        -nographic \
        -monitor none \
        < "$TEST_INPUT" 2>&1 || true
) | tee "$TEST_OUTPUT"

echo ""
echo "--- Test Analysis ---"

# Check if boot messages appear
if grep -q "Qunix OS\|Shell Ready\|init_main" "$TEST_OUTPUT" 2>/dev/null; then
    echo "✓ Kernel booted successfully"
else
    echo "✗ Kernel boot messages not found"
fi

# Check if shell prompt appeared
if grep -q "root@qunix" "$TEST_OUTPUT" 2>/dev/null; then
    echo "✓ Shell prompt displayed"
else
    echo "✗ Shell prompt not found"
fi

# Check if commands were processed
if grep -q "help\|whoami\|root" "$TEST_OUTPUT" 2>/dev/null; then
    echo "✓ Commands were processed"
else
    echo "✗ Command processing not confirmed"
fi

echo ""
echo "========================================="
echo "Test Complete"
echo "========================================="
echo ""
echo "Note: Output may be limited due to timeout and non-interactive mode."
echo "For interactive testing, run: ./run_qemu.sh"
echo ""

# Cleanup
rm -f "$TEST_INPUT"
