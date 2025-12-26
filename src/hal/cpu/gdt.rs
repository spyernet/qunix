use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use lazy_static::lazy_static;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const PAGE_FAULT_IST_INDEX: u16 = 1;
pub const GENERAL_PROTECTION_IST_INDEX: u16 = 2;

const STACK_SIZE: usize = 4096 * 5;

#[repr(C, align(16))]
struct Stack([u8; STACK_SIZE]);

static mut DOUBLE_FAULT_STACK: Stack = Stack([0; STACK_SIZE]);
static mut PAGE_FAULT_STACK: Stack = Stack([0; STACK_SIZE]);
static mut GP_FAULT_STACK: Stack = Stack([0; STACK_SIZE]);
static mut PRIVILEGE_STACK: Stack = Stack([0; STACK_SIZE]);

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            let stack_start = VirtAddr::from_ptr(unsafe { &DOUBLE_FAULT_STACK });
            stack_start + STACK_SIZE
        };
        
        tss.interrupt_stack_table[PAGE_FAULT_IST_INDEX as usize] = {
            let stack_start = VirtAddr::from_ptr(unsafe { &PAGE_FAULT_STACK });
            stack_start + STACK_SIZE
        };
        
        tss.interrupt_stack_table[GENERAL_PROTECTION_IST_INDEX as usize] = {
            let stack_start = VirtAddr::from_ptr(unsafe { &GP_FAULT_STACK });
            stack_start + STACK_SIZE
        };
        
        tss.privilege_stack_table[0] = {
            let stack_start = VirtAddr::from_ptr(unsafe { &PRIVILEGE_STACK });
            stack_start + STACK_SIZE
        };
        
        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        
        let kernel_code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let kernel_data_selector = gdt.add_entry(Descriptor::kernel_data_segment());
        let user_data_selector = gdt.add_entry(Descriptor::user_data_segment());
        let user_code_selector = gdt.add_entry(Descriptor::user_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        
        (gdt, Selectors {
            kernel_code_selector,
            kernel_data_selector,
            user_code_selector,
            user_data_selector,
            tss_selector,
        })
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Selectors {
    pub kernel_code_selector: SegmentSelector,
    pub kernel_data_selector: SegmentSelector,
    pub user_code_selector: SegmentSelector,
    pub user_data_selector: SegmentSelector,
    pub tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, DS, ES, SS, Segment};
    
    GDT.0.load();
    
    unsafe {
        CS::set_reg(GDT.1.kernel_code_selector);
        DS::set_reg(GDT.1.kernel_data_selector);
        ES::set_reg(GDT.1.kernel_data_selector);
        SS::set_reg(GDT.1.kernel_data_selector);
        load_tss(GDT.1.tss_selector);
    }
}

pub fn get_selectors() -> Selectors {
    GDT.1
}

pub fn get_kernel_code_selector() -> SegmentSelector {
    GDT.1.kernel_code_selector
}

pub fn get_kernel_data_selector() -> SegmentSelector {
    GDT.1.kernel_data_selector
}

pub fn get_user_code_selector() -> SegmentSelector {
    GDT.1.user_code_selector
}

pub fn get_user_data_selector() -> SegmentSelector {
    GDT.1.user_data_selector
}
