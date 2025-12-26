use core::ptr;
use core::mem::offset_of;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::sync::Arc;
use spin::RwLock;
use alloc::vec;
use crate::fs::{FileMode, FileStat, FileType, FsResult, FsError};
use crate::fs::vfs::node::{VfsNode, DirEntry, Filesystem, InodeNumber};
use super::block::{Ext4Superblock, Ext4BlockGroupDesc, BlockCache};
use super::inode::{Ext4Inode, Ext4DirEntry, EXT4_ROOT_INO};

pub struct Ext4Filesystem {
    superblock: Ext4Superblock,
    block_groups: Vec<Ext4BlockGroupDesc>,
    block_size: u32,
    block_cache: BlockCache,
    device: Arc<RwLock<dyn BlockDevice + Send + Sync>>,
    read_only: bool,
}

pub trait BlockDevice {
    fn read_block(&self, block_num: u64, buf: &mut [u8]) -> Result<(), &'static str>;
    fn write_block(&mut self, block_num: u64, buf: &[u8]) -> Result<(), &'static str>;
    fn block_size(&self) -> u32;
    fn block_count(&self) -> u64;
}

impl Ext4Filesystem {
    pub fn mount(device: Arc<RwLock<dyn BlockDevice + Send + Sync>>, read_only: bool) -> FsResult<Self> {
        let mut superblock_buf = [0u8; 1024];
        
        {
            let dev = device.read();
            dev.read_block(1, &mut superblock_buf).map_err(|_| FsError::IoError)?;
        }
        
        let superblock: Ext4Superblock = unsafe {
            core::ptr::read(superblock_buf.as_ptr() as *const Ext4Superblock)
        };
        
        if !superblock.is_valid() {
            return Err(FsError::InvalidArgument);
        }
        
        let block_size = superblock.block_size();
        let bg_count = superblock.block_group_count();
        
        let mut block_groups = Vec::with_capacity(bg_count as usize);
        let bg_per_block = block_size / core::mem::size_of::<Ext4BlockGroupDesc>() as u32;
        
        let bg_start_block = if block_size == 1024 { 2 } else { 1 };
        
        for i in 0..bg_count {
            let block_idx = bg_start_block + (i / bg_per_block);
            let offset_in_block = (i % bg_per_block) as usize * core::mem::size_of::<Ext4BlockGroupDesc>();
            
            let mut block_buf = vec![0u8; block_size as usize];
            {
                let dev = device.read();
                dev.read_block(block_idx as u64, &mut block_buf).map_err(|_| FsError::IoError)?;
            }
            
            let bg: Ext4BlockGroupDesc = unsafe {
                core::ptr::read(block_buf[offset_in_block..].as_ptr() as *const Ext4BlockGroupDesc)
            };
            block_groups.push(bg);
        }
        
        Ok(Ext4Filesystem {
            superblock,
            block_groups,
            block_size,
            block_cache: BlockCache::new(block_size, 256),
            device,
            read_only,
        })
    }
    
    fn read_inode(&self, inode_num: u32) -> FsResult<Ext4Inode> {
        if inode_num == 0 || inode_num > self.superblock.s_inodes_count {
            return Err(FsError::NotFound);
        }
        
        let inodes_per_group = self.superblock.s_inodes_per_group;
        let inode_size = self.superblock.inode_size() as u32;
        
        let bg_index = (inode_num - 1) / inodes_per_group;
        let inode_index = (inode_num - 1) % inodes_per_group;
        
        let bg = &self.block_groups[bg_index as usize];
        let inode_table_block = bg.inode_table();
        
        let inodes_per_block = self.block_size / inode_size;
        let block_offset = inode_index / inodes_per_block;
        let offset_in_block = (inode_index % inodes_per_block) * inode_size;
        
        let mut block_buf = vec![0u8; self.block_size as usize];
        {
            let dev = self.device.read();
            dev.read_block(inode_table_block + block_offset as u64, &mut block_buf)
                .map_err(|_| FsError::IoError)?;
        }
        
        let inode: Ext4Inode = unsafe {
            core::ptr::read(block_buf[offset_in_block as usize..].as_ptr() as *const Ext4Inode)
        };
        
        Ok(inode)
    }
    
    fn read_block_data(&self, block_num: u64) -> FsResult<Vec<u8>> {
        let mut buf = vec![0u8; self.block_size as usize];
        let dev = self.device.read();
        dev.read_block(block_num, &mut buf).map_err(|_| FsError::IoError)?;
        Ok(buf)
    }
    
    fn inode_to_vfs_node(&self, inode_num: u32, name: &str) -> FsResult<VfsNode> {
        let inode = self.read_inode(inode_num)?;
        
        let file_type = inode.file_type();
        let mode = FileMode::new(inode.i_mode);
        
        let data = match file_type {
            FileType::Regular => {
                let mut content = Vec::new();
                let size = inode.size();
                
                if size > 0 && size < 1024 * 1024 {
                    content = self.read_file_data(&inode)?;
                }
                
                crate::fs::vfs::node::VfsNodeData::Regular(content)
            }
            FileType::Directory => {
                let entries = self.read_directory_entries(&inode)?;
                crate::fs::vfs::node::VfsNodeData::Directory(entries)
            }
            FileType::Symlink => {
                let target = self.read_symlink(&inode)?;
                crate::fs::vfs::node::VfsNodeData::Symlink(target)
            }
            _ => crate::fs::vfs::node::VfsNodeData::Regular(Vec::new()),
        };
        
        Ok(VfsNode {
            name: name.to_string(),
            inode: inode_num as u64,
            mode,
            uid: inode.uid(),
            gid: inode.gid(),
            size: inode.size(),
            atime: inode.i_atime as u64,
            mtime: inode.i_mtime as u64,
            ctime: inode.i_ctime as u64,
            nlink: inode.i_links_count as u64,
            device: None,
            data,
        })
    }
    
    fn read_file_data(&self, inode: &Ext4Inode) -> FsResult<Vec<u8>> {
        let size = inode.size() as usize;
        let mut data = Vec::with_capacity(size);
        
        if inode.uses_extents() {
            self.read_extent_data(inode, &mut data, size)?;
        } else {
            self.read_block_map_data(inode, &mut data, size)?;
        }
        
        Ok(data)
    }
    
    fn read_extent_data(&self, _inode: &Ext4Inode, data: &mut Vec<u8>, _size: usize) -> FsResult<()> {
        Ok(())
    }
    
    fn read_block_map_data(&self, inode: &Ext4Inode, data: &mut Vec<u8>, size: usize) -> FsResult<()> {
        let mut remaining = size;
        let mut block_idx = 0u32;
        
        while remaining > 0 && block_idx < 12 {
            let block_num = inode.i_block[block_idx as usize];
            if block_num == 0 {
                break;
            }
            
            let block_data = self.read_block_data(block_num as u64)?;
            let to_copy = core::cmp::min(remaining, self.block_size as usize);
            data.extend_from_slice(&block_data[..to_copy]);
            
            remaining -= to_copy;
            block_idx += 1;
        }
        
        Ok(())
    }
    
    fn read_directory_entries(&self, inode: &Ext4Inode) -> FsResult<Vec<DirEntry>> {
        let mut entries = Vec::new();
        let data = self.read_file_data(inode)?;
        
        let mut offset = 0usize;
        while offset < data.len() {
            if offset + 8 > data.len() {
                break;
            }
            
            let dir_entry: Ext4DirEntry = unsafe {
                core::ptr::read(data[offset..].as_ptr() as *const Ext4DirEntry)
            };
            
            if dir_entry.inode != 0 && dir_entry.name_len > 0 {
                let name_start = offset + 8;
                let name_end = name_start + dir_entry.name_len as usize;
                
                if name_end <= data.len() {
                    let name_bytes = &data[name_start..name_end];
                    if let Ok(name) = core::str::from_utf8(name_bytes) {
                        entries.push(DirEntry::new(
                            name.to_string(),
                            dir_entry.inode as u64,
                            dir_entry.file_type(),
                        ));
                    }
                }
            }
            
            if dir_entry.rec_len == 0 {
                break;
            }
            offset += dir_entry.rec_len as usize;
        }
        
        Ok(entries)
    }
    
    fn read_symlink(&self, inode: &Ext4Inode) -> FsResult<String> {
        let size = inode.size() as usize;
        
        if size <= 60 {
            let mut block_buf = [0u8; 60];
            unsafe {
                let mut tmp = [0u32; 15];

                // compute raw pointer to i_block without referencing it
                let inode_ptr = inode as *const Ext4Inode;
                let i_block_ptr = inode_ptr.cast::<u8>()
                    .add(offset_of!(Ext4Inode, i_block))
                    as *const u32;

                for i in 0..15 {
                    let p = i_block_ptr.add(i);
                    tmp[i] = core::ptr::read_unaligned(p);
                }

                core::ptr::copy_nonoverlapping(
                    tmp.as_ptr() as *const u8,
                    block_buf.as_mut_ptr(),
                    size
                );
            }

            let bytes = &block_buf[..size];
            String::from_utf8(bytes.to_vec()).map_err(|_| FsError::IoError)
        } else {
            let data = self.read_file_data(inode)?;
            String::from_utf8(data).map_err(|_| FsError::IoError)
        }
    }
}

impl Filesystem for Ext4Filesystem {
    fn name(&self) -> &str {
        "ext4"
    }
    
    fn root(&self) -> FsResult<VfsNode> {
        self.inode_to_vfs_node(EXT4_ROOT_INO, "/")
    }
    
    fn lookup(&self, parent: InodeNumber, name: &str) -> FsResult<VfsNode> {
        let parent_inode = self.read_inode(parent as u32)?;
        
        if !parent_inode.is_dir() {
            return Err(FsError::NotDirectory);
        }
        
        let entries = self.read_directory_entries(&parent_inode)?;
        
        for entry in entries {
            if entry.name == name {
                return self.inode_to_vfs_node(entry.inode as u32, name);
            }
        }
        
        Err(FsError::NotFound)
    }
    
    fn read(&self, inode: InodeNumber, offset: u64, buf: &mut [u8]) -> FsResult<usize> {
        let node = self.read_inode(inode as u32)?;
        let data = self.read_file_data(&node)?;
        
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
        let node = self.inode_to_vfs_node(inode as u32, "")?;
        Ok(node.stat())
    }
    
    fn readdir(&self, inode: InodeNumber) -> FsResult<Vec<DirEntry>> {
        let node = self.read_inode(inode as u32)?;
        
        if !node.is_dir() {
            return Err(FsError::NotDirectory);
        }
        
        self.read_directory_entries(&node)
    }
    
    fn sync(&mut self) -> FsResult<()> {
        Ok(())
    }
}
