use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use lazy_static::lazy_static;
use crate::{println, serial_println};
use super::gdt;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        
        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.debug.set_handler_fn(debug_handler);
        idt.non_maskable_interrupt.set_handler_fn(nmi_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.overflow.set_handler_fn(overflow_handler);
        idt.bound_range_exceeded.set_handler_fn(bound_range_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.device_not_available.set_handler_fn(device_not_available_handler);
        
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        
        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.segment_not_present.set_handler_fn(segment_not_present_handler);
        idt.stack_segment_fault.set_handler_fn(stack_segment_handler);
        
        unsafe {
            idt.general_protection_fault
                .set_handler_fn(general_protection_handler)
                .set_stack_index(gdt::GENERAL_PROTECTION_IST_INDEX);
        }
        
        unsafe {
            idt.page_fault
                .set_handler_fn(page_fault_handler)
                .set_stack_index(gdt::PAGE_FAULT_IST_INDEX);
        }
        
        idt.x87_floating_point.set_handler_fn(x87_fp_handler);
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        idt.machine_check.set_handler_fn(machine_check_handler);
        idt.simd_floating_point.set_handler_fn(simd_fp_handler);
        idt.virtualization.set_handler_fn(virtualization_handler);
        idt.security_exception.set_handler_fn(security_exception_handler);
        
        idt[super::interrupts::InterruptIndex::Timer.as_usize()]
            .set_handler_fn(super::interrupts::timer_interrupt_handler);
        idt[super::interrupts::InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(super::interrupts::keyboard_interrupt_handler);
        idt[super::interrupts::InterruptIndex::PrimaryAta.as_usize()]
            .set_handler_fn(super::interrupts::primary_ata_handler);
        idt[super::interrupts::InterruptIndex::SecondaryAta.as_usize()]
            .set_handler_fn(super::interrupts::secondary_ata_handler);
        
        idt[0x80].set_handler_fn(syscall_handler);
        
        idt
    };
}

pub fn init() {
    IDT.load();
}

extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: DIVIDE BY ZERO");
    println!("{:#?}", stack_frame);
    crate::hlt_loop();
}

extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: DEBUG");
    println!("{:#?}", stack_frame);
}

extern "x86-interrupt" fn nmi_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: NON-MASKABLE INTERRUPT");
    println!("{:#?}", stack_frame);
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT");
    println!("{:#?}", stack_frame);
}

extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: OVERFLOW");
    println!("{:#?}", stack_frame);
    crate::hlt_loop();
}

extern "x86-interrupt" fn bound_range_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BOUND RANGE EXCEEDED");
    println!("{:#?}", stack_frame);
    crate::hlt_loop();
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: INVALID OPCODE");
    println!("{:#?}", stack_frame);
    crate::hlt_loop();
}

extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: DEVICE NOT AVAILABLE");
    println!("{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    println!("EXCEPTION: DOUBLE FAULT (error code: {})", error_code);
    println!("{:#?}", stack_frame);
    serial_println!("DOUBLE FAULT: {:#?}", stack_frame);
    crate::hlt_loop();
}

extern "x86-interrupt" fn invalid_tss_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("EXCEPTION: INVALID TSS (error code: {})", error_code);
    println!("{:#?}", stack_frame);
    crate::hlt_loop();
}

extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("EXCEPTION: SEGMENT NOT PRESENT (error code: {})", error_code);
    println!("{:#?}", stack_frame);
    crate::hlt_loop();
}

extern "x86-interrupt" fn stack_segment_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("EXCEPTION: STACK SEGMENT FAULT (error code: {})", error_code);
    println!("{:#?}", stack_frame);
    crate::hlt_loop();
}

extern "x86-interrupt" fn general_protection_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("EXCEPTION: GENERAL PROTECTION FAULT (error code: {})", error_code);
    println!("{:#?}", stack_frame);
    serial_println!("GPF: error_code={}, {:#?}", error_code, stack_frame);
    crate::hlt_loop();
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    
    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    serial_println!("PAGE FAULT: addr={:?}, error={:?}", Cr2::read(), error_code);
    crate::hlt_loop();
}

extern "x86-interrupt" fn x87_fp_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: x87 FLOATING POINT");
    println!("{:#?}", stack_frame);
}

extern "x86-interrupt" fn alignment_check_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("EXCEPTION: ALIGNMENT CHECK (error code: {})", error_code);
    println!("{:#?}", stack_frame);
    crate::hlt_loop();
}

extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    println!("EXCEPTION: MACHINE CHECK");
    println!("{:#?}", stack_frame);
    crate::hlt_loop();
}

extern "x86-interrupt" fn simd_fp_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: SIMD FLOATING POINT");
    println!("{:#?}", stack_frame);
}

extern "x86-interrupt" fn virtualization_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: VIRTUALIZATION");
    println!("{:#?}", stack_frame);
}

extern "x86-interrupt" fn security_exception_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("EXCEPTION: SECURITY EXCEPTION (error code: {})", error_code);
    println!("{:#?}", stack_frame);
    crate::hlt_loop();
}

extern "x86-interrupt" fn syscall_handler(stack_frame: InterruptStackFrame) {
    crate::kernel::sys::handle_syscall_interrupt(&stack_frame);
}

#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
