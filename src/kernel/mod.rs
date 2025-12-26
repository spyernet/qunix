//src/kernel/mod.rs
pub mod scheduler;
pub mod sys;
pub mod init;
pub mod kernel;

pub use init::*;
pub use kernel::*;

use crate::println;

pub fn init() {
    println!("  [KERNEL] Initializing scheduler...");
    scheduler::init();
    
    println!("  [KERNEL] Initializing syscall interface...");
    sys::init();
    
    println!("  [KERNEL] Initializing filesystem...");
    crate::fs::init();
    
    println!("  [KERNEL] Initializing security framework...");
    crate::qsf::init();
}
