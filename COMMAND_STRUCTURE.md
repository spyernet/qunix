# Command Module Structure

## Overview
Commands have been reorganized into a modular structure similar to GNU coreutils, making it easy to add new commands and maintain existing ones.

## Directory Structure

```
src/userland/shell/commands/
├── mod.rs                      # Main command dispatcher
├── system/
│   ├── mod.rs                  # System command module exports
│   ├── help.rs                 # help command
│   ├── clear.rs                # clear command
│   └── exit.rs                 # exit command
├── info/
│   ├── mod.rs                  # Info command module exports
│   ├── whoami.rs               # whoami command
│   ├── id.rs                   # id command
│   ├── uname.rs                # uname command
│   └── pwd.rs                  # pwd command
├── file/
│   ├── mod.rs                  # File command module exports
│   ├── echo.rs                 # echo command
│   ├── cat.rs                  # cat command
│   ├── ls.rs                   # ls command
│   ├── touch.rs                # touch command
│   ├── mkdir.rs                # mkdir command
│   ├── rm.rs                   # rm command
│   ├── cd.rs                   # cd command
│   └── chmod.rs                # chmod command
└── process/
    ├── mod.rs                  # Process command module exports
    ├── ps.rs                   # ps command
    └── fork.rs                 # fork command
```

## Command Dispatcher

The main `execute()` function in commands/mod.rs routes commands to their implementations:

```rust
pub fn execute(command: &str, args: &[&str]) {
    match command {
        "help" => { /* help implementation */ },
        "whoami" => { /* whoami implementation */ },
        // ... more commands
        _ => { serial_println!("command not found: {}", command); },
    }
}
```

## Adding a New Command

To add a new command (e.g., `grep`):

### 1. Create the command file
Create `src/userland/shell/commands/file/grep.rs`:

```rust
// grep - Search for patterns in files

pub fn run(args: &[&str]) {
    if args.is_empty() {
        crate::serial_println!("Usage: grep <pattern> <file>");
    } else {
        crate::serial_println!("(grep {} would search {})", args[0], args.get(1).unwrap_or(&""));
    }
}
```

### 2. Add to module exports
Update `src/userland/shell/commands/file/mod.rs`:

```rust
pub mod grep;
```

### 3. Add to dispatcher
Update `src/userland/shell/commands/mod.rs`:

```rust
"grep" => {
    if args.is_empty() {
        serial_println!("Usage: grep <pattern> <file>");
    } else {
        serial_println!("(grep {} would search {})", args[0], args.get(1).unwrap_or(&""));
    }
},
```

## Command Categories

### System Commands
- `help` - Show available commands
- `clear` - Clear screen
- `exit` - Exit shell

### Info Commands
- `whoami` - Print current user
- `id` - Print user/group IDs
- `uname` - Print system information
- `pwd` - Print working directory

### File Commands
- `echo` - Echo text
- `cat` - Display file
- `ls` - List directory
- `touch` - Create file
- `mkdir` - Create directory
- `rm` - Remove file
- `cd` - Change directory
- `chmod` - Change permissions

### Process Commands
- `ps` - List processes
- `fork` - Test fork syscall

## Implementation Notes

1. **Output**: All commands use `serial_println!()` for output visibility in nographic mode
2. **Arguments**: Commands receive arguments as `&[&str]` from the shell parser
3. **Error Handling**: Commands print usage on invalid arguments
4. **Stubs**: Many commands print placeholder messages as filesystem is not fully implemented

## Future Enhancements

- [ ] Implement actual filesystem operations (cat, ls, touch, mkdir, rm, cd)
- [ ] Add more GNU coreutils commands (grep, find, sed, awk, etc.)
- [ ] Implement pipes and redirects
- [ ] Add environment variables and shell expansion
- [ ] Implement job control and background processes
- [ ] Add command history and line editing

## Comparison to Old System

### Old System (monolithic)
- All commands in `src/kernel/init.rs` as individual functions
- Mixed VGA and serial output
- Hard to extend or reorganize
- 250+ lines of command code in one file

### New System (modular)
- Commands split across multiple files by category
- Consistent serial output
- Easy to add new commands
- Better code organization and maintainability
- Following GNU coreutils structure conventions
