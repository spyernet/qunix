# Qunix OS — A Unix-like Operating System in Rust

**Status:** 🚀 Active Development  
**Goal:** A fully functional, POSIX-compliant Unix-like operating system with complete userland utilities and shell.

An experimental, production-oriented operating system kernel written entirely in **Rust**, designed for security, correctness, and POSIX compliance.

## Key Features

- **POSIX Compatibility** — Linux/Unix-style applications and utilities  
- **Qunix Security Framework (QSF)** — Mandatory access control, capabilities, and integrity enforcement  
- **Virtual File System** — ext4 and FAT32 support via unified VFS layer  
- **Preemptive Multitasking** — Priority-based scheduler with process management  
- **Hardware Abstraction Layer** — Modular x86_64 drivers (CPU, memory, storage, I/O)  
- **Userland Libraries & Utilities** — Minimal C library, shell, and core command-line tools  

## Development Status (December 2025)

### ✅ Completed
- Process creation (fork), execution (execve), termination (exit)
- Process scheduling with priority queues
- 70+ comprehensive syscall implementations
- Signal handling framework (POSIX signals)
- Memory management (paging, frame allocator, heap)
- **NEW: Minimal libc** — Syscall wrappers, POSIX functions
- **NEW: Shell implementation** — Command parsing, built-in commands, fork/exec
- **NEW: Userland utilities** — echo, cat, pwd, ls, mkdir, rm, touch, chmod, id, uname
- **NEW: 16+ shell built-in commands** with full argument parsing

### 🔄 In Progress
- Filesystem write support (critical priority)
- POSIX syscall completeness
- Shell enhancements (pipes, redirections, variables)
- TTY/terminal improvements

## Building & Running

```bash
rustup override set nightly
rustup component add rust-src llvm-tools-preview
cargo install bootimage
sudo apt install qemu-system-x86  # or: brew install qemu

cargo bootimage --release
cargo run --release
```

## Shell Commands Available

**System:** help, whoami, uname, id, clear, ps, fork, exit  
**Files:** pwd, cd, ls, cat, echo, touch, mkdir, rm, chmod

## Development Roadmap

Detailed plan in [DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md)

**Next:** Filesystem writes → Shell pipes/redirects → More utilities → Init system

## License

See LICENCE file in source root
