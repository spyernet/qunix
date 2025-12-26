use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::sync::Arc;
use spin::RwLock;
use alloc::vec;
use crate::fs::{FileMode, FileStat, FileType, FsResult, FsError};
use crate::fs::vfs::node::{VfsNode, VfsNodeData, DirEntry, Filesystem, InodeNumber};
use crate::fs::ext4::ext4::BlockDevice;
use super::fat::{Fat32Bpb, FatTable, FAT32_EOC};
use super::dir::{Fat32DirEntry, Fat32LfnEntry, decode_long_name, DIR_ENTRY_SIZE};

pub struct Fat32Filesystem {
    bpb: Fat32Bpb,
    fat: FatTable,
    device: Arc<RwLock<dyn BlockDevice + Send + Sync>>,
    read_only: bool,
    cluster_size: u32,
}

impl Fat32Filesystem {
    pub fn mount(device: Arc<RwLock<dyn BlockDevice + Send + Sync>>, read_only: bool) -> FsResult<Self> {
        let mut bpb_buf = [0u8; 512];
        
        {
            let dev = device.read();
            dev.read_block(0, &mut bpb_buf).map_err(|_| FsError::IoError)?;
        }
        
        let bpb: Fat32Bpb = unsafe {
            core::ptr::read(bpb_buf.as_ptr() as *const Fat32Bpb)
        };
        
        if !bpb.is_valid() {
            return Err(FsError::InvalidArgument);
        }
        
        let fat_size_bytes = bpb.fat_size() as usize * bpb.bytes_per_sector as usize;
        let mut fat_data = vec![0u8; fat_size_bytes];
        
        let fat_start_sector = bpb.first_fat_sector();
        let sectors_to_read = bpb.fat_size();
        
        {
            let dev = device.read();
            for i in 0..sectors_to_read {
                let offset = i as usize * bpb.bytes_per_sector as usize;
                dev.read_block((fat_start_sector + i) as u64, &mut fat_data[offset..offset + bpb.bytes_per_sector as usize])
                    .map_err(|_| FsError::IoError)?;
            }
        }
        
        let fat = FatTable::from_data(&fat_data);
        let cluster_size = bpb.cluster_size();
        
        Ok(Fat32Filesystem {
            bpb,
            fat,
            device,
            read_only,
            cluster_size,
        })
    }
    
    fn read_cluster(&self, cluster: u32) -> FsResult<Vec<u8>> {
        let sector = self.bpb.cluster_to_sector(cluster);
        let mut data = vec![0u8; self.cluster_size as usize];
        
        let sectors_per_cluster = self.bpb.sectors_per_cluster as u32;
        let bytes_per_sector = self.bpb.bytes_per_sector as usize;
        
        let dev = self.device.read();
        for i in 0..sectors_per_cluster {
            let offset = i as usize * bytes_per_sector;
            dev.read_block((sector + i) as u64, &mut data[offset..offset + bytes_per_sector])
                .map_err(|_| FsError::IoError)?;
        }
        
        Ok(data)
    }
    
    fn read_cluster_chain(&self, start_cluster: u32) -> FsResult<Vec<u8>> {
        let chain = self.fat.get_chain(start_cluster);
        let mut data = Vec::with_capacity(chain.len() * self.cluster_size as usize);
        
        for cluster in chain {
            let cluster_data = self.read_cluster(cluster)?;
            data.extend_from_slice(&cluster_data);
        }
        
        Ok(data)
    }
    
    fn read_directory(&self, start_cluster: u32) -> FsResult<Vec<(String, Fat32DirEntry)>> {
        let data = self.read_cluster_chain(start_cluster)?;
        let mut entries = Vec::new();
        let mut lfn_entries: Vec<Fat32LfnEntry> = Vec::new();
        
        let mut i = 0;
        while i + DIR_ENTRY_SIZE <= data.len() {
            let entry: Fat32DirEntry = unsafe {
                core::ptr::read(data[i..].as_ptr() as *const Fat32DirEntry)
            };
            
            if entry.is_last() {
                break;
            }
            
            if entry.is_free() {
                i += DIR_ENTRY_SIZE;
                lfn_entries.clear();
                continue;
            }
            
            if entry.is_long_name() {
                let lfn: Fat32LfnEntry = unsafe {
                    core::ptr::read(data[i..].as_ptr() as *const Fat32LfnEntry)
                };
                lfn_entries.push(lfn);
                i += DIR_ENTRY_SIZE;
                continue;
            }
            
            if entry.is_volume_id() {
                i += DIR_ENTRY_SIZE;
                lfn_entries.clear();
                continue;
            }
            
            let name = if !lfn_entries.is_empty() {
                lfn_entries.sort_by_key(|e| e.sequence_number());
                decode_long_name(&lfn_entries)
            } else {
                entry.short_name()
            };
            
            entries.push((name, entry));
            lfn_entries.clear();
            i += DIR_ENTRY_SIZE;
        }
        
        Ok(entries)
    }
    
    fn cluster_to_inode(&self, cluster: u32) -> InodeNumber {
        cluster as u64
    }
    
    fn inode_to_cluster(&self, inode: InodeNumber) -> u32 {
        inode as u32
    }
    
    fn entry_to_vfs_node(&self, name: &str, entry: &Fat32DirEntry) -> VfsNode {
        let file_type = entry.file_type();
        let mode = if entry.is_directory() {
            FileMode::new(FileMode::S_IFDIR | 0o755)
        } else {
            FileMode::new(FileMode::S_IFREG | 0o644)
        };
        
        VfsNode {
            name: name.to_string(),
            inode: self.cluster_to_inode(entry.first_cluster()),
            mode,
            uid: 0,
            gid: 0,
            size: entry.file_size as u64,
            atime: entry.access_time(),
            mtime: entry.modification_time(),
            ctime: entry.creation_time(),
            nlink: 1,
            device: None,
            data: VfsNodeData::Regular(Vec::new()),
        }
    }
}

impl Filesystem for Fat32Filesystem {
    fn name(&self) -> &str {
        "fat32"
    }
    
    fn root(&self) -> FsResult<VfsNode> {
        let root_cluster = self.bpb.root_cluster;
        
        Ok(VfsNode {
            name: "/".to_string(),
            inode: self.cluster_to_inode(root_cluster),
            mode: FileMode::new(FileMode::S_IFDIR | 0o755),
            uid: 0,
            gid: 0,
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
            nlink: 2,
            device: None,
            data: VfsNodeData::Directory(Vec::new()),
        })
    }
    
    fn lookup(&self, parent: InodeNumber, name: &str) -> FsResult<VfsNode> {
        let parent_cluster = self.inode_to_cluster(parent);
        let entries = self.read_directory(parent_cluster)?;
        
        for (entry_name, entry) in entries {
            if entry_name.eq_ignore_ascii_case(name) {
                return Ok(self.entry_to_vfs_node(&entry_name, &entry));
            }
        }
        
        Err(FsError::NotFound)
    }
    
    fn read(&self, inode: InodeNumber, offset: u64, buf: &mut [u8]) -> FsResult<usize> {
        let cluster = self.inode_to_cluster(inode);
        let data = self.read_cluster_chain(cluster)?;
        
        if offset >= data.len() as u64 {
            return Ok(0);
        }
        
        let start = offset as usize;
        let end = core::cmp::min(start + buf.len(), data.len());
        let len = end - start;
        
        buf[..len].copy_from_slice(&data[start..end]);
        Ok(len)
    }
    
    fn write(&mut self, _inode: InodeNumber, _offset: u64, _buf: &[u8]) -> FsResult<usize> {
        if self.read_only {
            return Err(FsError::ReadOnly);
        }
        Err(FsError::NotSupported)
    }
    
    fn create(&mut self, _parent: InodeNumber, _name: &str, _mode: FileMode) -> FsResult<VfsNode> {
        if self.read_only {
            return Err(FsError::ReadOnly);
        }
        Err(FsError::NotSupported)
    }
    
    fn mkdir(&mut self, _parent: InodeNumber, _name: &str, _mode: FileMode) -> FsResult<VfsNode> {
        if self.read_only {
            return Err(FsError::ReadOnly);
        }
        Err(FsError::NotSupported)
    }
    
    fn unlink(&mut self, _parent: InodeNumber, _name: &str) -> FsResult<()> {
        if self.read_only {
            return Err(FsError::ReadOnly);
        }
        Err(FsError::NotSupported)
    }
    
    fn rmdir(&mut self, _parent: InodeNumber, _name: &str) -> FsResult<()> {
        if self.read_only {
            return Err(FsError::ReadOnly);
        }
        Err(FsError::NotSupported)
    }
    
    fn rename(&mut self, _old_parent: InodeNumber, _old_name: &str, _new_parent: InodeNumber, _new_name: &str) -> FsResult<()> {
        if self.read_only {
            return Err(FsError::ReadOnly);
        }
        Err(FsError::NotSupported)
    }
    
    fn stat(&self, inode: InodeNumber) -> FsResult<FileStat> {
        let cluster = self.inode_to_cluster(inode);
        
        Ok(FileStat {
            dev: 0,
            ino: inode,
            mode: FileMode::new(FileMode::S_IFREG | 0o644),
            nlink: 1,
            uid: 0,
            gid: 0,
            rdev: 0,
            size: 0,
            blksize: self.cluster_size as u64,
            blocks: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        })
    }
    
    fn readdir(&self, inode: InodeNumber) -> FsResult<Vec<DirEntry>> {
        let cluster = self.inode_to_cluster(inode);
        let entries = self.read_directory(cluster)?;
        
        Ok(entries.into_iter()
            .filter(|(name, _)| name != "." && name != "..")
            .map(|(name, entry)| {
                DirEntry::new(
                    name,
                    self.cluster_to_inode(entry.first_cluster()),
                    entry.file_type(),
                )
            })
            .collect())
    }
    
    fn sync(&mut self) -> FsResult<()> {
        Ok(())
    }
}
