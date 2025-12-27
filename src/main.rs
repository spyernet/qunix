#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(qunix::test_runner)]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use qunix::hal;
use qunix::kernel;
use qunix::println;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Qunix OS v{}", env!("CARGO_PKG_VERSION"));
    println!("=====================================");
    println!("Secure. POSIX-Compliant. Rust-Built.");
    println!();
    
    qunix::serial_println!("=====================================");
    qunix::serial_println!("Qunix OS v{}", env!("CARGO_PKG_VERSION"));
    qunix::serial_println!("Secure. POSIX-Compliant. Rust-Built.");
    qunix::serial_println!("=====================================");

    println!("[BOOT] Initializing Hardware Abstraction Layer...");
    qunix::serial_println!("[BOOT] Initializing Hardware Abstraction Layer...");

    // CRITICAL BOOT ORDER (DO NOT CHANGE):
    // 1. VGA/Serial already initialized by bootloader
    // 2. Frame allocator from boot_info memory map (MUST be first)
    hal::memory::frame_allocator::init_from_boot_info(&boot_info.memory_map);
    println!("[BOOT] Frame allocator initialized");
    qunix::serial_println!("[BOOT] Frame allocator initialized");

    // 3. CPU setup (GDT, IDT, interrupts)
    hal::init(boot_info);
    println!("[BOOT] HAL initialized successfully");
    qunix::serial_println!("[BOOT] HAL initialized successfully");

    // 4. Kernel subsystems (scheduler, VFS, etc.)
    println!("[BOOT] Initializing kernel subsystems...");
    qunix::serial_println!("[BOOT] Initializing kernel subsystems...");
    kernel::init();
    println!("[BOOT] Kernel initialized successfully");
    qunix::serial_println!("[BOOT] Kernel initialized successfully");

    println!();
    println!("[BOOT] Qunix kernel boot complete!");
    println!("[BOOT] Starting init process (PID 1)...");
    println!();
    
    qunix::serial_println!();
    qunix::serial_println!("[BOOT] Qunix kernel boot complete!");
    qunix::serial_println!("[BOOT] Starting init process (PID 1)...");
    qunix::serial_println!();

    #[cfg(test)]
    test_main();

    #[cfg(not(test))]
    kernel::start_init_process();

    qunix::hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use qunix::serial_println;

    println!();
    println!("=====================================");
    println!("KERNEL PANIC!");
    println!("=====================================");
    println!("{}", info);
    println!();

    serial_println!("KERNEL PANIC: {}", info);

    qunix::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    qunix::test_panic_handler(info)
}