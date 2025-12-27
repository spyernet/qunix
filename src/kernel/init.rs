use crate::{print, println};
use crate::kernel::scheduler::{Task, SCHEDULER};
use alloc::string::String;

use x86_64::instructions::interrupts;

pub fn start_init_process() {
    println!("[INIT] Starting init process (PID 1)...");

    let init_task = match Task::new(1, String::from("init"), init_main as usize, true) {
        Ok(task) => task,
        Err(e) => {
            panic!("[INIT] PANIC: Failed to create init task: {}", e);
        }
    };

    println!("[INIT] Init task created successfully");
    println!("[INIT] Disabling interrupts before taking scheduler lock");
    interrupts::disable();

    {
        let mut scheduler = SCHEDULER.lock();
        println!("[INIT] Scheduler locked, adding task");
        scheduler.add_task(init_task);
        println!("[INIT] Task added to scheduler");
    } // lock released here

    println!("[INIT] Re‑enabling interrupts");
    interrupts::enable();

    println!("[INIT] Lock released, calling init_main");
    init_main();
}

fn init_main() {
    println!("[INIT] >>> Entered init_main()");
    crate::serial_println!("[INIT] >>> Entered init_main()");
    
    println!();
    println!("╔══════════════════════════════════╗");
    println!("║     Qunix OS - Shell Ready       ║");
    println!("║   Type 'help' for commands       ║");
    println!("╚══════════════════════════════════╝");
    println!();
    
    crate::serial_println!();
    crate::serial_println!("╔══════════════════════════════════╗");
    crate::serial_println!("║     Qunix OS - Shell Ready       ║");
    crate::serial_println!("║   Type 'help' for commands       ║");
    crate::serial_println!("╚══════════════════════════════════╝");
    crate::serial_println!();

    shell_loop();
}

fn shell_loop() {
    loop {
        // Ensure the prompt is visible on both VGA and serial
        crate::serial_print!("root@qunix:/# ");
        crate::println!("root@qunix:/# "); // Also print to VGA for compatibility
        
        let mut buf = [0u8; 128];
        let len = crate::hal::drivers::serial::read_line(&mut buf);
        
        let line = core::str::from_utf8(&buf[..len]).unwrap_or("");
        if !line.is_empty() {
            handle_shell_input(line);
        }
    }
}

pub fn handle_shell_input(input: &str) {
    let input = input.trim();
    if input.is_empty() {
        return;
    }

    let mut args: [&str; 16] = [""; 16];
    let mut count = 0;
    let mut iter = input.split_whitespace();
    while let Some(token) = iter.next() {
        if count < 16 {
            args[count] = token;
            count += 1;
        } else {
            break;
        }
    }
    if count == 0 {
        return;
    }

    let command = args[0];
    let actual_args = &args[1..count];

    // Use the new modular command system
    crate::userland::shell::execute(command, actual_args);
}

// Shell input is now handled by modular command system
// See src/userland/shell/commands/ for individual command implementations
