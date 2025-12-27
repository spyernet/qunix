// help - Show available commands

pub fn run() {
    crate::println!("Qunix Shell - Available Commands:");
    crate::println!();
    crate::println!("System Info:");
    crate::println!("  help      - Show this help message");
    crate::println!("  whoami    - Print current user");
    crate::println!("  uname     - Print system information");
    crate::println!("  id        - Print user ID information");
    crate::println!("  pwd       - Print working directory");
    crate::println!();
    crate::println!("File Operations:");
    crate::println!("  echo TEXT - Echo text to terminal");
    crate::println!("  cat FILE  - Display file contents");
    crate::println!("  ls [DIR]  - List directory contents");
    crate::println!("  touch FILE- Create empty file");
    crate::println!("  mkdir DIR - Create directory");
    crate::println!("  rm FILE   - Remove file");
    crate::println!("  cd DIR    - Change directory");
    crate::println!("  chmod MODE FILE - Change file permissions");
    crate::println!();
    crate::println!("System:");
    crate::println!("  clear     - Clear the screen");
    crate::println!("  ps        - List running processes");
    crate::println!("  fork      - Test fork syscall");
    crate::println!("  exit      - Exit shell (disabled in init)");
}
