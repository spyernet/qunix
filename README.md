# Qunix OS

A production-grade, POSIX-compatible operating system kernel written entirely in **Rust**.

Qunix OS is a bare-metal, monolithic kernel focused on **security**, **correctness**, and **POSIX compliance**, while maintaining a clean and modular architecture.

## Highlights

- **POSIX Compatibility** — Designed to support Linux/Unix-style applications  
- **Qunix Security Framework (QSF)** — Mandatory access control, capabilities, and integrity enforcement  
- **Virtual File System** — ext4 and FAT32 support via a unified VFS layer  
- **Preemptive Multitasking** — Priority-based scheduler with context switching  
- **Hardware Abstraction Layer** — Modular x86_64 drivers (CPU, memory, storage, I/O)


## Status

Qunix is under active development and currently boots in QEMU, with core kernel services, memory management, filesystems (read support), and security scaffolding in place.

## License

MIT
