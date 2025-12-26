pub mod cpu;
pub mod memory;
pub mod drivers;
pub mod hal;

pub use hal::*;

use bootloader::BootInfo;
use crate::println;

pub fn init(boot_info: &'static BootInfo) {
    println!("  [HAL] Initializing GDT...");
    cpu::gdt::init();
    
    println!("  [HAL] Initializing IDT...");
    cpu::idt::init();
    
    println!("  [HAL] Initializing PIC...");
    unsafe { cpu::interrupts::PICS.lock().initialize() };
    
    println!("  [HAL] Enabling interrupts...");
    x86_64::instructions::interrupts::enable();
    
    println!("  [HAL] Initializing memory management...");
    let phys_mem_offset = x86_64::VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::paging::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        memory::frame_allocator::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    
    println!("  [HAL] Initializing kernel heap...");
    memory::heap::init_heap(&mut mapper, &mut frame_allocator)
        .expect("Heap initialization failed");
    
    println!("  [HAL] Initializing serial port...");
    drivers::serial::init();
    
    println!("  [HAL] Initializing keyboard driver...");
    drivers::keyboard::init();
    
    println!("  [HAL] Initializing PIT timer...");
    drivers::pit::init();
    
    println!("  [HAL] Scanning PCI bus...");
    drivers::pci::scan_bus();
}
