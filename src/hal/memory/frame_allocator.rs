use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};
use x86_64::PhysAddr;
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use spin::Mutex;
use lazy_static::lazy_static;

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator { memory_map, next: 0 }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }

    pub fn total_memory(&self) -> u64 {
        self.memory_map
            .iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            .map(|r| r.range.end_addr() - r.range.start_addr())
            .sum()
    }

    pub fn used_frames(&self) -> usize {
        self.next
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

/// Global frame allocator - initialized during boot
lazy_static! {
    pub static ref FRAME_ALLOCATOR: Mutex<Option<BootInfoFrameAllocator>> = Mutex::new(None);
}

/// Initialize frame allocator from boot info memory map
/// Must be called early in boot process before heap or paging
pub fn init_from_boot_info(memory_map: &'static MemoryMap) {
    unsafe {
        let allocator = BootInfoFrameAllocator::init(memory_map);
        *FRAME_ALLOCATOR.lock() = Some(allocator);
    }
}

pub struct BitmapFrameAllocator {
    bitmap: alloc::vec::Vec<u64>,
    base_frame: u64,
    total_frames: usize,
    next_free: usize,
}

impl BitmapFrameAllocator {
    pub fn new(start_addr: PhysAddr, total_frames: usize) -> Self {
        let bitmap_size = (total_frames + 63) / 64;
        let mut bitmap = alloc::vec::Vec::with_capacity(bitmap_size);
        bitmap.resize(bitmap_size, 0);

        BitmapFrameAllocator {
            bitmap,
            base_frame: start_addr.as_u64() / 4096,
            total_frames,
            next_free: 0,
        }
    }

    pub fn mark_used(&mut self, frame: PhysFrame) {
        let frame_number = frame.start_address().as_u64() / 4096 - self.base_frame;
        if frame_number < self.total_frames as u64 {
            let index = frame_number as usize / 64;
            let bit = frame_number as usize % 64;
            self.bitmap[index] |= 1 << bit;
        }
    }

    pub fn mark_free(&mut self, frame: PhysFrame) {
        let frame_number = frame.start_address().as_u64() / 4096 - self.base_frame;
        if frame_number < self.total_frames as u64 {
            let index = frame_number as usize / 64;
            let bit = frame_number as usize % 64;
            self.bitmap[index] &= !(1 << bit);
            if frame_number < self.next_free as u64 {
                self.next_free = frame_number as usize;
            }
        }
    }

    pub fn is_used(&self, frame: PhysFrame) -> bool {
        let frame_number = frame.start_address().as_u64() / 4096 - self.base_frame;
        if frame_number < self.total_frames as u64 {
            let index = frame_number as usize / 64;
            let bit = frame_number as usize % 64;
            (self.bitmap[index] & (1 << bit)) != 0
        } else {
            true
        }
    }

    pub fn free_frames(&self) -> usize {
        let used: usize = self.bitmap.iter().map(|x| x.count_ones() as usize).sum();
        self.total_frames - used
    }

    pub fn used_frames(&self) -> usize {
        self.bitmap.iter().map(|x| x.count_ones() as usize).sum()
    }

    fn find_free_frame(&mut self) -> Option<usize> {
        for i in self.next_free..self.total_frames {
            let index = i / 64;
            let bit = i % 64;
            if (self.bitmap[index] & (1 << bit)) == 0 {
                return Some(i);
            }
        }

        for i in 0..self.next_free {
            let index = i / 64;
            let bit = i % 64;
            if (self.bitmap[index] & (1 << bit)) == 0 {
                return Some(i);
            }
        }

        None
    }
}

unsafe impl FrameAllocator<Size4KiB> for BitmapFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        if let Some(frame_idx) = self.find_free_frame() {
            let index = frame_idx / 64;
            let bit = frame_idx % 64;
            self.bitmap[index] |= 1 << bit;
            self.next_free = frame_idx + 1;

            let addr = self.base_frame + frame_idx as u64 * 4096;
            Some(PhysFrame::containing_address(PhysAddr::new(addr * 4096)))
        } else {
            None
        }
    }
}

pub trait FrameDeallocator {
    fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>);
}

impl FrameDeallocator for BitmapFrameAllocator {
    fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        self.mark_free(frame);
    }
}

pub struct ZoneAllocator {
    pub dma_zone: Option<BitmapFrameAllocator>,
    pub normal_zone: Option<BitmapFrameAllocator>,
    pub high_zone: Option<BitmapFrameAllocator>,
}

impl ZoneAllocator {
    pub const DMA_LIMIT: u64 = 16 * 1024 * 1024;
    pub const NORMAL_LIMIT: u64 = 896 * 1024 * 1024;

    pub fn new() -> Self {
        ZoneAllocator {
            dma_zone: None,
            normal_zone: None,
            high_zone: None,
        }
    }

    pub fn allocate_from_dma(&mut self) -> Option<PhysFrame> {
        self.dma_zone.as_mut()?.allocate_frame()
    }

    pub fn allocate_from_normal(&mut self) -> Option<PhysFrame> {
        self.normal_zone.as_mut()?.allocate_frame()
    }

    pub fn allocate_from_high(&mut self) -> Option<PhysFrame> {
        self.high_zone.as_mut()?.allocate_frame()
    }
}