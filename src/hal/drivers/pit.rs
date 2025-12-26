use x86_64::instructions::port::Port;
use spin::Mutex;
use lazy_static::lazy_static;

const PIT_FREQUENCY: u32 = 1193182;
const TARGET_FREQUENCY: u32 = 1000;
const PIT_CHANNEL0: u16 = 0x40;
const PIT_CHANNEL1: u16 = 0x41;
const PIT_CHANNEL2: u16 = 0x42;
const PIT_COMMAND: u16 = 0x43;

lazy_static! {
    static ref TICKS: Mutex<u64> = Mutex::new(0);
    static ref UPTIME_SECONDS: Mutex<u64> = Mutex::new(0);
}

pub fn init() {
    set_frequency(TARGET_FREQUENCY);
}

pub fn set_frequency(frequency: u32) {
    let divisor = PIT_FREQUENCY / frequency;
    
    unsafe {
        let mut command_port = Port::<u8>::new(PIT_COMMAND);
        let mut channel0_port = Port::<u8>::new(PIT_CHANNEL0);
        
        command_port.write(0x36);
        
        channel0_port.write((divisor & 0xFF) as u8);
        channel0_port.write(((divisor >> 8) & 0xFF) as u8);
    }
}

pub fn tick() {
    let mut ticks = TICKS.lock();
    *ticks += 1;
    
    if *ticks % TARGET_FREQUENCY as u64 == 0 {
        let mut seconds = UPTIME_SECONDS.lock();
        *seconds += 1;
    }
}

pub fn get_ticks() -> u64 {
    *TICKS.lock()
}

pub fn get_uptime_seconds() -> u64 {
    *UPTIME_SECONDS.lock()
}

pub fn get_uptime_ms() -> u64 {
    *TICKS.lock()
}

pub fn sleep_ms(milliseconds: u64) {
    let start = get_ticks();
    while get_ticks() - start < milliseconds {
        x86_64::instructions::hlt();
    }
}

pub fn sleep_seconds(seconds: u64) {
    sleep_ms(seconds * 1000);
}

pub fn busy_wait_us(microseconds: u64) {
    let cycles = microseconds * (PIT_FREQUENCY as u64 / 1_000_000);
    for _ in 0..cycles {
        core::hint::spin_loop();
    }
}

pub fn read_counter() -> u16 {
    unsafe {
        let mut command_port = Port::<u8>::new(PIT_COMMAND);
        let mut channel0_port = Port::<u8>::new(PIT_CHANNEL0);
        
        command_port.write(0x00);
        
        let low = channel0_port.read() as u16;
        let high = channel0_port.read() as u16;
        
        (high << 8) | low
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Timer {
    start_ticks: u64,
    duration_ms: u64,
}

impl Timer {
    pub fn new(duration_ms: u64) -> Self {
        Timer {
            start_ticks: get_ticks(),
            duration_ms,
        }
    }
    
    pub fn is_expired(&self) -> bool {
        get_ticks() - self.start_ticks >= self.duration_ms
    }
    
    pub fn remaining_ms(&self) -> u64 {
        let elapsed = get_ticks() - self.start_ticks;
        if elapsed >= self.duration_ms {
            0
        } else {
            self.duration_ms - elapsed
        }
    }
    
    pub fn reset(&mut self) {
        self.start_ticks = get_ticks();
    }
    
    pub fn elapsed_ms(&self) -> u64 {
        get_ticks() - self.start_ticks
    }
}

pub struct Stopwatch {
    start_ticks: u64,
}

impl Stopwatch {
    pub fn start() -> Self {
        Stopwatch {
            start_ticks: get_ticks(),
        }
    }
    
    pub fn elapsed_ms(&self) -> u64 {
        get_ticks() - self.start_ticks
    }
    
    pub fn elapsed_seconds(&self) -> u64 {
        self.elapsed_ms() / 1000
    }
    
    pub fn reset(&mut self) {
        self.start_ticks = get_ticks();
    }
}
