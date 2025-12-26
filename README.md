# Qunix OS

A production-grade, POSIX-compatible operating system kernel written entirely in Rust.

## Overview

Qunix OS is a bare-metal monolithic kernel designed with security and POSIX compliance in mind. It provides:

- **Full POSIX Compatibility**: Compatible with Linux, Unix, and POSIX applications
- **Qunix Security Framework (QSF)**: Mandatory access control with integrity verification, capabilities, and process confinement
- **Hybrid Virtual File System**: Support for ext4 and FAT32 filesystems
- **Preemptive Multitasking**: Priority-based scheduler with context switching
- **Hardware Abstraction Layer**: Modular drivers for x86_64 architecture

## Project Structure

```
qunix/
├── src/
│   ├── main.rs          # Kernel entry point
│   └── lib.rs           # Library root
├── hal/                  # Hardware Abstraction Layer
│   ├── cpu/             # GDT, IDT, interrupts
│   ├── memory/          # Paging, heap, MMU
│   └── drivers/         # VGA, serial, PCI, AHCI, USB
````markdown
# Qunix OS

An experimental, POSIX-oriented monolithic kernel written in Rust. Qunix focuses on strong security primitives (the Qunix Security Framework), POSIX compatibility, and modular hardware and filesystem abstractions.

## Current Development Status (December 2025)

- Kernel core, scheduler, and basic syscall layer: implemented.
- VFS layer with ext4 and FAT32 drivers: basic read-oriented implementations present.
- HAL drivers: VGA, serial, PIT, PCI enumeration, AHCI, and basic USB foundations implemented.
- Memory management: paging, frame allocator, and heap present.
- QSF (Qunix Security Framework): modules for integrity, capabilities, and confinement scaffolded and integrated with POSIX layer.
- Testing: QEMU-based runs supported; ongoing work to expand filesystem write support and complete userland compatibility.

## Repository Tree (current)

```
Cargo.toml
LICENSE
README.md
rust-toolchain.toml
x86_64-qunix.json
documentation/
	Qunix.md
src/
	lib.rs
	main.rs
	fs/
		mod.rs
		mount.rs
		ext4/
			block.rs
			ext4.rs
			inode.rs
			mod.rs
		fat32/
			dir.rs
			fat.rs
			fat32.rs
			mod.rs
		vfs/
			api.rs
			mod.rs
			node.rs
			vfs.rs
	hal/
		hal.rs
		mod.rs
		cpu/
			gdt.rs
			idt.rs
			interrupts.rs
			mod.rs
		drivers/
			ahci.rs
			keyboard.rs
			mod.rs
			pci.rs
			pit.rs
			serial.rs
			tty.rs
			usb.rs
			vga.rs
		memory/
			frame_allocator.rs
			heap.rs
			mmu.rs
			mod.rs
			paging.rs
	kernel/
		init.rs
		kernel.rs
		mod.rs
		scheduler/
			context.rs
			mod.rs
			scheduler.rs
			task.rs
		sys/
			mod.rs
			syscalls.rs
			posix/
				fs.rs
				mod.rs
				posix.rs
				proc.rs
				signals.rs
	qsf/
		mod.rs
		qsf.rs
		modules/
			capability.rs
			confinement.rs
			integrity.rs
			mod.rs
		policies/
			mod.rs

```

## File and Folder Descriptions

- `Cargo.toml`: Rust workspace/crate manifest and dependencies.
- `LICENSE`: Project license (MIT).
- `README.md`: This file.
- `rust-toolchain.toml`: Pinning Rust toolchain (nightly configuration).
- `x86_64-qunix.json`: Custom target spec for building the kernel.
- `documentation/Qunix.md`: High-level design notes, architecture diagrams, and contributor guidance.

### `src/`
- `lib.rs`: Library crate root; common utilities and re-exports used by kernel and tests.
- `main.rs`: Kernel entry point (boot setup and handoff to `kernel::init`).

### `src/fs/` (Filesystem subsystem)
- `mod.rs`: Filesystem subsystem glue (exports and initialization).
- `mount.rs`: Mount table and mount/unmount logic.

#### `src/fs/ext4/`
- `block.rs`: Block-level I/O helpers and block cache abstractions for ext4.
- `ext4.rs`: ext4 filesystem implementation (metadata parsing, superblock handling).
- `inode.rs`: ext4 inode operations and translation to VFS inode representation.
- `mod.rs`: Module exports and shared types for ext4 implementation.

#### `src/fs/fat32/`
- `dir.rs`: Directory parsing and FAT32 directory entry helpers.
- `fat.rs`: Core FAT table handling and cluster chain utilities.
- `fat32.rs`: FAT32-specific filesystem implementation (mount, lookup, read).
- `mod.rs`: FAT32 module exports.

#### `src/fs/vfs/`
- `api.rs`: Public VFS API used by syscalls and kernel subsystems.
- `mod.rs`: VFS module integration and shared types.
- `node.rs`: VFS node abstraction (files/directories/inodes) and reference management.
- `vfs.rs`: VFS core (lookup, open, read, write scaffolding and mounts).

### `src/hal/` (Hardware Abstraction Layer)
- `hal.rs`: HAL initialization and top-level API.
- `mod.rs`: HAL module exports.

#### `src/hal/cpu/`
- `gdt.rs`: Global Descriptor Table setup for x86_64 higher-half kernel.
- `idt.rs`: Interrupt Descriptor Table setup and handlers registration.
- `interrupts.rs`: IRQ/interrupt handling helpers and dispatching.
- `mod.rs`: CPU module exports.

#### `src/hal/drivers/`
- `ahci.rs`: AHCI SATA driver basics and block device registration.
- `keyboard.rs`: PS/2 keyboard driver and scancode processing.
- `mod.rs`: Driver registry and common utilities.
- `pci.rs`: PCI bus enumeration and device probing helpers.
- `pit.rs`: Programmable Interval Timer driver and timer abstractions.
- `serial.rs`: UART (serial) driver for debugging/console output.
- `tty.rs`: TTY layer used by simple console and potential userland terminals.
- `usb.rs`: USB stack foundations (enumeration hooks and device support).
- `vga.rs`: VGA text-mode driver for early kernel output.

#### `src/hal/memory/`
- `frame_allocator.rs`: Physical frame allocation (free list / buddy or bitmap allocator).
- `heap.rs`: Kernel heap allocator and allocator hooks for Rust global allocator.
- `mmu.rs`: MMU helpers and abstractions around page table operations.
- `mod.rs`: Memory module exports.
- `paging.rs`: Paging setup and higher-half kernel mapping helpers.

### `src/kernel/`
- `init.rs`: Kernel initialization sequence and device/service bring-up.
- `kernel.rs`: Kernel-level utilities, main runtime loops, and panic hooks.
- `mod.rs`: Kernel module exports.

#### `src/kernel/scheduler/`
- `context.rs`: Task context layout (registers saved/restored during switches).
- `mod.rs`: Scheduler exports.
- `scheduler.rs`: Main scheduler implementation (runqueue, priorities, preemption).
- `task.rs`: Task structure, creation, and lifecycle management.

#### `src/kernel/sys/`
- `mod.rs`: Syscall subsystem exports and initialization.
- `syscalls.rs`: Syscall table and syscall dispatch implementation.

##### `src/kernel/sys/posix/`
- `fs.rs`: POSIX-facing filesystem syscall implementations (open/read/write/stat).
- `mod.rs`: POSIX sys module exports.
- `posix.rs`: POSIX compatibility shims and helpers.
- `proc.rs`: Process-related syscalls and process group management.
- `signals.rs`: POSIX signal delivery and handling.

### `src/qsf/` (Qunix Security Framework)
- `mod.rs`: QSF exports and initialization.
- `qsf.rs`: Policy engine core and enforcement hooks.

#### `src/qsf/modules/`
- `capability.rs`: Capability model implementation and checks.
- `confinement.rs`: Process confinement (sandboxing) helpers.
- `integrity.rs`: File/process integrity verification (hash checks, attestation).
- `mod.rs`: Export module for QSF components.

#### `src/qsf/policies/`
- `mod.rs`: Policy definitions and loader.

## Building and Running (quick)

Prerequisites: Rust nightly, `bootimage`, `qemu-system-x86` (or equivalent).

Build and run in QEMU:

```bash
rustup override set nightly
rustup component add rust-src llvm-tools-preview
cargo install bootimage
cargo bootimage --release
cargo run --release
```

## Contributing

Any contribution is appreciated: opening issues, adding tests, expanding filesystem write support, or improving QSF policies. Follow Rust formatting, add documentation, and keep changes focused.

## License

MIT License

````
