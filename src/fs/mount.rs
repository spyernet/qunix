use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::sync::Arc;
use alloc::format;
use spin::{Mutex, RwLock};
use lazy_static::lazy_static;
use crate::fs::{FsResult, FsError};
use crate::fs::vfs::node::Filesystem;

#[derive(Clone)]
pub struct MountPoint {
    pub path: String,
    pub device: String,
    pub fs_type: String,
    pub flags: MountFlags,
    pub filesystem: Arc<RwLock<dyn Filesystem + Send + Sync>>,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct MountFlags: u32 {
        const RDONLY = 1 << 0;
        const NOSUID = 1 << 1;
        const NODEV = 1 << 2;
        const NOEXEC = 1 << 3;
        const SYNCHRONOUS = 1 << 4;
        const REMOUNT = 1 << 5;
        const MANDLOCK = 1 << 6;
        const DIRSYNC = 1 << 7;
        const NOATIME = 1 << 8;
        const NODIRATIME = 1 << 9;
        const BIND = 1 << 10;
        const MOVE = 1 << 11;
        const REC = 1 << 12;
        const SILENT = 1 << 13;
        const RELATIME = 1 << 14;
    }
}

lazy_static! {
    static ref MOUNT_TABLE: Mutex<Vec<MountPoint>> = Mutex::new(Vec::new());
}

pub fn init() {
}

pub fn mount(
    source: &str,
    target: &str,
    fs_type: &str,
    flags: MountFlags,
    filesystem: Arc<RwLock<dyn Filesystem + Send + Sync>>,
) -> FsResult<()> {
    let mut table = MOUNT_TABLE.lock();
    
    if table.iter().any(|m| m.path == target) {
        return Err(FsError::Busy);
    }
    
    let mount_point = MountPoint {
        path: target.to_string(),
        device: source.to_string(),
        fs_type: fs_type.to_string(),
        flags,
        filesystem,
    };
    
    table.push(mount_point);
    
    table.sort_by(|a, b| b.path.len().cmp(&a.path.len()));
    
    Ok(())
}

pub fn umount(target: &str) -> FsResult<()> {
    let mut table = MOUNT_TABLE.lock();
    
    if let Some(pos) = table.iter().position(|m| m.path == target) {
        table.remove(pos);
        Ok(())
    } else {
        Err(FsError::NotFound)
    }
}

pub fn find_mount_point(path: &str) -> Option<MountPoint> {
    let table = MOUNT_TABLE.lock();
    
    for mount in table.iter() {
        if path.starts_with(&mount.path) {
            return Some(mount.clone());
        }
    }
    
    None
}

pub fn get_mount_table() -> Vec<MountPoint> {
    MOUNT_TABLE.lock().clone()
}

pub fn is_mounted(path: &str) -> bool {
    MOUNT_TABLE.lock().iter().any(|m| m.path == path)
}

pub fn get_relative_path(path: &str, mount_point: &str) -> String {
    if mount_point == "/" {
        path.to_string()
    } else {
        let relative = path.strip_prefix(mount_point).unwrap_or(path);
        if relative.is_empty() {
            "/".to_string()
        } else if relative.starts_with('/') {
            relative.to_string()
        } else {
            format!("/{}", relative)
        }
    }
}

#[derive(Debug, Clone)]
pub struct MountInfo {
    pub device: String,
    pub mount_point: String,
    pub fs_type: String,
    pub options: String,
}

pub fn get_mounts() -> Vec<MountInfo> {
    MOUNT_TABLE.lock().iter().map(|m| {
        let mut options = Vec::new();
        
        if m.flags.contains(MountFlags::RDONLY) {
            options.push("ro");
        } else {
            options.push("rw");
        }
        
        if m.flags.contains(MountFlags::NOSUID) {
            options.push("nosuid");
        }
        
        if m.flags.contains(MountFlags::NODEV) {
            options.push("nodev");
        }
        
        if m.flags.contains(MountFlags::NOEXEC) {
            options.push("noexec");
        }
        
        if m.flags.contains(MountFlags::NOATIME) {
            options.push("noatime");
        }
        
        MountInfo {
            device: m.device.clone(),
            mount_point: m.path.clone(),
            fs_type: m.fs_type.clone(),
            options: options.join(","),
        }
    }).collect()
}

pub fn remount(target: &str, flags: MountFlags) -> FsResult<()> {
    let mut table = MOUNT_TABLE.lock();
    
    if let Some(mount) = table.iter_mut().find(|m| m.path == target) {
        mount.flags = flags;
        Ok(())
    } else {
        Err(FsError::NotFound)
    }
}

pub const MS_RDONLY: u32 = MountFlags::RDONLY.bits();
pub const MS_NOSUID: u32 = MountFlags::NOSUID.bits();
pub const MS_NODEV: u32 = MountFlags::NODEV.bits();
pub const MS_NOEXEC: u32 = MountFlags::NOEXEC.bits();
pub const MS_SYNCHRONOUS: u32 = MountFlags::SYNCHRONOUS.bits();
pub const MS_REMOUNT: u32 = MountFlags::REMOUNT.bits();
pub const MS_BIND: u32 = MountFlags::BIND.bits();
pub const MS_MOVE: u32 = MountFlags::MOVE.bits();
