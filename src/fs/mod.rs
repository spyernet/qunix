pub mod vfs;
pub mod ext4;
pub mod fat32;
pub mod mount;

pub use vfs::*;
pub use mount::*;

use alloc::string::String;

pub fn init() {
    vfs::init();
    mount::init();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Regular,
    Directory,
    Symlink,
    CharDevice,
    BlockDevice,
    Fifo,
    Socket,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileMode(pub u16);

impl FileMode {
    pub const S_IFMT: u16 = 0o170000;
    pub const S_IFSOCK: u16 = 0o140000;
    pub const S_IFLNK: u16 = 0o120000;
    pub const S_IFREG: u16 = 0o100000;
    pub const S_IFBLK: u16 = 0o060000;
    pub const S_IFDIR: u16 = 0o040000;
    pub const S_IFCHR: u16 = 0o020000;
    pub const S_IFIFO: u16 = 0o010000;
    
    pub const S_ISUID: u16 = 0o4000;
    pub const S_ISGID: u16 = 0o2000;
    pub const S_ISVTX: u16 = 0o1000;
    
    pub const S_IRWXU: u16 = 0o0700;
    pub const S_IRUSR: u16 = 0o0400;
    pub const S_IWUSR: u16 = 0o0200;
    pub const S_IXUSR: u16 = 0o0100;
    
    pub const S_IRWXG: u16 = 0o0070;
    pub const S_IRGRP: u16 = 0o0040;
    pub const S_IWGRP: u16 = 0o0020;
    pub const S_IXGRP: u16 = 0o0010;
    
    pub const S_IRWXO: u16 = 0o0007;
    pub const S_IROTH: u16 = 0o0004;
    pub const S_IWOTH: u16 = 0o0002;
    pub const S_IXOTH: u16 = 0o0001;
    
    pub fn new(mode: u16) -> Self {
        FileMode(mode)
    }
    
    pub fn file_type(&self) -> FileType {
        match self.0 & Self::S_IFMT {
            Self::S_IFREG => FileType::Regular,
            Self::S_IFDIR => FileType::Directory,
            Self::S_IFLNK => FileType::Symlink,
            Self::S_IFCHR => FileType::CharDevice,
            Self::S_IFBLK => FileType::BlockDevice,
            Self::S_IFIFO => FileType::Fifo,
            Self::S_IFSOCK => FileType::Socket,
            _ => FileType::Regular,
        }
    }
    
    pub fn is_dir(&self) -> bool {
        (self.0 & Self::S_IFMT) == Self::S_IFDIR
    }
    
    pub fn is_file(&self) -> bool {
        (self.0 & Self::S_IFMT) == Self::S_IFREG
    }
    
    pub fn is_symlink(&self) -> bool {
        (self.0 & Self::S_IFMT) == Self::S_IFLNK
    }
    
    pub fn permissions(&self) -> u16 {
        self.0 & 0o7777
    }
    
    pub fn can_read(&self, is_owner: bool, is_group: bool) -> bool {
        if is_owner {
            self.0 & Self::S_IRUSR != 0
        } else if is_group {
            self.0 & Self::S_IRGRP != 0
        } else {
            self.0 & Self::S_IROTH != 0
        }
    }
    
    pub fn can_write(&self, is_owner: bool, is_group: bool) -> bool {
        if is_owner {
            self.0 & Self::S_IWUSR != 0
        } else if is_group {
            self.0 & Self::S_IWGRP != 0
        } else {
            self.0 & Self::S_IWOTH != 0
        }
    }
    
    pub fn can_execute(&self, is_owner: bool, is_group: bool) -> bool {
        if is_owner {
            self.0 & Self::S_IXUSR != 0
        } else if is_group {
            self.0 & Self::S_IXGRP != 0
        } else {
            self.0 & Self::S_IXOTH != 0
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileStat {
    pub dev: u64,
    pub ino: u64,
    pub mode: FileMode,
    pub nlink: u64,
    pub uid: u32,
    pub gid: u32,
    pub rdev: u64,
    pub size: u64,
    pub blksize: u64,
    pub blocks: u64,
    pub atime: u64,
    pub mtime: u64,
    pub ctime: u64,
}

impl FileStat {
    pub fn new() -> Self {
        FileStat {
            dev: 0,
            ino: 0,
            mode: FileMode::new(0),
            nlink: 1,
            uid: 0,
            gid: 0,
            rdev: 0,
            size: 0,
            blksize: 4096,
            blocks: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        }
    }
}

#[derive(Debug)]
pub enum FsError {
    NotFound,
    PermissionDenied,
    AlreadyExists,
    NotDirectory,
    IsDirectory,
    NotEmpty,
    InvalidPath,
    IoError,
    NoSpace,
    ReadOnly,
    TooManyLinks,
    NameTooLong,
    InvalidArgument,
    NotSupported,
    Busy,
}

pub type FsResult<T> = Result<T, FsError>;
