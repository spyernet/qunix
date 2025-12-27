// Shell commands module - organized like GNU coreutils
// POSIX-compatible command implementations

pub mod system;
pub mod file;
pub mod process;
pub mod info;

// Don't re-export everything due to naming conflicts
// Instead, access commands directly or through the execute function

/// Execute a shell command with arguments
pub fn execute(command: &str, args: &[&str]) {
    use crate::serial_println;
    use crate::serial_print;
    
    match command {
        // System commands
        "help" => {
            serial_println!("Qunix Shell - Available Commands:");
            serial_println!();
            serial_println!("System Info:");
            serial_println!("  help      - Show this help message");
            serial_println!("  whoami    - Print current user");
            serial_println!("  uname     - Print system information");
            serial_println!("  id        - Print user ID information");
            serial_println!("  pwd       - Print working directory");
            serial_println!();
            serial_println!("File Operations:");
            serial_println!("  echo TEXT - Echo text to terminal");
            serial_println!("  cat FILE  - Display file contents");
            serial_println!("  ls [DIR]  - List directory contents");
            serial_println!("  touch FILE- Create empty file");
            serial_println!("  mkdir DIR - Create directory");
            serial_println!("  rm FILE   - Remove file");
            serial_println!("  cd DIR    - Change directory");
            serial_println!("  chmod MODE FILE - Change file permissions");
            serial_println!();
            serial_println!("System:");
            serial_println!("  clear     - Clear the screen");
            serial_println!("  ps        - List running processes");
            serial_println!("  fork      - Test fork syscall");
            serial_println!("  exit      - Exit shell (disabled in init)");
        },
        "clear" => crate::hal::drivers::vga::clear_screen(),
        "exit" => {
            serial_println!("Cannot exit from init shell. Use 'reboot' to restart.");
        },
        
        // Info commands
        "whoami" => {
            serial_println!("root");
        },
        "id" => {
            serial_println!("uid=0(root) gid=0(root) groups=0(root)");
        },
        "uname" => {
            serial_println!("Qunix 1.0 x86_64");
        },
        "pwd" => {
            serial_println!("/");
        },
        
        // File commands
        "echo" => {
            for (i, arg) in args.iter().enumerate() {
                if i > 0 { 
                    serial_print!(" ");
                }
                serial_print!("{}", arg);
            }
            serial_println!();
        },
        "cat" => {
            if args.is_empty() {
                serial_println!("Usage: cat <file>");
            } else {
                for filename in args {
                    serial_println!("(cat would read: {})", filename);
                }
            }
        },
        "ls" => {
            let dir = if args.is_empty() { "/" } else { args[0] };
            serial_println!("Listing directory: {}", dir);
            serial_println!("(filesystem read not fully implemented)");
        },
        "touch" => {
            if args.is_empty() {
                serial_println!("Usage: touch <file>");
            } else {
                for filename in args {
                    serial_println!("(touch would create: {})", filename);
                }
            }
        },
        "mkdir" => {
            if args.is_empty() {
                serial_println!("Usage: mkdir <directory>");
            } else {
                for dirname in args {
                    serial_println!("(mkdir would create: {})", dirname);
                }
            }
        },
        "rm" => {
            if args.is_empty() {
                serial_println!("Usage: rm <file>");
            } else {
                for filename in args {
                    serial_println!("(rm would delete: {})", filename);
                }
            }
        },
        "cd" => {
            if args.is_empty() {
                serial_println!("Usage: cd <directory>");
            } else {
                let newdir = args[0];
                serial_println!("(cd would change to: {})", newdir);
            }
        },
        "chmod" => {
            if args.len() < 2 {
                serial_println!("Usage: chmod <mode> <file>");
            } else {
                serial_println!("(chmod {} {})", args[0], args[1]);
            }
        },
        
        // Process commands
        "ps" => {
            serial_println!(" PID  NAME");
            use crate::kernel::scheduler::SCHEDULER;
            match SCHEDULER.try_lock() {
                Some(scheduler) => {
                    for task in scheduler.get_tasks() {
                        serial_println!("  {}  {}", task.pid, task.name);
                    }
                }
                None => {
                    serial_println!("  1   init");
                    serial_println!("(scheduler busy, showing init only)");
                }
            }
        },
        "fork" => {
            let pid: i32 = unsafe {
                let result: i64;
                core::arch::asm!(
                    "mov rax, 57; syscall",
                    out("rax") result,
                    options(nostack, preserves_flags)
                );
                result as i32
            };
            
            if pid == 0 {
                serial_println!("[CHILD] This is the child process");
            } else if pid > 0 {
                serial_println!("[PARENT] Forked child process: {}", pid);
            } else {
                serial_println!("fork() failed with error code: {}", pid);
            }
        },
        
        _ => {
            serial_println!("command not found: {}", command);
        },
    }
}
