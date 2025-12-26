use crate::fs::{FsResult, FsError, FileStat};
use crate::fs::vfs::api::OpenFlags;
use alloc::string::String;

pub fn posix_open(path: &str, flags: i32, mode: u32) -> FsResult<i32> {
    let open_flags = OpenFlags::from_bits_truncate(flags as u32);
    
    match crate::fs::vfs::api::open(path, open_flags, mode as u16) {
        Ok(_fd) => {
            Ok(3)
        }
        Err(e) => Err(e),
    }
}

pub fn posix_close(fd: i32) -> FsResult<()> {
    if fd < 0 {
        return Err(FsError::InvalidArgument);
    }
    Ok(())
}

pub fn posix_read(fd: i32, _buf: &mut [u8]) -> FsResult<usize> {
    if fd < 0 {
        return Err(FsError::InvalidArgument);
    }
    
    if fd == 0 {
        return Ok(0);
    }
    
    Err(FsError::InvalidArgument)
}

pub fn posix_write(fd: i32, buf: &[u8]) -> FsResult<usize> {
    if fd < 0 {
        return Err(FsError::InvalidArgument);
    }
    
    if fd == 1 || fd == 2 {
        if let Ok(s) = core::str::from_utf8(buf) {
            crate::print!("{}", s);
        }
        return Ok(buf.len());
    }
    
    Err(FsError::InvalidArgument)
}

pub fn posix_lseek(fd: i32, _offset: i64, _whence: i32) -> FsResult<u64> {
    if fd < 0 {
        return Err(FsError::InvalidArgument);
    }
    
    Err(FsError::InvalidArgument)
}

pub fn posix_stat(path: &str) -> FsResult<PosixStat> {
    let stat = crate::fs::vfs::api::stat(path)?;
    Ok(PosixStat::from(stat))
}

pub fn posix_fstat(fd: i32) -> FsResult<PosixStat> {
    if fd < 0 {
        return Err(FsError::InvalidArgument);
    }
    
    Err(FsError::InvalidArgument)
}

pub fn posix_lstat(path: &str) -> FsResult<PosixStat> {
    posix_stat(path)
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PosixStat {
    pub st_dev: u64,
    pub st_ino: u64,
    pub st_mode: u32,
    pub st_nlink: u64,
    pub st_uid: u32,
    pub st_gid: u32,
    pub st_rdev: u64,
    pub st_size: i64,
    pub st_blksize: i64,
    pub st_blocks: i64,
    pub st_atime: i64,
    pub st_atime_nsec: i64,
    pub st_mtime: i64,
    pub st_mtime_nsec: i64,
    pub st_ctime: i64,
    pub st_ctime_nsec: i64,
}

impl From<FileStat> for PosixStat {
    fn from(stat: FileStat) -> Self {
        PosixStat {
            st_dev: stat.dev,
            st_ino: stat.ino,
            st_mode: stat.mode.0 as u32,
            st_nlink: stat.nlink,
            st_uid: stat.uid,
            st_gid: stat.gid,
            st_rdev: stat.rdev,
            st_size: stat.size as i64,
            st_blksize: stat.blksize as i64,
            st_blocks: stat.blocks as i64,
            st_atime: stat.atime as i64,
            st_atime_nsec: 0,
            st_mtime: stat.mtime as i64,
            st_mtime_nsec: 0,
            st_ctime: stat.ctime as i64,
            st_ctime_nsec: 0,
        }
    }
}

pub fn posix_access(path: &str, mode: i32) -> FsResult<()> {
    crate::fs::vfs::api::access(path, mode)
}

pub fn posix_chmod(path: &str, mode: u32) -> FsResult<()> {
    crate::fs::vfs::api::chmod(path, mode as u16)
}

pub fn posix_chown(path: &str, uid: u32, gid: u32) -> FsResult<()> {
    crate::fs::vfs::api::chown(path, uid, gid)
}

pub fn posix_mkdir(path: &str, mode: u32) -> FsResult<()> {
    crate::fs::vfs::api::mkdir(path, mode as u16)
}

pub fn posix_rmdir(path: &str) -> FsResult<()> {
    crate::fs::vfs::api::rmdir(path)
}

pub fn posix_unlink(path: &str) -> FsResult<()> {
    crate::fs::vfs::api::unlink(path)
}

pub fn posix_rename(old_path: &str, new_path: &str) -> FsResult<()> {
    crate::fs::vfs::api::rename(old_path, new_path)
}

pub fn posix_symlink(target: &str, linkpath: &str) -> FsResult<()> {
    crate::fs::vfs::api::symlink(target, linkpath)
}

pub fn posix_readlink(path: &str) -> FsResult<String> {
    crate::fs::vfs::api::readlink(path)
}

pub fn posix_truncate(path: &str, length: i64) -> FsResult<()> {
    if length < 0 {
        return Err(FsError::InvalidArgument);
    }
    crate::fs::vfs::api::truncate(path, length as u64)
}

pub fn posix_getcwd() -> FsResult<String> {
    let vfs = crate::fs::vfs::vfs::VFS.lock();
    Ok(String::from(vfs.get_cwd()))
}

pub fn posix_chdir(path: &str) -> FsResult<()> {
    let mut vfs = crate::fs::vfs::vfs::VFS.lock();
    vfs.set_cwd(path)
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct PosixDirent {
    pub d_ino: u64,
    pub d_off: i64,
    pub d_reclen: u16,
    pub d_type: u8,
    pub d_name: [u8; 256],
}

pub const DT_UNKNOWN: u8 = 0;
pub const DT_FIFO: u8 = 1;
pub const DT_CHR: u8 = 2;
pub const DT_DIR: u8 = 4;
pub const DT_BLK: u8 = 6;
pub const DT_REG: u8 = 8;
pub const DT_LNK: u8 = 10;
pub const DT_SOCK: u8 = 12;

pub const O_RDONLY: i32 = 0;
pub const O_WRONLY: i32 = 1;
pub const O_RDWR: i32 = 2;
pub const O_CREAT: i32 = 0o100;
pub const O_EXCL: i32 = 0o200;
pub const O_TRUNC: i32 = 0o1000;
pub const O_APPEND: i32 = 0o2000;
pub const O_NONBLOCK: i32 = 0o4000;
pub const O_DIRECTORY: i32 = 0o200000;

pub const SEEK_SET: i32 = 0;
pub const SEEK_CUR: i32 = 1;
pub const SEEK_END: i32 = 2;

pub const R_OK: i32 = 4;
pub const W_OK: i32 = 2;
pub const X_OK: i32 = 1;
pub const F_OK: i32 = 0;
