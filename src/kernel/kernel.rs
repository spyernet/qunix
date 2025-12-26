use spin::Mutex;
use lazy_static::lazy_static;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelState {
    Booting,
    Running,
    Halting,
    Panic,
}

lazy_static! {
    static ref KERNEL_STATE: Mutex<KernelState> = Mutex::new(KernelState::Booting);
}

pub fn get_state() -> KernelState {
    *KERNEL_STATE.lock()
}

pub fn set_state(state: KernelState) {
    *KERNEL_STATE.lock() = state;
}

pub fn is_running() -> bool {
    get_state() == KernelState::Running
}

#[derive(Debug)]
pub struct KernelInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub arch: &'static str,
    pub build_date: &'static str,
}

pub const KERNEL_INFO: KernelInfo = KernelInfo {
    name: "Qunix",
    version: env!("CARGO_PKG_VERSION"),
    arch: "x86_64",
    build_date: "2025",
};

pub fn version_string() -> String {
    alloc::format!("{} {} ({}) {}", 
        KERNEL_INFO.name,
        KERNEL_INFO.version,
        KERNEL_INFO.arch,
        KERNEL_INFO.build_date
    )
}

lazy_static! {
    static ref KERNEL_PARAMS: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

pub fn parse_cmdline(cmdline: &str) {
    let mut params = KERNEL_PARAMS.lock();
    params.clear();
    
    for param in cmdline.split_whitespace() {
        params.push(String::from(param));
    }
}

pub fn get_param(key: &str) -> Option<String> {
    let params = KERNEL_PARAMS.lock();
    
    for param in params.iter() {
        if param.starts_with(key) {
            if let Some(pos) = param.find('=') {
                return Some(String::from(&param[pos + 1..]));
            } else if param == key {
                return Some(String::new());
            }
        }
    }
    
    None
}

pub fn has_param(key: &str) -> bool {
    get_param(key).is_some()
}

#[derive(Debug, Clone, Copy)]
pub struct SystemTime {
    pub seconds: u64,
    pub nanoseconds: u32,
}

impl SystemTime {
    pub fn now() -> Self {
        let ticks = crate::hal::drivers::pit::get_ticks();
        SystemTime {
            seconds: ticks / 1000,
            nanoseconds: ((ticks % 1000) * 1_000_000) as u32,
        }
    }
    
    pub fn from_secs(seconds: u64) -> Self {
        SystemTime {
            seconds,
            nanoseconds: 0,
        }
    }
    
    pub fn as_secs(&self) -> u64 {
        self.seconds
    }
    
    pub fn as_millis(&self) -> u64 {
        self.seconds * 1000 + (self.nanoseconds / 1_000_000) as u64
    }
}

pub fn print_boot_banner() {
    use crate::println;
    
    println!();
    println!("  ____              _       ");
    println!(" / __ \\            (_)      ");
    println!("| |  | |_   _ _ __  ___  __ ");
    println!("| |  | | | | | '_ \\| \\ \\/ / ");
    println!("| |__| | |_| | | | | |>  <  ");
    println!(" \\___\\_\\\\__,_|_| |_|_/_/\\_\\ ");
    println!();
    println!("Qunix Operating System v{}", KERNEL_INFO.version);
    println!("Secure. POSIX-Compliant. Rust-Built.");
    println!();
}

pub mod panic {
    use crate::println;
    use core::panic::PanicInfo;
    
    pub fn kernel_panic(info: &PanicInfo) -> ! {
        super::set_state(super::KernelState::Panic);
        
        println!();
        println!("=====================================");
        println!("         KERNEL PANIC");
        println!("=====================================");
        println!();
        
        if let Some(location) = info.location() {
            println!("Location: {}:{}:{}", 
                location.file(),
                location.line(),
                location.column()
            );
        }
        
        if let Some(message) = info.message().as_str() {
            println!("Message: {}", message);
        }
        
        println!();
        println!("System halted.");
        
        crate::hlt_loop()
    }
}

pub mod debug {
    use crate::serial_println;
    
    pub fn log(level: &str, module: &str, message: &str) {
        serial_println!("[{}] {}: {}", level, module, message);
    }
    
    pub fn debug(module: &str, message: &str) {
        log("DEBUG", module, message);
    }
    
    pub fn info(module: &str, message: &str) {
        log("INFO", module, message);
    }
    
    pub fn warn(module: &str, message: &str) {
        log("WARN", module, message);
    }
    
    pub fn error(module: &str, message: &str) {
        log("ERROR", module, message);
    }
}
