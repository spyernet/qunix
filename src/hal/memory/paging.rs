use x86_64::{
    structures::paging::{
        PageTable,
        OffsetPageTable,
        PhysFrame,
        Mapper,
        Page,
        PageTableFlags,
        Size4KiB,
        FrameAllocator,
    },
    structures::paging::mapper::{MapToError, UnmapError},
    VirtAddr,
    PhysAddr,
    registers::control::Cr3,
};

use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref PAGE_TABLE_MAPPER: Mutex<Option<OffsetPageTable<'static>>> = Mutex::new(None);
    static ref PHYS_MEM_OFFSET: Mutex<Option<VirtAddr>> = Mutex::new(None);
}

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    *PHYS_MEM_OFFSET.lock() = Some(physical_memory_offset);
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    &mut *page_table_ptr
}

pub fn translate_addr(addr: VirtAddr) -> Option<PhysAddr> {
    let phys_mem_offset = PHYS_MEM_OFFSET.lock().expect("Paging not initialized");
    translate_addr_inner(addr, phys_mem_offset)
}

fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    let (level_4_table_frame, _) = Cr3::read();
    let table_indexes = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];
    let mut frame = level_4_table_frame;

    for &index in &table_indexes {
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &*table_ptr };
        let entry = &table[index];

        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(_) => return None,
        };
    }

    Some(frame.start_address() + u64::from(addr.page_offset()))
}

pub fn map_page(
    page: Page<Size4KiB>,
    frame: PhysFrame<Size4KiB>,
    flags: PageTableFlags,
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    unsafe {
        mapper.map_to(page, frame, flags, frame_allocator)?.flush();
    }
    Ok(())
}

pub fn unmap_page(
    page: Page<Size4KiB>,
    mapper: &mut impl Mapper<Size4KiB>,
) -> Result<PhysFrame<Size4KiB>, UnmapError> {
    let (frame, flush) = mapper.unmap(page)?;
    flush.flush();
    Ok(frame)
}

pub fn create_mapping(
    virt_addr: VirtAddr,
    phys_addr: PhysAddr,
    size: u64,
    flags: PageTableFlags,
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let start_page = Page::containing_address(virt_addr);
    let end_page = Page::containing_address(virt_addr + size - 1u64);
    let page_range = Page::range_inclusive(start_page, end_page);

    let start_frame = PhysFrame::<Size4KiB>::containing_address(phys_addr);

    for (i, page) in page_range.enumerate() {
        let frame = PhysFrame::containing_address(
            start_frame.start_address() + (i as u64 * 4096)
        );
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    Ok(())
}

pub fn identity_map(
    frame: PhysFrame<Size4KiB>,
    flags: PageTableFlags,
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page = Page::containing_address(VirtAddr::new(frame.start_address().as_u64()));
    unsafe {
        mapper.map_to(page, frame, flags, frame_allocator)?.flush();
    }
    Ok(())
}

pub fn get_physical_memory_offset() -> Option<VirtAddr> {
    *PHYS_MEM_OFFSET.lock()
}

pub fn phys_to_virt(phys_addr: PhysAddr) -> Option<VirtAddr> {
    PHYS_MEM_OFFSET.lock().map(|offset| offset + phys_addr.as_u64())
}

pub fn flush_tlb() {
    let (frame, flags) = Cr3::read();
    unsafe {
        Cr3::write(frame, flags);
    }
}

pub fn flush_tlb_page(addr: VirtAddr) {
    x86_64::instructions::tlb::flush(addr);
}
