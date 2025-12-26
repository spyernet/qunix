# Qunix OS - Quick Start Guide

## Building and Running

### Setup (one-time)

```bash
# 1. Set Rust to nightly
rustup override set nightly

# 2. Add required components
rustup component add rust-src llvm-tools-preview

# 3. Install bootimage tool
cargo install bootimage

# 4. Install QEMU emulator
# Ubuntu/Debian:
sudo apt update && sudo apt install qemu-system-x86

# macOS:
brew install qemu

# Windows: Download from https://www.qemu.org/download/
```

### Building the Kernel

```bash
cd /workspaces/qunix

# Build the kernel image with bootloader
cargo bootimage --release

# This creates:
# target/x86_64-qunix/release/bootimage-qunix.bin
```

### Running in QEMU

```bash
# Method 1: Using cargo run (automatic)
cargo run --release

# Method 2: Manual QEMU invocation
qemu-system-x86_64 \
  -drive format=raw,file=target/x86_64-qunix/release/bootimage-qunix.bin \
  -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
  -display none \
  -serial stdio
```

## Using the Shell

Once Qunix boots, you'll see:

```
╔══════════════════════════════════╗
║     Qunix OS - Shell Ready       ║
║   Type 'help' for commands       ║
╚══════════════════════════════════╝

root@qunix:/# _
```

### Available Commands

#### System Information
```bash
root@qunix:/# help                  # Show command help
root@qunix:/# whoami                # Print current user (root)
root@qunix:/# id                    # Print UID/GID info
root@qunix:/# uname                 # Print system info (Qunix 1.0 x86_64)
root@qunix:/# pwd                   # Print working directory
root@qunix:/# ps                    # List running processes
```

#### File Operations
```bash
root@qunix:/# echo "Hello, Qunix!"  # Print text
root@qunix:/# cat file.txt          # Display file contents
root@qunix:/# ls /                  # List directory
root@qunix:/# mkdir /newdir         # Create directory
root@qunix:/# touch file.txt        # Create empty file
root@qunix:/# rm file.txt           # Remove file
root@qunix:/# chmod 644 file.txt    # Change permissions
root@qunix:/# cd /                  # Change directory
```

#### System Testing
```bash
root@qunix:/# clear                 # Clear screen
root@qunix:/# fork                  # Test fork syscall
```

### Command Examples

```bash
# Print a greeting
root@qunix:/# echo "Welcome to Qunix OS!"
Welcome to Qunix OS!

# Check current user
root@qunix:/# whoami
root

# Get user information
root@qunix:/# id
uid=0(root) gid=0(root) groups=0(root)

# Check system type
root@qunix:/# uname
Qunix 1.0 x86_64

# Test process creation
root@qunix:/# fork
[PARENT] Forked child process: <PID>

# Show help
root@qunix:/# help
Qunix Shell - Available Commands:

System Info:
  help      - Show this help message
  whoami    - Print current user
  uname     - Print system information
  id        - Print user ID information
  ...
```

## Exit QEMU

Press **Ctrl+C** to stop the emulation and exit QEMU.

## Troubleshooting

### "command not found" for `cargo`

Make sure Rust is properly installed:
```bash
rustup --version
cargo --version
```

### QEMU window doesn't appear

The kernel outputs to serial port. Output appears in the terminal where you ran `cargo run`.

### Build fails with "cannot find -lc"

This is expected. Qunix is bare-metal and doesn't link against system libc. This is handled by the custom `x86_64-qunix.json` target specification.

### Kernel panics on boot

Check the error message in console output. Common causes:
- Memory initialization issues
- Interrupt setup problems
- Missing required features

See `DEVELOPMENT_GUIDE.md` for troubleshooting details.

## What's Implemented

### Working Features ✅
- Shell with 16+ built-in commands
- Process creation (fork syscall)
- File operations (basic)
- System information queries
- TTY input/output

### Partially Implemented ⚠️
- File read operations
- File system browsing

### Not Yet Implemented ❌
- File write operations
- Pipes and redirections (|, >, <)
- Environment variables
- External command execution (exec)
- Job control (fg, bg, jobs)

## Next Steps in Development

1. **Filesystem Write Support**
   - Will enable file creation and modification
   - Required for full file I/O operations

2. **Shell Pipes and Redirections**
   - Enables powerful command chaining
   - Critical for Unix-like workflow

3. **Additional Utilities**
   - grep, find, cp, mv, etc.
   - Expands functionality significantly

For detailed development roadmap, see [DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md).

## Architecture Details

### Boot Process

```
1. Bootloader (bootimage) → x86_64 entry point
2. Early kernel setup (paging, GDT, IDT)
3. HAL initialization (hardware drivers)
4. Kernel subsystem initialization (VFS, scheduler)
5. Create init process (PID 1)
6. Init process launches shell
7. Shell loop (read → parse → execute → prompt)
```

### Memory Layout

- **Kernel space:** 0xFFFFFFFF80000000 (higher-half kernel)
- **User space:** 0x0 - 0x7FFFFFFF
- **Kernel heap:** 0x4444_4444_0000 (16 MB)

### Process Model

Each process has:
- PID (process ID)
- PPID (parent process ID)
- UID/GID (user and group IDs)
- Current working directory
- File descriptor table
- Signal handlers
- Memory context (page tables)
- Execution context (CPU registers)

## Advanced Usage

### Testing Fork

```bash
root@qunix:/# fork
[PARENT] Forked child process: 2
```

This tests the fork syscall by creating a child process.

### Checking Processes

```bash
root@qunix:/# ps
 PID  NAME
  1   init
  2   (fork test child)
```

### File Operations (when write support is added)

```bash
# Create file
root@qunix:/# touch /tmp/test.txt

# Write to file
root@qunix:/# echo "Hello" > /tmp/test.txt

# Read file
root@qunix:/# cat /tmp/test.txt
Hello

# List files
root@qunix:/# ls /tmp/
test.txt
```

## Support and Documentation

- **General OS Info:** [README.md](README.md)
- **Development Roadmap:** [DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md)
- **Implementation Details:** [IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md)
- **POSIX Standards:** https://pubs.opengroup.org/onlinepubs/9699919799/
- **Linux Kernel Docs:** https://www.kernel.org/doc/html/latest/

## License

MIT License - See [LICENSE](LICENSE) file for details.

---

**Last Updated:** December 26, 2025  
**Qunix Version:** 0.2.0

