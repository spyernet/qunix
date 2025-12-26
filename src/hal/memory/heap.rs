use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};
use linked_list_allocator::LockedHeap;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 8 * 1024 * 1024;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE);
    }

    Ok(())
}

pub fn heap_used() -> usize {
    ALLOCATOR.lock().used()
}

pub fn heap_free() -> usize {
    ALLOCATOR.lock().free()
}

pub fn heap_size() -> usize {
    HEAP_SIZE
}

#[derive(Debug, Clone, Copy)]
pub struct HeapStats {
    pub total: usize,
    pub used: usize,
    pub free: usize,
}

pub fn get_heap_stats() -> HeapStats {
    let allocator = ALLOCATOR.lock();
    HeapStats {
        total: HEAP_SIZE,
        used: allocator.used(),
        free: allocator.free(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::boxed::Box;
    use alloc::vec::Vec;

    #[test_case]
    fn simple_allocation() {
        let heap_value_1 = Box::new(41);
        let heap_value_2 = Box::new(13);
        assert_eq!(*heap_value_1, 41);
        assert_eq!(*heap_value_2, 13);
    }

    #[test_case]
    fn large_vec() {
        let n = 1000;
        let mut vec = Vec::new();
        for i in 0..n {
            vec.push(i);
        }
        assert_eq!(vec.iter().sum::<u32>(), (n - 1) as u32 * n as u32 / 2);
    }

    #[test_case]
    fn many_boxes() {
        for i in 0..10_000 {
            let x = Box::new(i);
            assert_eq!(*x, i);
        }
    }

    #[test_case]
    fn many_boxes_long_lived() {
        let long_lived = Box::new(1);
        for i in 0..10_000 {
            let x = Box::new(i);
            assert_eq!(*x, i);
        }
        assert_eq!(*long_lived, 1);
    }
}
