pub mod modules;
pub mod policies;
pub mod qsf;

pub use qsf::*;

use crate::println;

pub fn init() {
    println!("  [QSF] Initializing Qunix Security Framework...");
    qsf::init_qsf();
    println!("  [QSF] Security framework initialized");
}
