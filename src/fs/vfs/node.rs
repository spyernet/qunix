use alloc::string::String;
use alloc::vec::Vec;
use alloc::sync::Arc;
use spin::RwLock;
use crate::fs::{FileMode, FileStat, FileType, FsResult, FsError};

pub type InodeNumber = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceId {
    pub major: u16,
    pub minor: u16,
}

impl DeviceId {
    pub fn new(major: u16, minor: u16) -> Self {
        DeviceId { major, minor }
    }
    
    pub fn to_u64(&self) -> u64 {
        ((self.major as u64) << 16) | (self.minor as u64)
    }
}

#[derive(Clone)]
pub struct VfsNode {
    pub name: String,
    pub inode: InodeNumber,
    pub mode: FileMode,
    pub uid: u32,
    pub gid: u32,
    pub size: u64,
    pub atime: u64,
    pub mtime: u64,
    pub ctime: u64,
    pub nlink: u64,
    pub device: Option<DeviceId>,
    pub data: VfsNodeData,
}

#[derive(Clone)]
pub enum VfsNodeData {
    Regular(Vec<u8>),
    Directory(Vec<DirEntry>),
    Symlink(String),
    Device(DeviceId),
    Fifo,
    Socket,
    Mounted(Arc<RwLock<dyn Filesystem + Send + Sync>>),
}

#[derive(Clone, Debug)]
pub struct DirEntry {
    pub name: String,
    pub inode: InodeNumber,
    pub file_type: FileType,
}

impl DirEntry {
    pub fn new(name: String, inode: InodeNumber, file_type: FileType) -> Self {
        DirEntry { name, inode, file_type }
    }
}

impl VfsNode {
    pub fn new_file(name: String, inode: InodeNumber, mode: u16) -> Self {
        VfsNode {
            name,
            inode,
            mode: FileMode::new(FileMode::S_IFREG | (mode & 0o7777)),
            uid: 0,
            gid: 0,
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
            nlink: 1,
            device: None,
            data: VfsNodeData::Regular(Vec::new()),
        }
    }
    
    pub fn new_directory(name: String, inode: InodeNumber, mode: u16) -> Self {
        let mut entries = Vec::new();
        entries.push(DirEntry::new(".".into(), inode, FileType::Directory));
        
        VfsNode {
            name,
            inode,
            mode: FileMode::new(FileMode::S_IFDIR | (mode & 0o7777)),
            uid: 0,
            gid: 0,
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
            nlink: 2,
            device: None,
            data: VfsNodeData::Directory(entries),
        }
    }
    
    pub fn new_symlink(name: String, inode: InodeNumber, target: String) -> Self {
        let size = target.len() as u64;
        VfsNode {
            name,
            inode,
            mode: FileMode::new(FileMode::S_IFLNK | 0o777),
            uid: 0,
            gid: 0,
            size,
            atime: 0,
            mtime: 0,
            ctime: 0,
            nlink: 1,
            device: None,
            data: VfsNodeData::Symlink(target),
        }
    }
    
    pub fn new_char_device(name: String, inode: InodeNumber, device: DeviceId, mode: u16) -> Self {
        VfsNode {
            name,
            inode,
            mode: FileMode::new(FileMode::S_IFCHR | (mode & 0o7777)),
            uid: 0,
            gid: 0,
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
            nlink: 1,
            device: Some(device),
            data: VfsNodeData::Device(device),
        }
    }
    
    pub fn new_block_device(name: String, inode: InodeNumber, device: DeviceId, mode: u16) -> Self {
        VfsNode {
            name,
            inode,
            mode: FileMode::new(FileMode::S_IFBLK | (mode & 0o7777)),
            uid: 0,
            gid: 0,
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
            nlink: 1,
            device: Some(device),
            data: VfsNodeData::Device(device),
        }
    }
    
    pub fn file_type(&self) -> FileType {
        self.mode.file_type()
    }
    
    pub fn is_dir(&self) -> bool {
        self.mode.is_dir()
    }
    
    pub fn is_file(&self) -> bool {
        self.mode.is_file()
    }
    
    pub fn is_symlink(&self) -> bool {
        self.mode.is_symlink()
    }
    
    pub fn stat(&self) -> FileStat {
        FileStat {
            dev: 0,
            ino: self.inode,
            mode: self.mode,
            nlink: self.nlink,
            uid: self.uid,
            gid: self.gid,
            rdev: self.device.map_or(0, |d| d.to_u64()),
            size: self.size,
            blksize: 4096,
            blocks: (self.size + 511) / 512,
            atime: self.atime,
            mtime: self.mtime,
            ctime: self.ctime,
        }
    }
    
    pub fn read(&self, offset: u64, buf: &mut [u8]) -> FsResult<usize> {
        match &self.data {
            VfsNodeData::Regular(data) => {
                if offset >= data.len() as u64 {
                    return Ok(0);
                }
                let start = offset as usize;
                let end = core::cmp::min(start + buf.len(), data.len());
                let len = end - start;
                buf[..len].copy_from_slice(&data[start..end]);
                Ok(len)
            }
            VfsNodeData::Symlink(target) => {
                if offset >= target.len() as u64 {
                    return Ok(0);
                }
                let start = offset as usize;
                let end = core::cmp::min(start + buf.len(), target.len());
                let len = end - start;
                buf[..len].copy_from_slice(&target.as_bytes()[start..end]);
                Ok(len)
            }
            _ => Err(FsError::InvalidArgument),
        }
    }
    
    pub fn write(&mut self, offset: u64, buf: &[u8]) -> FsResult<usize> {
        match &mut self.data {
            VfsNodeData::Regular(data) => {
                let offset = offset as usize;
                if offset + buf.len() > data.len() {
                    data.resize(offset + buf.len(), 0);
                }
                data[offset..offset + buf.len()].copy_from_slice(buf);
                self.size = data.len() as u64;
                Ok(buf.len())
            }
            _ => Err(FsError::InvalidArgument),
        }
    }
    
    pub fn truncate(&mut self, size: u64) -> FsResult<()> {
        match &mut self.data {
            VfsNodeData::Regular(data) => {
                data.resize(size as usize, 0);
                self.size = size;
                Ok(())
            }
            _ => Err(FsError::InvalidArgument),
        }
    }
    
    pub fn add_entry(&mut self, entry: DirEntry) -> FsResult<()> {
        match &mut self.data {
            VfsNodeData::Directory(entries) => {
                if entries.iter().any(|e| e.name == entry.name) {
                    return Err(FsError::AlreadyExists);
                }
                entries.push(entry);
                Ok(())
            }
            _ => Err(FsError::NotDirectory),
        }
    }
    
    pub fn remove_entry(&mut self, name: &str) -> FsResult<DirEntry> {
        match &mut self.data {
            VfsNodeData::Directory(entries) => {
                if let Some(pos) = entries.iter().position(|e| e.name == name) {
                    Ok(entries.remove(pos))
                } else {
                    Err(FsError::NotFound)
                }
            }
            _ => Err(FsError::NotDirectory),
        }
    }
    
    pub fn lookup(&self, name: &str) -> FsResult<&DirEntry> {
        match &self.data {
            VfsNodeData::Directory(entries) => {
                entries.iter()
                    .find(|e| e.name == name)
                    .ok_or(FsError::NotFound)
            }
            _ => Err(FsError::NotDirectory),
        }
    }
    
    pub fn readdir(&self) -> FsResult<&[DirEntry]> {
        match &self.data {
            VfsNodeData::Directory(entries) => Ok(entries),
            _ => Err(FsError::NotDirectory),
        }
    }
}

pub trait Filesystem {
    fn name(&self) -> &str;
    fn root(&self) -> FsResult<VfsNode>;
    fn lookup(&self, parent: InodeNumber, name: &str) -> FsResult<VfsNode>;
    fn read(&self, inode: InodeNumber, offset: u64, buf: &mut [u8]) -> FsResult<usize>;
    fn write(&mut self, inode: InodeNumber, offset: u64, buf: &[u8]) -> FsResult<usize>;
    fn create(&mut self, parent: InodeNumber, name: &str, mode: FileMode) -> FsResult<VfsNode>;
    fn mkdir(&mut self, parent: InodeNumber, name: &str, mode: FileMode) -> FsResult<VfsNode>;
    fn unlink(&mut self, parent: InodeNumber, name: &str) -> FsResult<()>;
    fn rmdir(&mut self, parent: InodeNumber, name: &str) -> FsResult<()>;
    fn rename(&mut self, old_parent: InodeNumber, old_name: &str, new_parent: InodeNumber, new_name: &str) -> FsResult<()>;
    fn stat(&self, inode: InodeNumber) -> FsResult<FileStat>;
    fn readdir(&self, inode: InodeNumber) -> FsResult<Vec<DirEntry>>;
    fn sync(&mut self) -> FsResult<()>;
}
