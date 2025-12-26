use volatile::Volatile;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use core::ptr::{read_volatile, write_volatile};

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
const VGA_BUFFER_ADDR: usize = 0xb8000;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    row_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    // Helper: pointer to a ScreenChar in the buffer (raw pointer)
    #[inline(always)]
    fn screenchar_ptr(&self, row: usize, col: usize) -> *mut ScreenChar {
        // Get pointer to the Volatile<ScreenChar>, then reinterpret as pointer to ScreenChar
        &self.buffer.chars[row][col] as *const Volatile<ScreenChar> as *mut ScreenChar
    }

    // Helper: write a ScreenChar via a volatile store
    #[inline(always)]
    unsafe fn write_screenchar_ptr(&mut self, row: usize, col: usize, value: ScreenChar) {
        let ptr = self.screenchar_ptr(row, col);
        write_volatile(ptr, value);
    }

    // Helper: read a ScreenChar via a volatile load
    #[inline(always)]
    unsafe fn read_screenchar_ptr(&self, row: usize, col: usize) -> ScreenChar {
        let ptr = self.screenchar_ptr(row, col);
        read_volatile(ptr)
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            b'\r' => self.column_position = 0,
            b'\t' => {
                let spaces = 4 - (self.column_position % 4);
                for _ in 0..spaces {
                    self.write_byte(b' ');
                }
            }
            0x08 => {
                if self.column_position > 0 {
                    self.column_position -= 1;
                    unsafe {
                        self.write_screenchar_ptr(self.row_position, self.column_position, ScreenChar {
                            ascii_character: b' ',
                            color_code: self.color_code,
                        });
                    }
                }
            }
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row_position;
                let col = self.column_position;

                let color_code = self.color_code;
                unsafe {
                    self.write_screenchar_ptr(row, col, ScreenChar {
                        ascii_character: byte,
                        color_code,
                    });
                }
                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' | b'\r' | b'\t' | 0x08 => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        if self.row_position >= BUFFER_HEIGHT - 1 {
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    // replaced Volatile.read / write with volatile pointer operations
                    let character = unsafe { self.read_screenchar_ptr(row, col) };
                    unsafe { self.write_screenchar_ptr(row - 1, col, character) };
                }
            }
            self.clear_row(BUFFER_HEIGHT - 1);
        } else {
            self.row_position += 1;
        }
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            unsafe { self.write_screenchar_ptr(row, col, blank) };
        }
    }

    pub fn clear(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
        self.row_position = 0;
        self.column_position = 0;
    }

    pub fn set_color(&mut self, foreground: Color, background: Color) {
        self.color_code = ColorCode::new(foreground, background);
    }

    pub fn get_position(&self) -> (usize, usize) {
        (self.row_position, self.column_position)
    }

    pub fn set_position(&mut self, row: usize, col: usize) {
        if row < BUFFER_HEIGHT && col < BUFFER_WIDTH {
            self.row_position = row;
            self.column_position = col;
        }
    }

    fn update_cursor(&self) {
        let pos = self.row_position * BUFFER_WIDTH + self.column_position;
        
        unsafe {
            use x86_64::instructions::port::Port;
            let mut cmd_port = Port::<u8>::new(0x3D4);
            let mut data_port = Port::<u8>::new(0x3D5);
            
            cmd_port.write(0x0F);
            data_port.write((pos & 0xFF) as u8);
            cmd_port.write(0x0E);
            data_port.write(((pos >> 8) & 0xFF) as u8);
        }
    }

    pub fn enable_cursor(&mut self, cursor_start: u8, cursor_end: u8) {
        unsafe {
            use x86_64::instructions::port::Port;
            let mut cmd_port = Port::<u8>::new(0x3D4);
            let mut data_port = Port::<u8>::new(0x3D5);
            
            cmd_port.write(0x0A);
            data_port.write((cursor_start & 0x1F) | 0x20);
            
            cmd_port.write(0x0B);
            data_port.write(cursor_end & 0x1F);
        }
    }

    pub fn disable_cursor(&mut self) {
        unsafe {
            use x86_64::instructions::port::Port;
            let mut cmd_port = Port::<u8>::new(0x3D4);
            let mut data_port = Port::<u8>::new(0x3D5);
            
            cmd_port.write(0x0A);
            data_port.write(0x20);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        row_position: 0,
        color_code: ColorCode::new(Color::LightGreen, Color::Black),
        buffer: unsafe { &mut *(VGA_BUFFER_ADDR as *mut Buffer) },
    });
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

pub fn clear_screen() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().clear();
    });
}

pub fn set_color(foreground: Color, background: Color) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().set_color(foreground, background);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_println_simple() {
        crate::println!("test_println_simple output");
    }

    #[test_case]
    fn test_println_many() {
        for _ in 0..200 {
            crate::println!("test_println_many output");
        }
    }

    #[test_case]
    fn test_println_output() {
        use core::fmt::Write;
        use x86_64::instructions::interrupts;

        let s = "Some test string that fits on a single line";
        interrupts::without_interrupts(|| {
            let mut writer = WRITER.lock();
            writeln!(writer, "\n{}", s).expect("writeln failed");
            for (i, c) in s.chars().enumerate() {
                let screen_char = unsafe { writer.read_screenchar_ptr(BUFFER_HEIGHT - 2, i) };
                assert_eq!(char::from(screen_char.ascii_character), c);
            }
        });
    }
}
