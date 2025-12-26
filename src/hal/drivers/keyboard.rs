extern crate alloc;

use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use spin::Mutex;
use lazy_static::lazy_static;

const KEYBOARD_BUFFER_SIZE: usize = 256;

#[allow(dead_code)]
struct CharRingBuffer {
    buf: [char; KEYBOARD_BUFFER_SIZE],
    head: usize,
    tail: usize,
    full: bool,
}

#[allow(dead_code)]
impl CharRingBuffer {
    const fn new() -> Self {
        Self {
            buf: ['\0'; KEYBOARD_BUFFER_SIZE],
            head: 0,
            tail: 0,
            full: false,
        }
    }

    fn push(&mut self, v: char) {
        self.buf[self.head] = v;
        self.head = (self.head + 1) % KEYBOARD_BUFFER_SIZE;
        if self.full {
            self.tail = (self.tail + 1) % KEYBOARD_BUFFER_SIZE;
        }
        self.full = self.head == self.tail;
    }

    fn pop(&mut self) -> Option<char> {
        if self.head == self.tail && !self.full {
            return None;
        }
        let v = self.buf[self.tail];
        self.tail = (self.tail + 1) % KEYBOARD_BUFFER_SIZE;
        self.full = false;
        Some(v)
    }

    fn is_empty(&self) -> bool {
        self.head == self.tail && !self.full
    }

    fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
        self.full = false;
    }

    fn len(&self) -> usize {
        if self.full {
            KEYBOARD_BUFFER_SIZE
        } else if self.head >= self.tail {
            self.head - self.tail
        } else {
            KEYBOARD_BUFFER_SIZE - self.tail + self.head
        }
    }

    fn front(&self) -> Option<char> {
        if self.is_empty() {
            None
        } else {
            Some(self.buf[self.tail])
        }
    }
}

#[allow(dead_code)]
struct U8RingBuffer {
    buf: [u8; KEYBOARD_BUFFER_SIZE],
    head: usize,
    tail: usize,
    full: bool,
}

#[allow(dead_code)]
impl U8RingBuffer {
    const fn new() -> Self {
        Self {
            buf: [0u8; KEYBOARD_BUFFER_SIZE],
            head: 0,
            tail: 0,
            full: false,
        }
    }

    fn push(&mut self, v: u8) {
        self.buf[self.head] = v;
        self.head = (self.head + 1) % KEYBOARD_BUFFER_SIZE;
        if self.full {
            self.tail = (self.tail + 1) % KEYBOARD_BUFFER_SIZE;
        }
        self.full = self.head == self.tail;
    }

    fn pop(&mut self) -> Option<u8> {
        if self.head == self.tail && !self.full {
            return None;
        }
        let v = self.buf[self.tail];
        self.tail = (self.tail + 1) % KEYBOARD_BUFFER_SIZE;
        self.full = false;
        Some(v)
    }

    fn is_empty(&self) -> bool {
        self.head == self.tail && !self.full
    }

    fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
        self.full = false;
    }

    fn len(&self) -> usize {
        if self.full {
            KEYBOARD_BUFFER_SIZE
        } else if self.head >= self.tail {
            self.head - self.tail
        } else {
            KEYBOARD_BUFFER_SIZE - self.tail + self.head
        }
    }
}

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
        Keyboard::new(
            ScancodeSet1::new(),
            layouts::Us104Key,
            HandleControl::Ignore,
        )
    );

    static ref KEY_BUFFER: Mutex<CharRingBuffer> = Mutex::new(CharRingBuffer::new());
    static ref SCANCODE_BUFFER: Mutex<U8RingBuffer> = Mutex::new(U8RingBuffer::new());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    pub scancode: u8,
    pub key: Option<DecodedKey>,
    pub pressed: bool,
}

static mut SHIFT_PRESSED: bool = false;
static mut CTRL_PRESSED: bool = false;
static mut ALT_PRESSED: bool = false;
static mut CAPS_LOCK: bool = false;

pub fn init() {
    unsafe {
        SHIFT_PRESSED = false;
        CTRL_PRESSED = false;
        ALT_PRESSED = false;
        CAPS_LOCK = false;
    }
}

pub fn handle_scancode(scancode: u8) {
    let mut keyboard = KEYBOARD.lock();
    SCANCODE_BUFFER.lock().push(scancode);

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => {
                    // Only enqueue; shell_loop reads and echoes
                    KEY_BUFFER.lock().push(character);
                }
                DecodedKey::RawKey(raw_key) => {
                    handle_raw_key(raw_key);
                }
            }
        }
    }
}

fn handle_raw_key(key: pc_keyboard::KeyCode) {
    use pc_keyboard::KeyCode;

    match key {
        KeyCode::LShift | KeyCode::RShift => {
            unsafe { SHIFT_PRESSED = true; }
        }
        KeyCode::LControl | KeyCode::RControl => {
            unsafe { CTRL_PRESSED = true; }
        }
        KeyCode::LAlt => {
            unsafe { ALT_PRESSED = true; }
        }
        KeyCode::CapsLock => {
            unsafe { CAPS_LOCK = !CAPS_LOCK; }
        }
        KeyCode::ArrowUp => {
            crate::kernel::sys::handle_special_key(SpecialKey::ArrowUp);
        }
        KeyCode::ArrowDown => {
            crate::kernel::sys::handle_special_key(SpecialKey::ArrowDown);
        }
        KeyCode::ArrowLeft => {
            crate::kernel::sys::handle_special_key(SpecialKey::ArrowLeft);
        }
        KeyCode::ArrowRight => {
            crate::kernel::sys::handle_special_key(SpecialKey::ArrowRight);
        }
        KeyCode::Home => {
            crate::kernel::sys::handle_special_key(SpecialKey::Home);
        }
        KeyCode::End => {
            crate::kernel::sys::handle_special_key(SpecialKey::End);
        }
        KeyCode::PageUp => {
            crate::kernel::sys::handle_special_key(SpecialKey::PageUp);
        }
        KeyCode::PageDown => {
            crate::kernel::sys::handle_special_key(SpecialKey::PageDown);
        }
        KeyCode::Delete => {
            crate::kernel::sys::handle_special_key(SpecialKey::Delete);
        }
        KeyCode::Insert => {
            crate::kernel::sys::handle_special_key(SpecialKey::Insert);
        }
        KeyCode::F1 => crate::kernel::sys::handle_special_key(SpecialKey::F1),
        KeyCode::F2 => crate::kernel::sys::handle_special_key(SpecialKey::F2),
        KeyCode::F3 => crate::kernel::sys::handle_special_key(SpecialKey::F3),
        KeyCode::F4 => crate::kernel::sys::handle_special_key(SpecialKey::F4),
        KeyCode::F5 => crate::kernel::sys::handle_special_key(SpecialKey::F5),
        KeyCode::F6 => crate::kernel::sys::handle_special_key(SpecialKey::F6),
        KeyCode::F7 => crate::kernel::sys::handle_special_key(SpecialKey::F7),
        KeyCode::F8 => crate::kernel::sys::handle_special_key(SpecialKey::F8),
        KeyCode::F9 => crate::kernel::sys::handle_special_key(SpecialKey::F9),
        KeyCode::F10 => crate::kernel::sys::handle_special_key(SpecialKey::F10),
        KeyCode::F11 => crate::kernel::sys::handle_special_key(SpecialKey::F11),
        KeyCode::F12 => crate::kernel::sys::handle_special_key(SpecialKey::F12),
        _ => {}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialKey {
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    PageUp,
    PageDown,
    Delete,
    Insert,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

pub fn read_char() -> Option<char> {
    KEY_BUFFER.lock().pop()
}

pub fn read_char_blocking() -> char {
    loop {
        if let Some(c) = read_char() {
            return c;
        }
        x86_64::instructions::hlt();
    }
}

pub fn read_scancode() -> Option<u8> {
    SCANCODE_BUFFER.lock().pop()
}

pub fn is_shift_pressed() -> bool {
    unsafe { SHIFT_PRESSED }
}

pub fn is_ctrl_pressed() -> bool {
    unsafe { CTRL_PRESSED }
}

pub fn is_alt_pressed() -> bool {
    unsafe { ALT_PRESSED }
}

pub fn is_caps_lock_on() -> bool {
    unsafe { CAPS_LOCK }
}

pub fn clear_buffer() {
    KEY_BUFFER.lock().clear();
    SCANCODE_BUFFER.lock().clear();
}

pub fn buffer_len() -> usize {
    KEY_BUFFER.lock().len()
}

pub fn peek_char() -> Option<char> {
    KEY_BUFFER.lock().front()
}
