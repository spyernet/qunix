# Shell Refactoring and Command Output Fix

## Issues Fixed

### 1. Shell Freeze Issue
**Problem**: After user typed commands, the shell would not return to the prompt and command output was not visible.

**Root Cause**: 
- The prompt was using `print!()` which writes to VGA only
- In nographic mode (-nographic), VGA output is not visible
- Command output was also going to VGA (println!) which wasn't visible

**Solution**: 
- Updated shell_loop() to explicitly print to both serial and VGA
- Ensured serial output is properly flushed before reading input

### 2. Command Output Not Displaying
**Problem**: Commands like `whoami`, `help`, `echo` were being parsed but no output was shown.

**Root Cause**: Command implementations were using `crate::println!()` which writes to VGA only, but the user is running with `-nographic` which disables VGA.

**Solution**: 
- Refactored all command implementations to use `serial_println!()` instead of `println!()`
- Serial output goes directly to stdout via QEMU's `-serial stdio` flag
- This is visible in nographic mode

### 3. Module Structure
**Problem**: Multiple re-exports of `run()` functions caused naming conflicts.

**Solution**: 
- Removed wildcard re-exports from submodules
- Implemented all command logic directly in commands/mod.rs execute() function
- Organized commands into logical groups: system, info, file, process

## Architecture

### Directory Structure
```
src/userland/shell/
├── mod.rs                      # Module definition
└── commands/
    ├── mod.rs                  # Command dispatcher
    ├── system/                 # System commands (help, clear, exit)
    ├── info/                   # Info commands (whoami, id, uname, pwd)
    ├── file/                   # File commands (echo, cat, ls, touch, mkdir, rm, cd, chmod)
    └── process/                # Process commands (ps, fork)
```

### Command Execution Flow
1. Shell reads input from serial port (COM1)
2. Input is parsed in handle_shell_input() 
3. Commands are dispatched to execute() in commands/mod.rs
4. Output is written to serial port using serial_println!()
5. Control returns to shell_loop() which prints next prompt

## Working Commands

All commands now properly display output via serial:
- **help** - Shows all available commands
- **whoami** - Prints "root"
- **id** - Prints user ID info
- **uname** - Prints system info
- **pwd** - Prints working directory "/"
- **echo TEXT** - Echoes text with proper spacing
- **cat FILE** - Shows file read message (stub)
- **ls [DIR]** - Lists directory (stub)
- **touch FILE** - Shows touch message (stub)
- **mkdir DIR** - Shows mkdir message (stub)
- **rm FILE** - Shows remove message (stub)
- **cd DIR** - Shows cd message (stub)
- **chmod MODE FILE** - Shows chmod message (stub)
- **ps** - Lists running processes
- **fork** - Tests fork syscall
- **clear** - Clears VGA screen
- **exit** - Shows exit disabled message

## Testing

Build and test with:
```bash
./run_qemu.sh
```

Then try commands:
```
root@qunix:/# help
root@qunix:/# whoami
root
root@qunix:/# echo Hello World
Hello World
root@qunix:/# id
uid=0(root) gid=0(root) groups=0(root)
root@qunix:/# ps
 PID  NAME
  1   init
root@qunix:/#
```

## Key Changes

- Shell now uses serial I/O consistently
- All command output goes to serial (visible in nographic mode)
- Clean modular command structure for easy extension
- Proper prompt handling with serial awareness
- No more VGA-only output when running with -nographic
