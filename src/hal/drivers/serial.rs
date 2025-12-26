use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;
use core::fmt;

const COM1_PORT: u16 = 0x3F8;
const COM2_PORT: u16 = 0x2F8;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(COM1_PORT) };
        serial_port.init();
        Mutex::new(serial_port)
    };
    
    pub static ref SERIAL2: Mutex<Option<SerialPort>> = Mutex::new(None);
}

pub fn init() {
    let mut serial2 = SERIAL2.lock();
    let mut port = unsafe { SerialPort::new(COM2_PORT) };
    port.init();
    *serial2 = Some(port);
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        SERIAL1
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    });
}

pub fn write_byte(byte: u8) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        SERIAL1.lock().send(byte);
    });
}

pub fn read_byte() -> Option<u8> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut serial = SERIAL1.lock();
        if serial_data_available(&serial) {
            Some(serial.receive())
        } else {
            None
        }
    })
}

fn serial_data_available(serial: &SerialPort) -> bool {
    unsafe {
        let mut status_port = x86_64::instructions::port::Port::<u8>::new(COM1_PORT + 5);
        (status_port.read() & 0x01) != 0
    }
}

pub fn write_string(s: &str) {
    for byte in s.bytes() {
        write_byte(byte);
    }
}

pub struct SerialWriter;

impl fmt::Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write_string(s);
        Ok(())
    }
}

pub fn get_serial_writer() -> SerialWriter {
    SerialWriter
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_serial_output() {
        crate::serial_println!("test_serial_output");
    }
}
