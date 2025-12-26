# Qunix Operating System
### Secure. POSIX-Compliant. Rust-Built.

![Build Status](https://img.shields.io/badge/build-passing-brightgreen) ![Architecture](https://img.shields.io/badge/arch-x86__64-blue) ![License](https://img.shields.io/badge/license-MIT-orange)

**Qunix** is a next-generation monolithic kernel written in Rust, designed to bridge the gap between memory-safe modern development and strict POSIX compliance. It features a unique security subsystem (QSF), a hybrid Virtual File System (VFS), and a fully preemptive multitasking scheduler, capable of running standard Unix tools like `zsh`, `nano`, and `ls` right out of the box.

---

## 📂 Project Structure

The Qunix architecture is modular, separating hardware abstractions from core kernel logic and security policies.

```text
Qunix/
├── Cargo.toml                          # Project manifest, dependencies (bootloader 0.9, alloc, etc.)
├── rust-toolchain.toml                 # Pins Nightly toolchain for kernel builds
├── README.md                           # Documentation for building and running Qunix
│
├── qsf/                                # SSF: System Security Foundation (core security framework)
│   ├── policies/                       # High-level security policies for sandboxing, capabilities, etc.
│   │   ├── default.qpol                # Default global QSF security rules
│   │   ├── network.qpol                # Network access controls & restrictions
│   │   └── sandbox.qpol                # Per-process sandboxing profiles
│   ├── modules/                        # Internal QSF components (Rust implementations)
│   │   ├── integrity.rs                # File integrity checking, hash validation, tamper detection
│   │   ├── capability.rs               # POSIX-like capabilities system (fine-grained privileges)
│   │   └── confinement.rs              # Process isolation, mandatory access controls
│   └── qsf.rs                          # Main QSF orchestrator (parses policies + applies rules)
│
├── fs/                                 # Filesystem subsystem (VFS + ext4 + FAT32)
│   ├── ext4/                           # Complete ext4 implementation
│   │   ├── block.rs                    # Block group structures, superblock parsing
│   │   ├── inode.rs                    # Inode operations (read/write/stat)
│   │   └── ext4.rs                     # Main ext4 filesystem driver (mount, read, write)
│   ├── fat32/                          # FAT32 implementation for legacy devices
│   │   ├── fat.rs                      # FAT table parsing and traversal logic
│   │   ├── dir.rs                      # Directory entry parsing, LFN, short names
│   │   └── fat32.rs                    # Main FAT32 driver (mount, read, write)
│   ├── vfs/                            # Virtual filesystem (unifies ext4 + FAT32 + devfs)
│   │   ├── node.rs                     # VFS nodes (files, dirs, special devices)
│   │   ├── api.rs                      # VFS API: open, read, write, mountpoint mgmt
│   │   └── vfs.rs                      # Core VFS manager and dispatch layer
│   └── mount.rs                        # Mount table, POSIX mount/unmount logic
│
├── hal/                                # Hardware Abstraction Layer
│   ├── cpu/                            # CPU architecture code (x86_64)
│   │   ├── gdt.rs                      # Global Descriptor Table setup
│   │   ├── idt.rs                      # Interrupt Descriptor Table setup
│   │   └── interrupts.rs               # Interrupt handlers and dispatch
│   ├── memory/                         # Memory management subsystem
│   │   ├── paging.rs                   # Page tables, higher-half mapping
│   │   ├── heap.rs                     # Kernel heap allocator
│   │   └── mmu.rs                      # Low-level MMU operations
│   ├── drivers/                        # Hardware drivers (PCI, USB, storage, TTY, etc.)
│   │   ├── pci.rs                      # PCI bus scanning + device mapping
│   │   ├── ahci.rs                     # SATA AHCI driver for SSDs/HDDs
│   │   ├── usb.rs                      # USB stack (UHCI/EHCI/XHCI)
│   │   └── tty.rs                      # TTY/console driver
│   └── hal.rs                          # HAL entry, unifying CPU/memory/drivers
│
├── kernel/                             # Core kernel logic
│   ├── scheduler/                      # Multitasking subsystem
│   │   ├── task.rs                     # Task struct, context metadata
│   │   ├── context.rs                  # Context switching (register save/restore)
│   │   └── scheduler.rs                # Priority scheduler, run queue management
│   ├── sys/                            # System call layer
│   │   ├── posix/                      # POSIX compatibility layer
│   │   │   ├── fs.rs                   # POSIX filesystem syscalls (open, read, write, stat...)
│   │   │   ├── proc.rs                 # Processes, fork/exec, /proc backend
│   │   │   ├── signals.rs              # Full POSIX signal implementation
│   │   │   └── posix.rs                # Unified POSIX API interface
│   │   └── syscalls.rs                 # Kernel syscall dispatcher
│   ├── init.rs                         # Early kernel init (memory, drivers, filesystem, QSF)
│   └── kernel.rs                       # High-level kernel state and orchestration
│
├── src/                                # Boot entry + embedded userland binaries
│   ├── main.rs                         # Kernel entrypoint (called by bootloader 0.9)
│   ├── lib.rs                          # Shared kernel utilities
│   ├── sbin/                           # Core system binaries (statically shipped)
│   │   ├── init                        # First user process (PID 1)
│   │   ├── service-manager             # Manages system services/daemons
│   │   └── mountd                      # Automount daemon for ext4/FAT32 devices
│   └── bin/                            # Userland binaries (prebuilt, static)
│       ├── zsh                         # Default shell
│       ├── x11                         # X11 server binary (placeholder)
│       ├── ls                          # File listing utility
│       ├── cat                         # Display file content
│       ├── cp                          # Copy files
│       ├── ifconfig                    # Network interface listing/config
│       ├── ping                        # ICMP tool
│       ├── netd                        # Network daemon
│       └── nano                        # Terminal text editor
│
├── .cargo/
│   └── config.toml                     # Configures bootimage runner + target JSON
```

---

## 🛡️ QSF: The System Security Foundation

Qunix is not just a kernel; it is a secure computing environment. The **QSF** (located in `qsf/`) is a mandatory access control engine deeply integrated into the kernel lifecycle.

*   **Policy Driven:** Security is defined in human-readable `.qpol` files.
    *   `sandbox.qpol`: Defines rigid boundaries for untrusted processes.
    *   `network.qpol`: Controls packet flow and socket binding permissions.
*   **Integrity Enforcement:** The `integrity.rs` module ensures that no binary runs unless its hash matches the kernel's trust store, preventing runtime injection attacks.
*   **Capability Model:** Unlike standard Unix "root vs user," QSF utilizes `capability.rs` to grant fine-grained permissions (e.g., `CAP_NET_BIND_SERVICE` or `CAP_SYS_TIME`) to specific binaries.

## 💿 Filesystem Architecture

Qunix implements a robust Virtual File System (VFS) that abstracts physical storage differences from the user.

*   **VFS Layer (`fs/vfs/`):** Provides the standard `open`, `read`, `write`, and `stat` interfaces. It handles mount points via `mount.rs`, allowing seamless mounting of different devices under a single root `/`.
*   **Ext4 (`fs/ext4/`):** A fully compliant implementation of the Fourth Extended Filesystem, featuring block groups, inode management, and journaling support for high-performance storage.
*   **FAT32 (`fs/fat32/`):** Legacy support for UEFI boot partitions and USB flash drives, including Long File Name (LFN) support in `dir.rs`.

## 💻 Hardware Abstraction & Drivers

The **HAL** (`hal/`) is designed for the x86_64 architecture but structured for future portability.

*   **Memory Management:** Uses higher-half kernel mapping. `paging.rs` manages 4-level page tables, while `heap.rs` implements a custom allocator for kernel dynamic memory.
*   **Device Drivers:**
    *   **AHCI:** Native SATA driver for high-speed disk access.
    *   **USB:** Complete stack supporting UHCI/EHCI/XHCI controllers.
    *   **PCI:** Automatic bus scanning to detect and initialize peripherals.

## ⚙️ Kernel & POSIX Compatibility

Qunix aims to run Linux-compatible binaries through a strict POSIX layer.

*   **Scheduler (`kernel/scheduler/`):** A preemptive, priority-based round-robin scheduler. It manages thread contexts (`context.rs`) and supports efficient switching.
*   **POSIX Layer (`kernel/sys/posix/`):**
    *   `signals.rs`: Implements standard Unix signals (SIGINT, SIGKILL, SIGSEGV).
    *   `proc.rs`: Handles the complex logic of `fork()` (process cloning) and `exec()` (binary loading).
*   **Userland:** The OS ships with statically linked core tools including `zsh` (shell), `nano` (editor), and networking tools (`ping`, `ifconfig`), ensuring a usable environment immediately after boot.

---

## 🚀 Getting Started

### Prerequisites
*   Rust Nightly Toolchain (`rustup override set nightly`)
*   QEMU System x86_64
*   `bootimage` crate (`cargo install bootimage`)

### Build and Run
To compile the kernel and boot it in QEMU:

```bash
# Build the kernel and userland binaries
cargo bootimage

# Create a bootable disk image and launch QEMU
cargo run
```

### The Boot Process
1.  **Bootloader:** The `bootloader` crate loads the kernel into memory and jumps to `src/main.rs`.
2.  **HAL Init:** The kernel initializes the GDT, IDT, Paging, and Heap.
3.  **Driver Init:** PCI bus is scanned; AHCI and USB drivers are loaded.
4.  **VFS Mount:** The root filesystem (ext4) is mounted.
5.  **QSF Load:** Security policies (`.qpol`) are parsed and enforced.
6.  **Userland:** The kernel spawns `/sbin/init` (PID 1), which launches the `service-manager` and finally drops the user into `zsh`.

---

## 🔮 Roadmap

*   [ ] **GUI Implementation:** Activate the placeholder `x11` server and implement a frame-buffer compositor.
*   [ ] **Dynamic Linking:** Add support for `.so` shared libraries in the loader.
*   [ ] **Symmetric Multiprocessing (SMP):** Enable support for multi-core CPUs.
*   [ ] **Networking:** Expand `netd` to support full TCP/IP stack.

---

*Copyright © 2025 Qunix Project. Distributed under the MIT License.*
