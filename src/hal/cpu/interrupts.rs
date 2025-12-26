use x86_64::structures::idt::InterruptStackFrame;
use pic8259::ChainedPics;
use spin::Mutex;
use lazy_static::lazy_static;
use crate::{print, println};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

lazy_static! {
    pub static ref PICS: Mutex<ChainedPics> = Mutex::new(unsafe {
        ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
    });
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
    Cascade,
    Com2,
    Com1,
    Lpt2,
    FloppyDisk,
    Lpt1,
    RealTimeClock,
    Acpi,
    Available1,
    Available2,
    Mouse,
    Coprocessor,
    PrimaryAta,
    SecondaryAta,
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    crate::hal::drivers::pit::tick();
    
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
    
    crate::kernel::scheduler::schedule();
}

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    
    crate::hal::drivers::keyboard::handle_scancode(scancode);

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

pub extern "x86-interrupt" fn primary_ata_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::PrimaryAta.as_u8());
    }
}

pub extern "x86-interrupt" fn secondary_ata_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::SecondaryAta.as_u8());
    }
}

pub fn enable() {
    x86_64::instructions::interrupts::enable();
}

pub fn disable() {
    x86_64::instructions::interrupts::disable();
}

pub fn are_enabled() -> bool {
    x86_64::instructions::interrupts::are_enabled()
}

pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    x86_64::instructions::interrupts::without_interrupts(f)
}

pub fn end_of_interrupt(irq: u8) {
    unsafe {
        PICS.lock().notify_end_of_interrupt(irq);
    }
}

pub fn set_irq_mask(irq: u8, masked: bool) {
    let mut pics = PICS.lock();
    
    if irq < 8 {
        let mut port = x86_64::instructions::port::Port::<u8>::new(0x21);
        let mut mask = unsafe { port.read() };
        if masked {
            mask |= 1 << irq;
        } else {
            mask &= !(1 << irq);
        }
        unsafe { port.write(mask) };
    } else {
        let mut port = x86_64::instructions::port::Port::<u8>::new(0xA1);
        let mut mask = unsafe { port.read() };
        let irq = irq - 8;
        if masked {
            mask |= 1 << irq;
        } else {
            mask &= !(1 << irq);
        }
        unsafe { port.write(mask) };
    }
}
