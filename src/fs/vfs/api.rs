use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use lazy_static::lazy_static;
use crate::fs::{FileMode, FileStat, FsResult, FsError};
use super::vfs::VFS;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct OpenFlags: u32 {
        const O_RDONLY = 0;
        const O_WRONLY = 1;
        const O_RDWR = 2;
        const O_CREAT = 0o100;
        const O_EXCL = 0o200;
        const O_NOCTTY = 0o400;
        const O_TRUNC = 0o1000;
        const O_APPEND = 0o2000;
        const O_NONBLOCK = 0o4000;
        const O_SYNC = 0o10000;
        const O_DIRECTORY = 0o200000;
        const O_NOFOLLOW = 0o400000;
        const O_CLOEXEC = 0o2000000;
    }
}

impl OpenFlags {
    pub fn can_read(&self) -> bool {
        let access = self.bits() & 3;
        access == 0 || access == 2
    }
    
    pub fn can_write(&self) -> bool {
        let access = self.bits() & 3;
        access == 1 || access == 2
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekFrom {
    Start(u64),
    Current(i64),
    End(i64),
}

pub const SEEK_SET: i32 = 0;
pub const SEEK_CUR: i32 = 1;
pub const SEEK_END: i32 = 2;

#[derive(Debug)]
pub struct FileDescriptor {
    pub path: String,
    pub inode: u64,
    pub flags: OpenFlags,
    pub offset: u64,
    pub mode: FileMode,
}

impl FileDescriptor {
    pub fn new(path: String, inode: u64, flags: OpenFlags, mode: FileMode) -> Self {
        FileDescriptor {
            path,
            inode,
            flags,
            offset: 0,
            mode,
        }
    }
    
    pub fn seek(&mut self, pos: SeekFrom, size: u64) -> FsResult<u64> {
        let new_offset = match pos {
            SeekFrom::Start(n) => n as i64,
            SeekFrom::Current(n) => self.offset as i64 + n,
            SeekFrom::End(n) => size as i64 + n,
        };
        
        if new_offset < 0 {
            return Err(FsError::InvalidArgument);
        }
        
        self.offset = new_offset as u64;
        Ok(self.offset)
    }
}

pub fn open(path: &str, flags: OpenFlags, mode: u16) -> FsResult<FileDescriptor> {
    // Lookup first, outside of any VFS lock lifetime issues
    let lookup_result = {
        let vfs = VFS.lock();
        vfs.lookup_path(path).map(|node| node.clone())
    };

    let node = match lookup_result {
        Ok(node) => {
            if flags.contains(OpenFlags::O_EXCL) && flags.contains(OpenFlags::O_CREAT) {
                return Err(FsError::AlreadyExists);
            }
            node
        }
        Err(FsError::NotFound) if flags.contains(OpenFlags::O_CREAT) => {
            let mut vfs = VFS.lock();
            vfs.create_file(path, FileMode::new(mode))?
        }
        Err(e) => return Err(e),
    };

    if flags.contains(OpenFlags::O_DIRECTORY) && !node.is_dir() {
        return Err(FsError::NotDirectory);
    }

    Ok(FileDescriptor::new(
        String::from(path),
        node.inode,
        flags,
        node.mode,
    ))
}

pub fn read(fd: &mut FileDescriptor, buf: &mut [u8]) -> FsResult<usize> {
    if !fd.flags.can_read() {
        return Err(FsError::PermissionDenied);
    }
    
    let vfs = VFS.lock();
    let node = vfs.get_node(fd.inode)?;
    let bytes_read = node.read(fd.offset, buf)?;
    fd.offset += bytes_read as u64;
    Ok(bytes_read)
}

pub fn write(fd: &mut FileDescriptor, buf: &[u8]) -> FsResult<usize> {
    if !fd.flags.can_write() {
        return Err(FsError::PermissionDenied);
    }
    
    let mut vfs = VFS.lock();
    
    if fd.flags.contains(OpenFlags::O_APPEND) {
        let node = vfs.get_node(fd.inode)?;
        fd.offset = node.size;
    }
    
    let bytes_written = vfs.write_node(fd.inode, fd.offset, buf)?;
    fd.offset += bytes_written as u64;
    Ok(bytes_written)
}

pub fn close(_fd: FileDescriptor) -> FsResult<()> {
    Ok(())
}

pub fn lseek(fd: &mut FileDescriptor, offset: i64, whence: i32) -> FsResult<u64> {
    let vfs = VFS.lock();
    let node = vfs.get_node(fd.inode)?;
    
    let pos = match whence {
        SEEK_SET => SeekFrom::Start(offset as u64),
        SEEK_CUR => SeekFrom::Current(offset),
        SEEK_END => SeekFrom::End(offset),
        _ => return Err(FsError::InvalidArgument),
    };
    
    fd.seek(pos, node.size)
}

pub fn stat(path: &str) -> FsResult<FileStat> {
    let vfs = VFS.lock();
    let node = vfs.lookup_path(path)?;
    Ok(node.stat())
}

pub fn fstat(fd: &FileDescriptor) -> FsResult<FileStat> {
    let vfs = VFS.lock();
    let node = vfs.get_node(fd.inode)?;
    Ok(node.stat())
}

pub fn mkdir(path: &str, mode: u16) -> FsResult<()> {
    let mut vfs = VFS.lock();
    vfs.create_directory(path, FileMode::new(mode))?;
    Ok(())
}

pub fn rmdir(path: &str) -> FsResult<()> {
    let mut vfs = VFS.lock();
    vfs.remove_directory(path)
}

pub fn unlink(path: &str) -> FsResult<()> {
    let mut vfs = VFS.lock();
    vfs.remove_file(path)
}

pub fn rename(old_path: &str, new_path: &str) -> FsResult<()> {
    let mut vfs = VFS.lock();
    vfs.rename(old_path, new_path)
}

pub fn readdir(path: &str) -> FsResult<Vec<super::node::DirEntry>> {
    let vfs = VFS.lock();
    let node = vfs.lookup_path(path)?;
    
    match node.readdir() {
        Ok(entries) => Ok(entries.to_vec()),
        Err(e) => Err(e),
    }
}

pub fn chmod(path: &str, mode: u16) -> FsResult<()> {
    let mut vfs = VFS.lock();
    vfs.chmod(path, mode)
}

pub fn chown(path: &str, uid: u32, gid: u32) -> FsResult<()> {
    let mut vfs = VFS.lock();
    vfs.chown(path, uid, gid)
}

pub fn symlink(target: &str, linkpath: &str) -> FsResult<()> {
    let mut vfs = VFS.lock();
    vfs.create_symlink(linkpath, target)?;
    Ok(())
}

pub fn readlink(path: &str) -> FsResult<String> {
    let vfs = VFS.lock();
    vfs.read_symlink(path)
}

pub fn access(path: &str, mode: i32) -> FsResult<()> {
    let vfs = VFS.lock();
    let _node = vfs.lookup_path(path)?;
    Ok(())
}

pub fn truncate(path: &str, length: u64) -> FsResult<()> {
    let mut vfs = VFS.lock();
    vfs.truncate(path, length)
}

pub fn sync() -> FsResult<()> {
    let mut vfs = VFS.lock();
    vfs.sync()
}
