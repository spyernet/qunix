# Qunix Unix-like OS - Development Implementation Guide

## Overview

This document outlines the complete implementation roadmap to transform Qunix from a monolithic kernel into a fully functional Unix-like operating system with complete POSIX compatibility, comprehensive userland utilities, and a working shell environment.

## Phase 1: Core POSIX Syscalls (COMPLETED)

### Status: ✅ In Progress

Implemented syscalls:
- **Process Management**: `fork()`, `execve()`, `exit()`, `wait4()`, `getpid()`, `getppid()`
- **File Operations**: `open()`, `close()`, `read()`, `write()`, `lseek()`, `stat()`, `fstat()`
- **Directory Operations**: `mkdir()`, `rmdir()`, `unlink()`, `chdir()`, `getcwd()`
- **File Permissions**: `chmod()`, `chown()`, `umask()`
- **User/Group**: `getuid()`, `getgid()`, `setuid()`, `setgid()`
- **Signals**: `kill()` (basic), signal handlers structure in place
- **File Descriptors**: `dup()`, `dup2()`, `pipe()` (scaffolding)

**Files Modified/Created:**
- `src/kernel/sys/syscalls.rs` - Full syscall dispatch and implementations
- `src/kernel/scheduler/task.rs` - Task structure with POSIX fields
- `src/kernel/scheduler/scheduler.rs` - Process scheduling

## Phase 2: Userland C Library (PARTIALLY COMPLETED)

### Status: ✅ Basic libc created

**File:** `src/userland/libc.rs`

Implemented:
- Syscall wrappers for all x86_64 calling conventions
- String functions: `strlen()`, `strcmp()`, `memcpy()`, `memset()`
- File I/O: `read()`, `write()`, `open()`, `close()`, `dup()`, `dup2()`, `pipe()`
- Process control: `fork()`, `execve()`, `exit()`, `waitpid()`
- Directory operations: `chdir()`, `getcwd()`, `mkdir()`, `rmdir()`, `unlink()`
- POSIX error constants and flags
- Signal numbers and constants

**Next Steps:**
- Add more string functions: `strcpy()`, `strcat()`, `strchr()`, `strtok()`
- Implement `malloc()`/`free()` (memory allocation)
- Add floating-point support
- Implement `printf()` family with format parsing
- File stat structures and operations

## Phase 3: Shell Implementation (BASIC COMPLETED)

### Status: ⚠️ Basic framework created

**File:** `src/userland/shell.rs`

Implemented:
- Command parsing and tokenization
- Shell state management (CWD, exit code)
- Built-in commands: `cd`, `pwd`, `echo`, `ls`, `help`, `clear`, `exit`
- Command execution via `fork()` + external binary lookup
- Background job support (partial)

**Built-in Commands:**
```bash
cd [DIR]      - Change directory
pwd           - Print working directory  
echo [ARGS]   - Echo arguments to stdout
ls [DIR]      - List directory (stub)
help          - Show built-in command help
clear         - Clear terminal screen
exit [CODE]   - Exit shell with optional exit code
```

**Next Steps:**
- Implement pipe (`|`) and redirection (`>`, `<`, `>>`)
- Add environment variable support (`$VAR`)
- Implement job control and backgrounding (`fg`, `bg`, `jobs`)
- Add history and line editing
- Implement wildcards and glob patterns (`*.txt`)
- Add variable assignment and shell functions

## Phase 4: Core Userland Utilities (SCAFFOLDING COMPLETED)

### Status: ⚠️ Stub implementations created

**File:** `src/userland/utils.rs`

Implemented:
- **cat** - Concatenate and display files
- **echo** - Print text to stdout
- **pwd** - Print working directory
- **ls** - List directory contents (stub)
- **mkdir** - Create directories
- **rm** - Remove files
- **touch** - Create empty files
- **uname** - System information
- **id** - Print user/group IDs

**Next Priority Utilities:**
- `grep` - Text search
- `find` - File search
- `cp` - Copy files
- `mv` - Move/rename files
- `head`/`tail` - Show file start/end
- `wc` - Count lines/words/bytes
- `sort` - Sort lines
- `uniq` - Remove duplicates
- `chmod` - Change file permissions
- `chown` - Change file ownership
- `ln` - Create links
- `basename`/`dirname` - Path manipulation

## Phase 5: Filesystem Write Support

### Current Status: ⚠️ Read-only

**Required Changes:**
1. Implement `write()` syscall properly
2. Add write support to ext4 driver (inode updates, block allocation)
3. Add write support to FAT32 driver (cluster allocation, FAT updates)
4. Implement `O_CREAT` flag handling
5. Implement `O_APPEND` flag handling
6. Implement `O_TRUNC` flag handling

**Files to Modify:**
- `src/fs/ext4/` - Block allocation, inode writing
- `src/fs/fat32/` - Cluster allocation, FAT updates
- `src/fs/vfs/` - Write path in VFS layer

## Phase 6: Process Management Enhancement

### Current Status: ⚠️ Basic framework

**Required Features:**
1. **Job Control:**
   - Process groups (`pgid`)
   - Sessions (`sid`)
   - Foreground/background jobs
   - `SIGTSTP`/`SIGCONT` handling
   - `jobs`, `fg`, `bg` commands

2. **Process Monitoring:**
   - `ps` - List processes
   - `top` - Monitor system resources
   - `/proc` filesystem (optional)

3. **Signal Handling:**
   - Proper signal delivery and masking
   - Signal handlers with proper context preservation
   - `SIGCHLD` handling in parent
   - `SIGTERM`/`SIGKILL` cleanup

## Phase 7: Terminal/TTY Improvements

### Current Status: ⚠️ Basic VGA driver

**Required Enhancements:**
1. **Line Discipline:**
   - Proper echo handling
   - Backspace support
   - Tab completion (optional)
   - Line buffering

2. **Terminal Control:**
   - ANSI escape sequence support
   - Cursor positioning
   - Color support
   - Terminal size queries

3. **TTY Operations:**
   - `ioctl()` syscall implementation
   - Terminal mode control (raw vs. canonical)
   - Input/output redirection support

## Phase 8: System Initialization & Boot

### Current Status: ⚠️ Kernel-only boot

**Required:**
1. **Init System:**
   - Implement `PID 1` (init) process
   - Read `/etc/inittab` or equivalent
   - Start system services (getty on tty0)
   - Handle process reaping

2. **Boot Sequence:**
   ```
   1. Bootloader → Kernel entry
   2. HAL initialization
   3. Memory/paging setup
   4. Driver initialization
   5. Filesystem mount (root filesystem)
   6. Load and execute /sbin/init
   7. Init spawns login/shell
   8. User interaction
   ```

3. **Root Filesystem:**
   - Create initrd/initramfs with essential utilities
   - Create directory structure: `/bin`, `/sbin`, `/etc`, `/tmp`, `/home`
   - Populate with shell, utilities, config files

## Phase 9: Extended System Features

### Status: ⚠️ Planning

**Nice to Have:**
1. **Networking** (future phase)
   - Network stack
   - TCP/IP implementation
   - Socket syscalls

2. **Advanced Filesystem Features**
   - Symlinks (`symlink()`, `readlink()`)
   - Hard links (`link()`)
   - File locking (`fcntl()`)

3. **Memory Management**
   - `mmap()` and `munmap()`
   - Shared memory
   - Memory protection

4. **User Management**
   - `/etc/passwd` parsing
   - Login system
   - User switching with proper privilege dropping

5. **Package Management** (far future)
   - Simple package format
   - Package installer

## Implementation Priority Matrix

```
HIGH IMPACT, EASY:
- Fix write() syscall         ←  START HERE
- Complete shell builtins     
- Implement pipe() properly

HIGH IMPACT, HARD:
- Filesystem write support
- Job control implementation
- TTY improvements
- Init system

LOW IMPACT, EASY:
- Additional utilities (grep, find, etc.)
- More syscalls (fcntl, ioctl stubs)

LOW IMPACT, HARD:
- Networking
- Advanced VFS features
- Performance optimization
```

## Testing Strategy

1. **Unit Tests:**
   - Syscall correctness
   - String/memory functions
   - Shell parsing

2. **Integration Tests:**
   - Fork + exec sequence
   - Pipe creation and usage
   - File I/O operations

3. **System Tests in QEMU:**
   ```bash
   # Boot and run interactive tests
   cargo run --release
   
   # In QEMU:
   qunix# echo "Hello, Unix!"
   qunix# cat /etc/passwd
   qunix# ls /
   qunix# mkdir /tmp && touch /tmp/test
   ```

4. **Regression Testing:**
   - After each change, verify:
     - Kernel still boots
     - Shell still launches
     - Basic commands work

## File Organization Summary

```
src/
├── kernel/
│   ├── sys/
│   │   ├── syscalls.rs       ← Syscall dispatch (UPDATED)
│   │   └── posix/
│   │       ├── fs.rs         ← File syscalls
│   │       ├── proc.rs       ← Process syscalls
│   │       ├── signals.rs    ← Signal handling
│   │       └── posix.rs      ← POSIX compatibility layer
│   └── scheduler/
│       ├── task.rs           ← Task/PCB structure (UPDATED)
│       └── scheduler.rs      ← Scheduling logic
├── fs/
│   ├── vfs/                  ← VFS (write support needed)
│   ├── ext4/                 ← ext4 driver (write support needed)
│   └── fat32/                ← FAT32 driver (write support needed)
└── userland/
    ├── mod.rs                ← Module root (NEW)
    ├── libc.rs               ← C library (NEW - BASIC)
    ├── shell.rs              ← Shell (NEW - BASIC)
    └── utils.rs              ← Utilities (NEW - STUBS)
```

## Next Immediate Actions

1. **Fix Write Syscall** (1-2 hours)
   - Implement proper `sys_write()` for files
   - Add write path to VFS

2. **Complete Shell Loop** (2-3 hours)
   - Fix line reading from TTY
   - Implement exec lookup
   - Test fork + exec flow

3. **Basic Filesystem Write** (4-6 hours)
   - Implement file creation in ext4
   - Implement file creation in FAT32
   - Test `touch` and `echo > file`

4. **Additional Utilities** (2-3 hours)
   - Implement grep, find, cp, mv
   - Test basic operations

## References

- POSIX.1-2008 Standard
- Linux Kernel Source (mm/*, fs/*, kernel/*)
- Musl libc (minimal C library implementation)
- GNU Coreutils (standard utilities)

## Version History

- **v0.1** (December 2025): Basic kernel with syscall framework
- **v0.2** (WIP): Userland libraries and shell
- **v1.0** (Target): Fully functional Unix-like OS

