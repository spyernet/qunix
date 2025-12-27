// clear - Clear the screen

pub fn run() {
    crate::hal::drivers::vga::clear_screen();
}
