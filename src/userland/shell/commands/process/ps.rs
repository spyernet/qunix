// ps - List running processes

use crate::kernel::scheduler::SCHEDULER;

pub fn run() {
    crate::println!(" PID  NAME");
    // Use try_lock which returns Option
    match SCHEDULER.try_lock() {
        Some(scheduler) => {
            for task in scheduler.get_tasks() {
                crate::println!("  {}  {}", task.pid, task.name);
            }
        }
        None => {
            crate::println!("  1   init");
            crate::println!("(scheduler busy, showing init only)");
        }
    }
}
