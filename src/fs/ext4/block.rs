use alloc::vec::Vec;

pub const EXT4_SUPER_MAGIC: u16 = 0xEF53;
pub const EXT4_BLOCK_SIZE_MIN: u32 = 1024;
pub const EXT4_BLOCK_SIZE_MAX: u32 = 65536;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Ext4Superblock {
    pub s_inodes_count: u32,
    pub s_blocks_count_lo: u32,
    pub s_r_blocks_count_lo: u32,
    pub s_free_blocks_count_lo: u32,
    pub s_free_inodes_count: u32,
    pub s_first_data_block: u32,
    pub s_log_block_size: u32,
    pub s_log_cluster_size: u32,
    pub s_blocks_per_group: u32,
    pub s_clusters_per_group: u32,
    pub s_inodes_per_group: u32,
    pub s_mtime: u32,
    pub s_wtime: u32,
    pub s_mnt_count: u16,
    pub s_max_mnt_count: u16,
    pub s_magic: u16,
    pub s_state: u16,
    pub s_errors: u16,
    pub s_minor_rev_level: u16,
    pub s_lastcheck: u32,
    pub s_checkinterval: u32,
    pub s_creator_os: u32,
    pub s_rev_level: u32,
    pub s_def_resuid: u16,
    pub s_def_resgid: u16,
    
    pub s_first_ino: u32,
    pub s_inode_size: u16,
    pub s_block_group_nr: u16,
    pub s_feature_compat: u32,
    pub s_feature_incompat: u32,
    pub s_feature_ro_compat: u32,
    pub s_uuid: [u8; 16],
    pub s_volume_name: [u8; 16],
    pub s_last_mounted: [u8; 64],
    pub s_algorithm_usage_bitmap: u32,
    
    pub s_prealloc_blocks: u8,
    pub s_prealloc_dir_blocks: u8,
    pub s_reserved_gdt_blocks: u16,
    
    pub s_journal_uuid: [u8; 16],
    pub s_journal_inum: u32,
    pub s_journal_dev: u32,
    pub s_last_orphan: u32,
    pub s_hash_seed: [u32; 4],
    pub s_def_hash_version: u8,
    pub s_jnl_backup_type: u8,
    pub s_desc_size: u16,
    pub s_default_mount_opts: u32,
    pub s_first_meta_bg: u32,
    pub s_mkfs_time: u32,
    pub s_jnl_blocks: [u32; 17],
    
    pub s_blocks_count_hi: u32,
    pub s_r_blocks_count_hi: u32,
    pub s_free_blocks_count_hi: u32,
    pub s_min_extra_isize: u16,
    pub s_want_extra_isize: u16,
    pub s_flags: u32,
    pub s_raid_stride: u16,
    pub s_mmp_interval: u16,
    pub s_mmp_block: u64,
    pub s_raid_stripe_width: u32,
    pub s_log_groups_per_flex: u8,
    pub s_checksum_type: u8,
    pub s_reserved_pad: u16,
    pub s_kbytes_written: u64,
    pub s_snapshot_inum: u32,
    pub s_snapshot_id: u32,
    pub s_snapshot_r_blocks_count: u64,
    pub s_snapshot_list: u32,
    pub s_error_count: u32,
    pub s_first_error_time: u32,
    pub s_first_error_ino: u32,
    pub s_first_error_block: u64,
    pub s_first_error_func: [u8; 32],
    pub s_first_error_line: u32,
    pub s_last_error_time: u32,
    pub s_last_error_ino: u32,
    pub s_last_error_line: u32,
    pub s_last_error_block: u64,
    pub s_last_error_func: [u8; 32],
    pub s_mount_opts: [u8; 64],
    pub s_usr_quota_inum: u32,
    pub s_grp_quota_inum: u32,
    pub s_overhead_blocks: u32,
    pub s_backup_bgs: [u32; 2],
    pub s_encrypt_algos: [u8; 4],
    pub s_encrypt_pw_salt: [u8; 16],
    pub s_lpf_ino: u32,
    pub s_prj_quota_inum: u32,
    pub s_checksum_seed: u32,
    pub s_reserved: [u32; 98],
    pub s_checksum: u32,
}

impl Ext4Superblock {
    pub fn is_valid(&self) -> bool {
        self.s_magic == EXT4_SUPER_MAGIC
    }
    
    pub fn block_size(&self) -> u32 {
        EXT4_BLOCK_SIZE_MIN << self.s_log_block_size
    }
    
    pub fn blocks_count(&self) -> u64 {
        (self.s_blocks_count_hi as u64) << 32 | self.s_blocks_count_lo as u64
    }
    
    pub fn free_blocks_count(&self) -> u64 {
        (self.s_free_blocks_count_hi as u64) << 32 | self.s_free_blocks_count_lo as u64
    }
    
    pub fn block_group_count(&self) -> u32 {
        let blocks = self.blocks_count();
        let blocks_per_group = self.s_blocks_per_group as u64;
        ((blocks + blocks_per_group - 1) / blocks_per_group) as u32
    }
    
    pub fn inode_size(&self) -> u16 {
        if self.s_rev_level == 0 {
            128
        } else {
            self.s_inode_size
        }
    }
    
    pub fn has_feature_compat(&self, feature: u32) -> bool {
        self.s_feature_compat & feature != 0
    }
    
    pub fn has_feature_incompat(&self, feature: u32) -> bool {
        self.s_feature_incompat & feature != 0
    }
    
    pub fn has_feature_ro_compat(&self, feature: u32) -> bool {
        self.s_feature_ro_compat & feature != 0
    }
}

pub const EXT4_FEATURE_COMPAT_DIR_PREALLOC: u32 = 0x0001;
pub const EXT4_FEATURE_COMPAT_IMAGIC_INODES: u32 = 0x0002;
pub const EXT4_FEATURE_COMPAT_HAS_JOURNAL: u32 = 0x0004;
pub const EXT4_FEATURE_COMPAT_EXT_ATTR: u32 = 0x0008;
pub const EXT4_FEATURE_COMPAT_RESIZE_INODE: u32 = 0x0010;
pub const EXT4_FEATURE_COMPAT_DIR_INDEX: u32 = 0x0020;
pub const EXT4_FEATURE_COMPAT_SPARSE_SUPER2: u32 = 0x0200;

pub const EXT4_FEATURE_INCOMPAT_COMPRESSION: u32 = 0x0001;
pub const EXT4_FEATURE_INCOMPAT_FILETYPE: u32 = 0x0002;
pub const EXT4_FEATURE_INCOMPAT_RECOVER: u32 = 0x0004;
pub const EXT4_FEATURE_INCOMPAT_JOURNAL_DEV: u32 = 0x0008;
pub const EXT4_FEATURE_INCOMPAT_META_BG: u32 = 0x0010;
pub const EXT4_FEATURE_INCOMPAT_EXTENTS: u32 = 0x0040;
pub const EXT4_FEATURE_INCOMPAT_64BIT: u32 = 0x0080;
pub const EXT4_FEATURE_INCOMPAT_MMP: u32 = 0x0100;
pub const EXT4_FEATURE_INCOMPAT_FLEX_BG: u32 = 0x0200;
pub const EXT4_FEATURE_INCOMPAT_EA_INODE: u32 = 0x0400;
pub const EXT4_FEATURE_INCOMPAT_DIRDATA: u32 = 0x1000;
pub const EXT4_FEATURE_INCOMPAT_CSUM_SEED: u32 = 0x2000;
pub const EXT4_FEATURE_INCOMPAT_LARGEDIR: u32 = 0x4000;
pub const EXT4_FEATURE_INCOMPAT_INLINE_DATA: u32 = 0x8000;
pub const EXT4_FEATURE_INCOMPAT_ENCRYPT: u32 = 0x10000;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Ext4BlockGroupDesc {
    pub bg_block_bitmap_lo: u32,
    pub bg_inode_bitmap_lo: u32,
    pub bg_inode_table_lo: u32,
    pub bg_free_blocks_count_lo: u16,
    pub bg_free_inodes_count_lo: u16,
    pub bg_used_dirs_count_lo: u16,
    pub bg_flags: u16,
    pub bg_exclude_bitmap_lo: u32,
    pub bg_block_bitmap_csum_lo: u16,
    pub bg_inode_bitmap_csum_lo: u16,
    pub bg_itable_unused_lo: u16,
    pub bg_checksum: u16,
    
    pub bg_block_bitmap_hi: u32,
    pub bg_inode_bitmap_hi: u32,
    pub bg_inode_table_hi: u32,
    pub bg_free_blocks_count_hi: u16,
    pub bg_free_inodes_count_hi: u16,
    pub bg_used_dirs_count_hi: u16,
    pub bg_itable_unused_hi: u16,
    pub bg_exclude_bitmap_hi: u32,
    pub bg_block_bitmap_csum_hi: u16,
    pub bg_inode_bitmap_csum_hi: u16,
    pub bg_reserved: u32,
}

impl Ext4BlockGroupDesc {
    pub fn block_bitmap(&self) -> u64 {
        (self.bg_block_bitmap_hi as u64) << 32 | self.bg_block_bitmap_lo as u64
    }
    
    pub fn inode_bitmap(&self) -> u64 {
        (self.bg_inode_bitmap_hi as u64) << 32 | self.bg_inode_bitmap_lo as u64
    }
    
    pub fn inode_table(&self) -> u64 {
        (self.bg_inode_table_hi as u64) << 32 | self.bg_inode_table_lo as u64
    }
    
    pub fn free_blocks_count(&self) -> u32 {
        (self.bg_free_blocks_count_hi as u32) << 16 | self.bg_free_blocks_count_lo as u32
    }
    
    pub fn free_inodes_count(&self) -> u32 {
        (self.bg_free_inodes_count_hi as u32) << 16 | self.bg_free_inodes_count_lo as u32
    }
}

pub struct BlockCache {
    cache: alloc::collections::BTreeMap<u64, Vec<u8>>,
    block_size: u32,
    max_entries: usize,
}

impl BlockCache {
    pub fn new(block_size: u32, max_entries: usize) -> Self {
        BlockCache {
            cache: alloc::collections::BTreeMap::new(),
            block_size,
            max_entries,
        }
    }
    
    pub fn get(&self, block_num: u64) -> Option<&Vec<u8>> {
        self.cache.get(&block_num)
    }
    
    pub fn insert(&mut self, block_num: u64, data: Vec<u8>) {
        if self.cache.len() >= self.max_entries {
            if let Some(&first_key) = self.cache.keys().next() {
                self.cache.remove(&first_key);
            }
        }
        self.cache.insert(block_num, data);
    }
    
    pub fn invalidate(&mut self, block_num: u64) {
        self.cache.remove(&block_num);
    }
    
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}
