use alloc::vec::Vec;
use crate::fs::{FileMode, FileType};

pub const EXT4_GOOD_OLD_INODE_SIZE: u16 = 128;
pub const EXT4_ROOT_INO: u32 = 2;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Ext4Inode {
    pub i_mode: u16,
    pub i_uid: u16,
    pub i_size_lo: u32,
    pub i_atime: u32,
    pub i_ctime: u32,
    pub i_mtime: u32,
    pub i_dtime: u32,
    pub i_gid: u16,
    pub i_links_count: u16,
    pub i_blocks_lo: u32,
    pub i_flags: u32,
    pub i_osd1: u32,
    pub i_block: [u32; 15],
    pub i_generation: u32,
    pub i_file_acl_lo: u32,
    pub i_size_high: u32,
    pub i_obso_faddr: u32,
    pub i_osd2: [u8; 12],
    pub i_extra_isize: u16,
    pub i_checksum_hi: u16,
    pub i_ctime_extra: u32,
    pub i_mtime_extra: u32,
    pub i_atime_extra: u32,
    pub i_crtime: u32,
    pub i_crtime_extra: u32,
    pub i_version_hi: u32,
    pub i_projid: u32,
}

impl Ext4Inode {
    pub fn size(&self) -> u64 {
        (self.i_size_high as u64) << 32 | self.i_size_lo as u64
    }
    
    pub fn set_size(&mut self, size: u64) {
        self.i_size_lo = (size & 0xFFFFFFFF) as u32;
        self.i_size_high = (size >> 32) as u32;
    }
    
    pub fn file_type(&self) -> FileType {
        let mode = FileMode::new(self.i_mode);
        mode.file_type()
    }
    
    pub fn is_dir(&self) -> bool {
        (self.i_mode & 0xF000) == 0x4000
    }
    
    pub fn is_file(&self) -> bool {
        (self.i_mode & 0xF000) == 0x8000
    }
    
    pub fn is_symlink(&self) -> bool {
        (self.i_mode & 0xF000) == 0xA000
    }
    
    pub fn uses_extents(&self) -> bool {
        self.i_flags & EXT4_EXTENTS_FL != 0
    }
    
    pub fn blocks_count(&self) -> u64 {
        let lo = self.i_blocks_lo as u64;
        let hi = ((self.i_osd2[1] as u64) << 8) | (self.i_osd2[0] as u64);
        
        if self.i_flags & EXT4_HUGE_FILE_FL != 0 {
            (hi << 32 | lo) << 9
        } else {
            hi << 32 | lo
        }
    }
    
    pub fn uid(&self) -> u32 {
        let hi = ((self.i_osd2[5] as u32) << 8) | (self.i_osd2[4] as u32);
        (hi << 16) | self.i_uid as u32
    }
    
    pub fn gid(&self) -> u32 {
        let hi = ((self.i_osd2[7] as u32) << 8) | (self.i_osd2[6] as u32);
        (hi << 16) | self.i_gid as u32
    }
}

pub const EXT4_SECRM_FL: u32 = 0x00000001;
pub const EXT4_UNRM_FL: u32 = 0x00000002;
pub const EXT4_COMPR_FL: u32 = 0x00000004;
pub const EXT4_SYNC_FL: u32 = 0x00000008;
pub const EXT4_IMMUTABLE_FL: u32 = 0x00000010;
pub const EXT4_APPEND_FL: u32 = 0x00000020;
pub const EXT4_NODUMP_FL: u32 = 0x00000040;
pub const EXT4_NOATIME_FL: u32 = 0x00000080;
pub const EXT4_DIRTY_FL: u32 = 0x00000100;
pub const EXT4_COMPRBLK_FL: u32 = 0x00000200;
pub const EXT4_NOCOMPR_FL: u32 = 0x00000400;
pub const EXT4_ENCRYPT_FL: u32 = 0x00000800;
pub const EXT4_INDEX_FL: u32 = 0x00001000;
pub const EXT4_IMAGIC_FL: u32 = 0x00002000;
pub const EXT4_JOURNAL_DATA_FL: u32 = 0x00004000;
pub const EXT4_NOTAIL_FL: u32 = 0x00008000;
pub const EXT4_DIRSYNC_FL: u32 = 0x00010000;
pub const EXT4_TOPDIR_FL: u32 = 0x00020000;
pub const EXT4_HUGE_FILE_FL: u32 = 0x00040000;
pub const EXT4_EXTENTS_FL: u32 = 0x00080000;
pub const EXT4_VERITY_FL: u32 = 0x00100000;
pub const EXT4_EA_INODE_FL: u32 = 0x00200000;
pub const EXT4_INLINE_DATA_FL: u32 = 0x10000000;
pub const EXT4_PROJINHERIT_FL: u32 = 0x20000000;
pub const EXT4_CASEFOLD_FL: u32 = 0x40000000;
pub const EXT4_RESERVED_FL: u32 = 0x80000000;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Ext4ExtentHeader {
    pub eh_magic: u16,
    pub eh_entries: u16,
    pub eh_max: u16,
    pub eh_depth: u16,
    pub eh_generation: u32,
}

impl Ext4ExtentHeader {
    pub const MAGIC: u16 = 0xF30A;
    
    pub fn is_valid(&self) -> bool {
        self.eh_magic == Self::MAGIC
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Ext4ExtentIdx {
    pub ei_block: u32,
    pub ei_leaf_lo: u32,
    pub ei_leaf_hi: u16,
    pub ei_unused: u16,
}

impl Ext4ExtentIdx {
    pub fn leaf(&self) -> u64 {
        (self.ei_leaf_hi as u64) << 32 | self.ei_leaf_lo as u64
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Ext4Extent {
    pub ee_block: u32,
    pub ee_len: u16,
    pub ee_start_hi: u16,
    pub ee_start_lo: u32,
}

impl Ext4Extent {
    pub fn start(&self) -> u64 {
        (self.ee_start_hi as u64) << 32 | self.ee_start_lo as u64
    }
    
    pub fn len(&self) -> u32 {
        if self.ee_len > 32768 {
            (self.ee_len - 32768) as u32
        } else {
            self.ee_len as u32
        }
    }
    
    pub fn is_unwritten(&self) -> bool {
        self.ee_len > 32768
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Ext4DirEntry {
    pub inode: u32,
    pub rec_len: u16,
    pub name_len: u8,
    pub file_type: u8,
}

impl Ext4DirEntry {
    pub fn file_type(&self) -> FileType {
        match self.file_type {
            1 => FileType::Regular,
            2 => FileType::Directory,
            3 => FileType::CharDevice,
            4 => FileType::BlockDevice,
            5 => FileType::Fifo,
            6 => FileType::Socket,
            7 => FileType::Symlink,
            _ => FileType::Regular,
        }
    }
}

pub const EXT4_FT_UNKNOWN: u8 = 0;
pub const EXT4_FT_REG_FILE: u8 = 1;
pub const EXT4_FT_DIR: u8 = 2;
pub const EXT4_FT_CHRDEV: u8 = 3;
pub const EXT4_FT_BLKDEV: u8 = 4;
pub const EXT4_FT_FIFO: u8 = 5;
pub const EXT4_FT_SOCK: u8 = 6;
pub const EXT4_FT_SYMLINK: u8 = 7;

pub struct InodeTable {
    inodes: alloc::collections::BTreeMap<u32, Ext4Inode>,
}

impl InodeTable {
    pub fn new() -> Self {
        InodeTable {
            inodes: alloc::collections::BTreeMap::new(),
        }
    }
    
    pub fn get(&self, inode_num: u32) -> Option<&Ext4Inode> {
        self.inodes.get(&inode_num)
    }
    
    pub fn insert(&mut self, inode_num: u32, inode: Ext4Inode) {
        self.inodes.insert(inode_num, inode);
    }
    
    pub fn remove(&mut self, inode_num: u32) -> Option<Ext4Inode> {
        self.inodes.remove(&inode_num)
    }
}
