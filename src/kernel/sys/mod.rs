pub mod posix;
pub mod syscalls;

pub use posix::*;
pub use syscalls::*;

use x86_64::structures::idt::InterruptStackFrame;
use crate::hal::drivers::keyboard::SpecialKey;
use core::sync::atomic::{AtomicUsize, Ordering};

const INPUT_BUF_SIZE: usize = 256;

static mut INPUT_BUFFER: [u8; INPUT_BUF_SIZE] = [0; INPUT_BUF_SIZE];
static INPUT_LEN: AtomicUsize = AtomicUsize::new(0);

pub fn init() {
    INPUT_LEN.store(0, Ordering::SeqCst);
}

pub fn handle_syscall_interrupt(_stack_frame: &InterruptStackFrame) {
}

pub fn handle_keyboard_input(c: char) {
    unsafe {
        let len = INPUT_LEN.load(Ordering::SeqCst);

        match c {
            '\n' | '\r' => {
                crate::println!();

                // build &str view of current buffer
                let slice = &INPUT_BUFFER[..len];
                if let Ok(line) = core::str::from_utf8(slice) {
                    crate::kernel::init::handle_shell_input(line);
                }

                // clear buffer
                INPUT_LEN.store(0, Ordering::SeqCst);
            }

            '\x08' | '\x7f' => {
                if len > 0 {
                    INPUT_LEN.store(len - 1, Ordering::SeqCst);
                    crate::print!("\x08 \x08");
                }
            }

            ch if ch.is_ascii() && !ch.is_control() => {
                if len < INPUT_BUF_SIZE {
                    INPUT_BUFFER[len] = ch as u8;
                    INPUT_LEN.store(len + 1, Ordering::SeqCst);
                    crate::print!("{}", ch);
                }
            }

            _ => {}
        }
    }
}

pub fn handle_special_key(key: SpecialKey) {
    match key {
        SpecialKey::ArrowUp => {
        }
        SpecialKey::ArrowDown => {
        }
        SpecialKey::ArrowLeft => {
        }
        SpecialKey::ArrowRight => {
        }
        _ => {}
    }
}
