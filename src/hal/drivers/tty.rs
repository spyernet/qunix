use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::VecDeque;
use spin::Mutex;
use lazy_static::lazy_static;
use crate::hal::drivers::vga::{WRITER, Color};
use crate::hal::drivers::keyboard;
use crate::print;

const TTY_BUFFER_SIZE: usize = 4096;
const MAX_LINE_LENGTH: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtyMode {
    Raw,
    Canonical,
    Cbreak,
}

#[derive(Debug, Clone)]
pub struct TerminalSettings {
    pub echo: bool,
    pub canonical: bool,
    pub signal_chars: bool,
    pub erase_char: char,
    pub kill_char: char,
    pub eof_char: char,
    pub intr_char: char,
    pub quit_char: char,
    pub susp_char: char,
}

impl Default for TerminalSettings {
    fn default() -> Self {
        TerminalSettings {
            echo: true,
            canonical: true,
            signal_chars: true,
            erase_char: '\x7f',
            kill_char: '\x15',
            eof_char: '\x04',
            intr_char: '\x03',
            quit_char: '\x1c',
            susp_char: '\x1a',
        }
    }
}

pub struct Tty {
    pub id: usize,
    pub input_buffer: VecDeque<u8>,
    pub output_buffer: VecDeque<u8>,
    pub line_buffer: String,
    pub settings: TerminalSettings,
    pub mode: TtyMode,
    pub foreground_pid: Option<u32>,
    pub rows: usize,
    pub cols: usize,
    pub cursor_row: usize,
    pub cursor_col: usize,
}

impl Tty {
    pub fn new(id: usize) -> Self {
        Tty {
            id,
            input_buffer: VecDeque::with_capacity(TTY_BUFFER_SIZE),
            output_buffer: VecDeque::with_capacity(TTY_BUFFER_SIZE),
            line_buffer: String::with_capacity(MAX_LINE_LENGTH),
            settings: TerminalSettings::default(),
            mode: TtyMode::Canonical,
            foreground_pid: None,
            rows: 25,
            cols: 80,
            cursor_row: 0,
            cursor_col: 0,
        }
    }
    
    pub fn write_byte(&mut self, byte: u8) {
        self.output_buffer.push_back(byte);
        self.flush_output();
    }
    
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            self.output_buffer.push_back(byte);
        }
        self.flush_output();
    }
    
    pub fn flush_output(&mut self) {
        while let Some(byte) = self.output_buffer.pop_front() {
            print!("{}", byte as char);
        }
    }
    
    pub fn read_byte(&mut self) -> Option<u8> {
        match self.mode {
            TtyMode::Raw | TtyMode::Cbreak => {
                self.input_buffer.pop_front()
            }
            TtyMode::Canonical => {
                if self.line_buffer.is_empty() {
                    None
                } else {
                    Some(self.line_buffer.remove(0) as u8)
                }
            }
        }
    }
    
    pub fn read_line(&mut self) -> Option<String> {
        if self.mode != TtyMode::Canonical {
            return None;
        }
        
        if !self.line_buffer.is_empty() {
            let line = self.line_buffer.clone();
            self.line_buffer.clear();
            Some(line)
        } else {
            None
        }
    }
    
    pub fn handle_input(&mut self, c: char) {
        match self.mode {
            TtyMode::Raw => {
                self.input_buffer.push_back(c as u8);
            }
            TtyMode::Cbreak => {
                if self.settings.signal_chars {
                    if c == self.settings.intr_char {
                        self.send_signal(2);
                        return;
                    } else if c == self.settings.quit_char {
                        self.send_signal(3);
                        return;
                    } else if c == self.settings.susp_char {
                        self.send_signal(20);
                        return;
                    }
                }
                self.input_buffer.push_back(c as u8);
                if self.settings.echo {
                    self.write_byte(c as u8);
                }
            }
            TtyMode::Canonical => {
                if self.settings.signal_chars {
                    if c == self.settings.intr_char {
                        self.send_signal(2);
                        if self.settings.echo {
                            self.write_string("^C\n");
                        }
                        self.line_buffer.clear();
                        return;
                    }
                }
                
                if c == self.settings.erase_char || c == '\x08' {
                    if !self.line_buffer.is_empty() {
                        self.line_buffer.pop();
                        if self.settings.echo {
                            self.write_string("\x08 \x08");
                        }
                    }
                    return;
                }
                
                if c == self.settings.kill_char {
                    let len = self.line_buffer.len();
                    self.line_buffer.clear();
                    if self.settings.echo {
                        for _ in 0..len {
                            self.write_string("\x08 \x08");
                        }
                    }
                    return;
                }
                
                if c == '\n' || c == '\r' {
                    self.line_buffer.push('\n');
                    if self.settings.echo {
                        self.write_byte(b'\n');
                    }
                    return;
                }
                
                if self.line_buffer.len() < MAX_LINE_LENGTH {
                    self.line_buffer.push(c);
                    if self.settings.echo {
                        self.write_byte(c as u8);
                    }
                }
            }
        }
    }
    
    fn send_signal(&self, _signal: i32) {
    }
    
    pub fn set_mode(&mut self, mode: TtyMode) {
        self.mode = mode;
    }
    
    pub fn set_echo(&mut self, echo: bool) {
        self.settings.echo = echo;
    }
    
    pub fn set_canonical(&mut self, canonical: bool) {
        self.settings.canonical = canonical;
        self.mode = if canonical { TtyMode::Canonical } else { TtyMode::Cbreak };
    }
    
    pub fn clear(&mut self) {
        crate::hal::drivers::vga::clear_screen();
        self.cursor_row = 0;
        self.cursor_col = 0;
    }
    
    pub fn set_cursor(&mut self, row: usize, col: usize) {
        if row < self.rows && col < self.cols {
            self.cursor_row = row;
            self.cursor_col = col;
            WRITER.lock().set_position(row, col);
        }
    }
    
    pub fn get_size(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }
    
    pub fn data_available(&self) -> bool {
        match self.mode {
            TtyMode::Raw | TtyMode::Cbreak => !self.input_buffer.is_empty(),
            TtyMode::Canonical => self.line_buffer.contains('\n'),
        }
    }
}

lazy_static! {
    static ref TTYS: Mutex<Vec<Tty>> = {
        let mut ttys = Vec::new();
        for i in 0..8 {
            ttys.push(Tty::new(i));
        }
        Mutex::new(ttys)
    };
    
    static ref CURRENT_TTY: Mutex<usize> = Mutex::new(0);
}

pub fn get_current_tty() -> usize {
    *CURRENT_TTY.lock()
}

pub fn switch_tty(id: usize) {
    let ttys = TTYS.lock();
    if id < ttys.len() {
        *CURRENT_TTY.lock() = id;
    }
}

pub fn write_to_tty(id: usize, s: &str) {
    let mut ttys = TTYS.lock();
    if id < ttys.len() {
        ttys[id].write_string(s);
    }
}

pub fn read_from_tty(id: usize) -> Option<u8> {
    let mut ttys = TTYS.lock();
    if id < ttys.len() {
        ttys[id].read_byte()
    } else {
        None
    }
}

pub fn handle_tty_input(c: char) {
    let current = *CURRENT_TTY.lock();
    let mut ttys = TTYS.lock();
    if current < ttys.len() {
        ttys[current].handle_input(c);
    }
}

pub fn clear_current_tty() {
    let current = *CURRENT_TTY.lock();
    let mut ttys = TTYS.lock();
    if current < ttys.len() {
        ttys[current].clear();
    }
}
