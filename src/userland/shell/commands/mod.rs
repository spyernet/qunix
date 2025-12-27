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
        "whoami" => info::whoami::run(),
        "id" => info::id::run(),
        "uname" => info::uname::run(),
        "pwd" => info::pwd::run(),
        
        // File commands
        "echo" => file::echo::run(args),
        "cat" => file::cat::run(args),
        "ls" => file::ls::run(args),
        "touch" => file::touch::run(args),
        "mkdir" => file::mkdir::run(args),
        "rm" => file::rm::run(args),
        "cd" => file::cd::run(args),
        "chmod" => file::chmod::run(args),
        
        // Process commands
        "ps" => process::ps::run(),
        "fork" => process::fork::run(),
        
        _ => {
            serial_println!("command not found: {}", command);
        },
    }
}
