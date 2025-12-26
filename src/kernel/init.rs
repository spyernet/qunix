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
    
    println!();
    println!("╔══════════════════════════════════╗");
    println!("║     Qunix OS - Shell Ready       ║");
    println!("║   Type 'help' for commands       ║");
    println!("╚══════════════════════════════════╝");
    println!();

    shell_loop();
}

fn shell_loop() {
    loop {
        print!("root@qunix:/# ");
        
        let mut buf = [0u8; 128];
        let mut len = 0;

        loop {
            let c = crate::hal::drivers::keyboard::read_char_blocking();
            
            match c {
                '\n' | '\r' => {
                    println!();
                    break;
                }
                '\u{8}' => {
                    if len > 0 {
                        len -= 1;
                        print!("\u{8} \u{8}");
                    }
                }
                _ => {
                    if len < buf.len() {
                        buf[len] = c as u8;
                        len += 1;
                        print!("{}", c);
                    }
                }
            }
        }

        let line = core::str::from_utf8(&buf[..len]).unwrap_or("");
        handle_shell_input(line);
    }
}

pub fn handle_shell_input(input: &str) {
    let input = input.trim();
    if input.is_empty() {
        return;
    }

    let mut args: [&str; 8] = [""; 8];
    let mut count = 0;
    let mut iter = input.split_whitespace();
    while let Some(token) = iter.next() {
        if count < 8 {
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

    match command {
        "help" => cmd_help(),
        "whoami" => cmd_whoami(),
        "uname" => cmd_uname(),
        "pwd" => cmd_pwd(),
        "echo" => cmd_echo(actual_args),
        "clear" => cmd_clear(),
        "ps" => cmd_ps(),
        "exit" => cmd_exit(),
        _ => println!("command not found: {}", command),
    }
}

fn cmd_help() {
    println!("Qunix Shell - Available Commands:");
    println!("  help    - Show this help message");
    println!("  whoami  - Print current user");
    println!("  uname   - Print system information");
    println!("  pwd     - Print working directory");
    println!("  echo    - Echo text to terminal");
    println!("  clear   - Clear the screen");
    println!("  ps      - List running processes");
    println!("  exit    - Exit shell (disabled in init)");
}

fn cmd_uname() {
    println!("Qunix 1.0 x86_64");
}

fn cmd_whoami() {
    println!("root");
}

fn cmd_pwd() {
    println!("/");
}

fn cmd_echo(args: &[&str]) {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 { print!(" "); }
        print!("{}", arg);
    }
    println!();
}

fn cmd_clear() {
    crate::hal::drivers::vga::clear_screen();
}

fn cmd_ps() {
    println!(" PID  NAME");
    // Use try_lock which returns Option
    match SCHEDULER.try_lock() {
        Some(scheduler) => {
            for task in scheduler.get_tasks() {
                println!("  {}  {}", task.pid, task.name);
            }
        }
        None => {
            println!("  1   init");
            println!("(scheduler busy, showing init only)");
        }
    }
}

fn cmd_exit() {
    println!("Cannot exit from init shell. Use 'reboot' to restart.");
}
