use x86_64::{
    VirtAddr, PhysAddr,
    structures::paging::{
        PageTable, PageTableFlags, PhysFrame, Page, Size4KiB,
        page_table::PageTableEntry,
    },
    registers::control::{Cr0, Cr0Flags, Cr3, Cr4, Cr4Flags},
};
use spin::Mutex;

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SHIFT: usize = 12;
pub const KERNEL_BASE: u64 = 0xFFFF_FFFF_8000_0000;
pub const USER_SPACE_END: u64 = 0x0000_7FFF_FFFF_FFFF;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageSize {
    Size4KiB,
    Size2MiB,
    Size1GiB,
}

impl PageSize {
    pub fn size(&self) -> u64 {
        match self {
            PageSize::Size4KiB => 4 * 1024,
            PageSize::Size2MiB => 2 * 1024 * 1024,
            PageSize::Size1GiB => 1024 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    pub start: PhysAddr,
    pub end: PhysAddr,
    pub region_type: MemoryRegionType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRegionType {
    Usable,
    Reserved,
    AcpiReclaimable,
    AcpiNvs,
    BadMemory,
    Kernel,
    Bootloader,
    FrameBuffer,
}

impl MemoryRegion {
    pub fn size(&self) -> u64 {
        self.end.as_u64() - self.start.as_u64()
    }

    pub fn contains(&self, addr: PhysAddr) -> bool {
        addr >= self.start && addr < self.end
    }
}

pub fn enable_write_protect() {
    unsafe {
        Cr0::update(|flags| {
            flags.insert(Cr0Flags::WRITE_PROTECT);
        });
    }
}

pub fn disable_write_protect() {
    unsafe {
        Cr0::update(|flags| {
            flags.remove(Cr0Flags::WRITE_PROTECT);
        });
    }
}

pub fn enable_global_pages() {
    unsafe {
        Cr4::update(|flags| {
            flags.insert(Cr4Flags::PAGE_GLOBAL);
        });
    }
}

pub fn is_paging_enabled() -> bool {
    Cr0::read().contains(Cr0Flags::PAGING)
}

pub fn get_cr3() -> (PhysFrame<Size4KiB>, x86_64::registers::control::Cr3Flags) {
    Cr3::read()
}

pub fn set_cr3(frame: PhysFrame<Size4KiB>, flags: x86_64::registers::control::Cr3Flags) {
    unsafe {
        Cr3::write(frame, flags);
    }
}

pub fn is_user_address(addr: VirtAddr) -> bool {
    addr.as_u64() <= USER_SPACE_END
}

pub fn is_kernel_address(addr: VirtAddr) -> bool {
    addr.as_u64() >= KERNEL_BASE
}

pub fn align_down(addr: u64, align: u64) -> u64 {
    addr & !(align - 1)
}

pub fn align_up(addr: u64, align: u64) -> u64 {
    (addr + align - 1) & !(align - 1)
}

pub fn page_align_down(addr: u64) -> u64 {
    align_down(addr, PAGE_SIZE as u64)
}

pub fn page_align_up(addr: u64) -> u64 {
    align_up(addr, PAGE_SIZE as u64)
}

pub fn pages_needed(size: u64) -> u64 {
    (size + PAGE_SIZE as u64 - 1) / PAGE_SIZE as u64
}

pub struct ProtectionFlags(u64);

impl ProtectionFlags {
    pub const NONE: Self = Self(0);
    pub const READ: Self = Self(1 << 0);
    pub const WRITE: Self = Self(1 << 1);
    pub const EXECUTE: Self = Self(1 << 2);
    pub const USER: Self = Self(1 << 3);

    pub fn to_page_table_flags(&self) -> PageTableFlags {
        let mut flags = PageTableFlags::PRESENT;
        if self.0 & Self::WRITE.0 != 0 {
            flags |= PageTableFlags::WRITABLE;
        }
        if self.0 & Self::USER.0 != 0 {
            flags |= PageTableFlags::USER_ACCESSIBLE;
        }
        if self.0 & Self::EXECUTE.0 == 0 {
            flags |= PageTableFlags::NO_EXECUTE;
        }
        flags
    }
}

impl core::ops::BitOr for ProtectionFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

pub fn invalidate_page(addr: VirtAddr) {
    x86_64::instructions::tlb::flush(addr);
}

pub fn invalidate_all() {
    let (frame, flags) = Cr3::read();
    unsafe {
        Cr3::write(frame, flags);
    }
}
