pub mod gdt;
pub mod idt;
pub mod interrupts;

pub use gdt::init;
pub use interrupts::*;
