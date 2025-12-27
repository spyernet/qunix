// ps - List running processes

use crate::kernel::scheduler::SCHEDULER;

pub fn run() {
    crate::serial_println!(" PID  NAME");
    // Use try_lock which returns Option
    match SCHEDULER.try_lock() {
        Some(scheduler) => {
            for task in scheduler.get_tasks() {
                crate::serial_println!("  {}  {}", task.pid, task.name);
            }
        }
        None => {
            crate::serial_println!("  1   init");
            crate::serial_println!("(scheduler busy, showing init only)");
        }
    }
}
