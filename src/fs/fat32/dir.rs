use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use crate::fs::FileType;

pub const DIR_ENTRY_SIZE: usize = 32;
pub const ATTR_READ_ONLY: u8 = 0x01;
pub const ATTR_HIDDEN: u8 = 0x02;
pub const ATTR_SYSTEM: u8 = 0x04;
pub const ATTR_VOLUME_ID: u8 = 0x08;
pub const ATTR_DIRECTORY: u8 = 0x10;
pub const ATTR_ARCHIVE: u8 = 0x20;
pub const ATTR_LONG_NAME: u8 = ATTR_READ_ONLY | ATTR_HIDDEN | ATTR_SYSTEM | ATTR_VOLUME_ID;
pub const ATTR_LONG_NAME_MASK: u8 = ATTR_READ_ONLY | ATTR_HIDDEN | ATTR_SYSTEM | ATTR_VOLUME_ID | ATTR_DIRECTORY | ATTR_ARCHIVE;

pub const DIR_FREE: u8 = 0xE5;
pub const DIR_LAST: u8 = 0x00;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Fat32DirEntry {
    pub name: [u8; 11],
    pub attr: u8,
    pub nt_res: u8,
    pub crt_time_tenth: u8,
    pub crt_time: u16,
    pub crt_date: u16,
    pub lst_acc_date: u16,
    pub fst_clus_hi: u16,
    pub wrt_time: u16,
    pub wrt_date: u16,
    pub fst_clus_lo: u16,
    pub file_size: u32,
}

impl Fat32DirEntry {
    pub fn is_free(&self) -> bool {
        self.name[0] == DIR_FREE
    }
    
    pub fn is_last(&self) -> bool {
        self.name[0] == DIR_LAST
    }
    
    pub fn is_volume_id(&self) -> bool {
        (self.attr & ATTR_VOLUME_ID) != 0 && (self.attr & ATTR_LONG_NAME_MASK) != ATTR_LONG_NAME
    }
    
    pub fn is_directory(&self) -> bool {
        (self.attr & ATTR_DIRECTORY) != 0
    }
    
    pub fn is_long_name(&self) -> bool {
        (self.attr & ATTR_LONG_NAME_MASK) == ATTR_LONG_NAME
    }
    
    pub fn first_cluster(&self) -> u32 {
        ((self.fst_clus_hi as u32) << 16) | (self.fst_clus_lo as u32)
    }
    
    pub fn set_first_cluster(&mut self, cluster: u32) {
        self.fst_clus_hi = (cluster >> 16) as u16;
        self.fst_clus_lo = (cluster & 0xFFFF) as u16;
    }
    
    pub fn short_name(&self) -> String {
        let name_part: String = self.name[..8]
            .iter()
            .take_while(|&&c| c != b' ')
            .map(|&c| c as char)
            .collect();
        
        let ext_part: String = self.name[8..11]
            .iter()
            .take_while(|&&c| c != b' ')
            .map(|&c| c as char)
            .collect();
        
        if ext_part.is_empty() {
            name_part
        } else {
            format!("{}.{}", name_part, ext_part)
        }
    }
    
    pub fn file_type(&self) -> FileType {
        if self.is_directory() {
            FileType::Directory
        } else {
            FileType::Regular
        }
    }
    
    pub fn creation_time(&self) -> u64 {
        fat_time_to_unix(self.crt_date, self.crt_time)
    }
    
    pub fn modification_time(&self) -> u64 {
        fat_time_to_unix(self.wrt_date, self.wrt_time)
    }
    
    pub fn access_time(&self) -> u64 {
        fat_time_to_unix(self.lst_acc_date, 0)
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Fat32LfnEntry {
    pub ord: u8,
    pub name1: [u16; 5],
    pub attr: u8,
    pub entry_type: u8,
    pub checksum: u8,
    pub name2: [u16; 6],
    pub fst_clus_lo: u16,
    pub name3: [u16; 2],
}

impl Fat32LfnEntry {
    pub const LAST_LONG_ENTRY: u8 = 0x40;
    
    pub fn is_last(&self) -> bool {
        (self.ord & Self::LAST_LONG_ENTRY) != 0
    }
    
    pub fn sequence_number(&self) -> u8 {
        self.ord & 0x3F
    }
    
    pub fn get_name_chars(&self) -> [u16; 13] {
        let mut chars = [0u16; 13];

        unsafe {
            // Build a dummy instance as raw bytes
            let mut tmp = core::mem::MaybeUninit::<Self>::uninit();
            let base = tmp.as_mut_ptr() as *mut u8;

            // Size of the struct
            let size = core::mem::size_of::<Self>();

            let mut name1_offset = 0usize;
            let mut name2_offset = 0usize;
            let mut name3_offset = 0usize;

            // We discover offsets by scanning for the three arrays' patterns.
            // This avoids touching the packed fields through references.
            for i in 0..size {
                let field = base.add(i) as *mut u16;

                // name1 is [u16;5]
                if name1_offset == 0 && i + 5 * 2 <= size {
                    name1_offset = i;
                }

                // name2 is [u16;6]
                if name2_offset == 0 && i + 6 * 2 <= size {
                    name2_offset = i;
                }

                // name3 is [u16;2]
                if name3_offset == 0 && i + 2 * 2 <= size {
                    name3_offset = i;
                }

                // Once all found, bail early
                if name1_offset != 0 && name2_offset != 0 && name3_offset != 0 {
                    break;
                }
            }

            // Real pointer to actual self
            let real_base = self as *const _ as *const u8;

            core::ptr::copy_nonoverlapping(
                real_base.add(name1_offset) as *const u16,
                chars.as_mut_ptr(),
                5,
            );

            core::ptr::copy_nonoverlapping(
                real_base.add(name2_offset) as *const u16,
                chars.as_mut_ptr().add(5),
                6,
            );

            core::ptr::copy_nonoverlapping(
                real_base.add(name3_offset) as *const u16,
                chars.as_mut_ptr().add(11),
                2,
            );
        }
        chars
    }
}

pub fn compute_sfn_checksum(name: &[u8; 11]) -> u8 {
    let mut sum: u8 = 0;
    for &byte in name.iter() {
        sum = sum.rotate_right(1).wrapping_add(byte);
    }
    sum
}

pub fn decode_long_name(entries: &[Fat32LfnEntry]) -> String {
    let mut chars: Vec<u16> = Vec::new();
    
    for entry in entries.iter().rev() {
        for &c in entry.get_name_chars().iter() {
            if c == 0x0000 || c == 0xFFFF {
                break;
            }
            chars.push(c);
        }
    }
    
    String::from_utf16_lossy(&chars)
}

pub fn encode_short_name(name: &str) -> [u8; 11] {
    let mut result = [b' '; 11];
    let name = name.to_uppercase();
    
    let parts: Vec<&str> = name.splitn(2, '.').collect();
    
    let base = parts[0];
    for (i, c) in base.chars().take(8).enumerate() {
        result[i] = c as u8;
    }
    
    if parts.len() > 1 {
        let ext = parts[1];
        for (i, c) in ext.chars().take(3).enumerate() {
            result[8 + i] = c as u8;
        }
    }
    
    result
}

pub fn is_valid_short_name(name: &str) -> bool {
    if name.len() > 12 {
        return false;
    }
    
    let parts: Vec<&str> = name.splitn(2, '.').collect();
    
    if parts[0].len() > 8 {
        return false;
    }
    
    if parts.len() > 1 && parts[1].len() > 3 {
        return false;
    }
    
    for c in name.chars() {
        if c == '.' {
            continue;
        }
        if !c.is_ascii_alphanumeric() && !matches!(c, '!' | '#' | '$' | '%' | '&' | '\'' | '(' | ')' | '-' | '@' | '^' | '_' | '`' | '{' | '}' | '~') {
            return false;
        }
    }
    
    true
}

fn fat_time_to_unix(date: u16, time: u16) -> u64 {
    let year = ((date >> 9) & 0x7F) as u64 + 1980;
    let month = ((date >> 5) & 0x0F) as u64;
    let day = (date & 0x1F) as u64;
    
    let hour = ((time >> 11) & 0x1F) as u64;
    let min = ((time >> 5) & 0x3F) as u64;
    let sec = ((time & 0x1F) * 2) as u64;
    
    let mut days = 0u64;
    for y in 1970..year {
        days += if is_leap_year(y) { 366 } else { 365 };
    }
    
    let days_in_month = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for m in 1..month {
        days += days_in_month[m as usize] as u64;
        if m == 2 && is_leap_year(year) {
            days += 1;
        }
    }
    
    days += day - 1;
    
    days * 86400 + hour * 3600 + min * 60 + sec
}

fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}
