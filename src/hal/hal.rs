use spin::Mutex;
use lazy_static::lazy_static;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    X86_64,
    Aarch64,
    Riscv64,
}

impl Architecture {
    pub fn current() -> Self {
        Architecture::X86_64
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            Architecture::X86_64 => "x86_64",
            Architecture::Aarch64 => "aarch64",
            Architecture::Riscv64 => "riscv64",
        }
    }
    
    pub fn page_size(&self) -> usize {
        4096
    }
    
    pub fn pointer_width(&self) -> usize {
        8
    }
}

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub arch: Architecture,
    pub total_memory: u64,
    pub available_memory: u64,
    pub cpu_count: usize,
    pub boot_time: u64,
}

lazy_static! {
    pub static ref SYSTEM_INFO: Mutex<SystemInfo> = Mutex::new(SystemInfo {
        arch: Architecture::X86_64,
        total_memory: 0,
        available_memory: 0,
        cpu_count: 1,
        boot_time: 0,
    });
}

pub fn get_system_info() -> SystemInfo {
    SYSTEM_INFO.lock().clone()
}

pub fn update_memory_info(total: u64, available: u64) {
    let mut info = SYSTEM_INFO.lock();
    info.total_memory = total;
    info.available_memory = available;
}

pub fn uptime_ticks() -> u64 {
    super::drivers::pit::get_ticks()
}

pub fn uptime_seconds() -> u64 {
    uptime_ticks() / 1000
}

pub trait Driver: Send + Sync {
    fn name(&self) -> &'static str;
    fn init(&mut self) -> Result<(), &'static str>;
    fn shutdown(&mut self) -> Result<(), &'static str>;
}

lazy_static! {
    pub static ref DRIVERS: Mutex<Vec<&'static dyn Driver>> = Mutex::new(Vec::new());
}

pub fn register_driver(driver: &'static dyn Driver) {
    DRIVERS.lock().push(driver);
}

pub fn disable_interrupts() {
    x86_64::instructions::interrupts::disable();
}

pub fn enable_interrupts() {
    x86_64::instructions::interrupts::enable();
}

pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    x86_64::instructions::interrupts::without_interrupts(f)
}

pub fn halt() {
    x86_64::instructions::hlt();
}

pub fn halt_loop() -> ! {
    loop {
        halt();
    }
}

#[inline]
pub fn io_wait() {
    unsafe {
        x86_64::instructions::port::Port::<u8>::new(0x80).write(0);
    }
}
