# Qunix OS - Boot and Run Guide

## Building the OS

First, build the bootable image:

```bash
cargo bootimage --target x86_64-qunix.json
```

This creates the bootimage at: `target/x86_64-qunix/debug/bootimage-qunix.bin`

## Running with QEMU

### Interactive Mode (Recommended)

Use the provided script for interactive shell access:

```bash
./run_qemu.sh
```

Or run QEMU directly with serial I/O enabled:

```bash
qemu-system-x86_64 \
  -drive format=raw,file=target/x86_64-qunix/debug/bootimage-qunix.bin \
  -serial stdio \
  -nographic \
  -monitor none
```

**Important**: The `-serial stdio` flag is required when using `-nographic` to see the boot messages and interact with the shell. Without it, QEMU will not output to your terminal.

### Available Shell Commands

Once the OS boots, you'll see the prompt:
```
root@qunix:/# 
```

Type any of these commands:

**System Info:**
- `help` - Show available commands
- `whoami` - Print current user (root)
- `uname` - Print system information
- `id` - Print user ID information
- `clear` - Clear the screen
- `ps` - List running processes

**File Operations:**
- `echo <text>` - Print text
- `pwd` - Print working directory  
- `cd <dir>` - Change directory (parsing only)
- `ls [dir]` - List directory (stub)
- `cat <file>` - Display file (stub)
- `touch <file>` - Create file (stub)
- `mkdir <dir>` - Create directory (stub)
- `rm <file>` - Remove file (stub)
- `chmod <mode> <file>` - Change permissions (stub)

**System:**
- `fork` - Test fork syscall
- `exit` - Exit shell (disabled in init mode)

### Boot Output

The kernel prints boot information to both VGA (screen) and serial output:

1. **Bootloader**: Initializes basic hardware
2. **HAL Init**: Sets up GDT, IDT, interrupts, memory, drivers
3. **Kernel Init**: Initializes scheduler, VFS, security framework
4. **Init Process**: Starts the init shell (PID 1)

All of this is logged to serial for debugging.

## Troubleshooting

### "OS doesn't respond"

If you run:
```bash
qemu-system-x86_64 \
  -drive format=raw,file=target/x86_64-qunix/debug/bootimage-qunix.bin \
  -nographic
```

Without `-serial stdio`, QEMU won't output anything because:
- `-nographic` disables the VGA display
- Without `-serial stdio`, serial output goes nowhere visible
- You won't see boot messages or the shell prompt

**Solution**: Always add `-serial stdio` when using `-nographic`.

### Other Common Issues

1. **"Command not found" for bootimage**
   ```bash
   cargo install bootimage
   ```

2. **Build errors with warnings**
   - These are expected; the build generates 50+ compiler warnings but still compiles successfully

3. **QEMU won't start**
   - Check that QEMU is installed: `which qemu-system-x86_64`
   - Verify the bootimage exists: `ls -la target/x86_64-qunix/debug/bootimage-qunix.bin`

## What's Happening Behind the Scenes

1. **Bootloader**: The bootloader (from the `bootloader` crate) loads the kernel into memory and switches to 64-bit mode
2. **Kernel Entry**: `_start` (from bootloader) calls `kernel_main` with boot information
3. **HAL Initialization**: Hardware Abstraction Layer sets up CPU, memory, and drivers
4. **Kernel Initialization**: Scheduler, filesystem, and security framework are initialized
5. **Init Process**: The first user process starts an interactive shell
6. **Shell Loop**: Shell reads commands from serial port and executes them

## Serial I/O Details

The Qunix OS uses serial port I/O for console interaction in headless/nographic mode:
- **COM1 (0x3F8)**: Primary serial port - connected to QEMU's stdio
- **VGA Display**: Also written to for compatibility (though not visible in nographic mode)
- **Keyboard**: Can be used when running with graphics (not in nographic mode)

The shell specifically reads input from the serial port at COM1, making it compatible with QEMU's `-nographic` mode.
