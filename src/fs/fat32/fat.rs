use alloc::vec::Vec;
use alloc::vec;

pub const FAT32_SIGNATURE: u16 = 0xAA55;
pub const FAT32_FSTYPE: [u8; 8] = *b"FAT32   ";

pub const FAT32_EOC: u32 = 0x0FFFFFF8;
pub const FAT32_BAD: u32 = 0x0FFFFFF7;
pub const FAT32_FREE: u32 = 0x00000000;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Fat32Bpb {
    pub jmp_boot: [u8; 3],
    pub oem_name: [u8; 8],
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sector_count: u16,
    pub num_fats: u8,
    pub root_entry_count: u16,
    pub total_sectors_16: u16,
    pub media: u8,
    pub fat_size_16: u16,
    pub sectors_per_track: u16,
    pub num_heads: u16,
    pub hidden_sectors: u32,
    pub total_sectors_32: u32,
    
    pub fat_size_32: u32,
    pub ext_flags: u16,
    pub fs_version: u16,
    pub root_cluster: u32,
    pub fs_info: u16,
    pub backup_boot_sector: u16,
    pub reserved: [u8; 12],
    pub drive_number: u8,
    pub reserved1: u8,
    pub boot_sig: u8,
    pub volume_id: u32,
    pub volume_label: [u8; 11],
    pub fs_type: [u8; 8],
}

impl Fat32Bpb {
    pub fn is_valid(&self) -> bool {
        self.bytes_per_sector >= 512 &&
        self.bytes_per_sector <= 4096 &&
        self.sectors_per_cluster > 0 &&
        self.num_fats > 0
    }
    
    pub fn cluster_size(&self) -> u32 {
        self.bytes_per_sector as u32 * self.sectors_per_cluster as u32
    }
    
    pub fn fat_size(&self) -> u32 {
        if self.fat_size_16 != 0 {
            self.fat_size_16 as u32
        } else {
            self.fat_size_32
        }
    }
    
    pub fn total_sectors(&self) -> u32 {
        if self.total_sectors_16 != 0 {
            self.total_sectors_16 as u32
        } else {
            self.total_sectors_32
        }
    }
    
    pub fn root_dir_sectors(&self) -> u32 {
        ((self.root_entry_count as u32 * 32) + (self.bytes_per_sector as u32 - 1)) / self.bytes_per_sector as u32
    }
    
    pub fn first_data_sector(&self) -> u32 {
        self.reserved_sector_count as u32 +
        (self.num_fats as u32 * self.fat_size()) +
        self.root_dir_sectors()
    }
    
    pub fn first_fat_sector(&self) -> u32 {
        self.reserved_sector_count as u32
    }
    
    pub fn data_sectors(&self) -> u32 {
        self.total_sectors() - self.first_data_sector()
    }
    
    pub fn cluster_count(&self) -> u32 {
        self.data_sectors() / self.sectors_per_cluster as u32
    }
    
    pub fn cluster_to_sector(&self, cluster: u32) -> u32 {
        ((cluster - 2) * self.sectors_per_cluster as u32) + self.first_data_sector()
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Fat32FsInfo {
    pub lead_sig: u32,
    pub reserved1: [u8; 480],
    pub struc_sig: u32,
    pub free_count: u32,
    pub next_free: u32,
    pub reserved2: [u8; 12],
    pub trail_sig: u32,
}

impl Fat32FsInfo {
    pub const LEAD_SIG: u32 = 0x41615252;
    pub const STRUC_SIG: u32 = 0x61417272;
    pub const TRAIL_SIG: u32 = 0xAA550000;
    
    pub fn is_valid(&self) -> bool {
        self.lead_sig == Self::LEAD_SIG &&
        self.struc_sig == Self::STRUC_SIG &&
        self.trail_sig == Self::TRAIL_SIG
    }
}

pub struct FatTable {
    entries: Vec<u32>,
    dirty: bool,
}

impl FatTable {
    pub fn new(size: usize) -> Self {
        FatTable {
            entries: vec![0; size],
            dirty: false,
        }
    }
    
    pub fn from_data(data: &[u8]) -> Self {
        let count = data.len() / 4;
        let mut entries = Vec::with_capacity(count);
        
        for i in 0..count {
            let offset = i * 4;
            let entry = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]) & 0x0FFFFFFF;
            entries.push(entry);
        }
        
        FatTable {
            entries,
            dirty: false,
        }
    }
    
    pub fn get(&self, cluster: u32) -> u32 {
        if cluster as usize >= self.entries.len() {
            FAT32_BAD
        } else {
            self.entries[cluster as usize]
        }
    }
    
    pub fn set(&mut self, cluster: u32, value: u32) {
        if (cluster as usize) < self.entries.len() {
            self.entries[cluster as usize] = value & 0x0FFFFFFF;
            self.dirty = true;
        }
    }
    
    pub fn is_free(&self, cluster: u32) -> bool {
        self.get(cluster) == FAT32_FREE
    }
    
    pub fn is_end_of_chain(&self, cluster: u32) -> bool {
        self.get(cluster) >= FAT32_EOC
    }
    
    pub fn is_bad(&self, cluster: u32) -> bool {
        self.get(cluster) == FAT32_BAD
    }
    
    pub fn allocate_cluster(&mut self) -> Option<u32> {
        for i in 2..self.entries.len() as u32 {
            if self.is_free(i) {
                self.set(i, FAT32_EOC);
                return Some(i);
            }
        }
        None
    }
    
    pub fn free_cluster(&mut self, cluster: u32) {
        self.set(cluster, FAT32_FREE);
    }
    
    pub fn extend_chain(&mut self, last_cluster: u32) -> Option<u32> {
        if let Some(new_cluster) = self.allocate_cluster() {
            self.set(last_cluster, new_cluster);
            Some(new_cluster)
        } else {
            None
        }
    }
    
    pub fn get_chain(&self, start_cluster: u32) -> Vec<u32> {
        let mut chain = Vec::new();
        let mut current = start_cluster;
        
        while current >= 2 && current < FAT32_BAD {
            chain.push(current);
            current = self.get(current);
            
            if chain.len() > self.entries.len() {
                break;
            }
        }
        
        chain
    }
    
    pub fn free_chain(&mut self, start_cluster: u32) {
        let chain = self.get_chain(start_cluster);
        for cluster in chain {
            self.free_cluster(cluster);
        }
    }
    
    pub fn count_free(&self) -> u32 {
        self.entries.iter().filter(|&&e| e == FAT32_FREE).count() as u32
    }
    
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}
