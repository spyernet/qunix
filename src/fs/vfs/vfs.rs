use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use alloc::collections::BTreeMap;
use spin::Mutex;
use lazy_static::lazy_static;
use crate::fs::{FileMode, FileStat, FileType, FsResult, FsError};
use super::node::{VfsNode, VfsNodeData, DirEntry, InodeNumber};

lazy_static! {
    pub static ref VFS: Mutex<VirtualFileSystem> = Mutex::new(VirtualFileSystem::new());
}

pub struct VirtualFileSystem {
    nodes: BTreeMap<InodeNumber, VfsNode>,
    next_inode: InodeNumber,
    cwd: String,
}

impl VirtualFileSystem {
    pub fn new() -> Self {
        let mut vfs = VirtualFileSystem {
            nodes: BTreeMap::new(),
            next_inode: 2,
            cwd: String::from("/"),
        };
        
        let root = VfsNode::new_directory("/".into(), 1, 0o755);
        vfs.nodes.insert(1, root);
        
        vfs
    }
    
    fn alloc_inode(&mut self) -> InodeNumber {
        let inode = self.next_inode;
        self.next_inode += 1;
        inode
    }
    
    pub fn resolve_path(&self, path: &str) -> String {
        if path.starts_with('/') {
            normalize_path(path)
        } else {
            let full = format!("{}/{}", self.cwd, path);
            normalize_path(&full)
        }
    }
    
    pub fn lookup_path(&self, path: &str) -> FsResult<&VfsNode> {
        let path = self.resolve_path(path);
        
        if path == "/" {
            return self.nodes.get(&1).ok_or(FsError::NotFound);
        }
        
        let mut current_inode: InodeNumber = 1;
        
        for component in path.split('/').filter(|s| !s.is_empty()) {
            let current = self.nodes.get(&current_inode).ok_or(FsError::NotFound)?;
            
            if !current.is_dir() {
                return Err(FsError::NotDirectory);
            }
            
            let entry = current.lookup(component)?;
            current_inode = entry.inode;
        }
        
        self.nodes.get(&current_inode).ok_or(FsError::NotFound)
    }
    
    pub fn lookup_path_mut(&mut self, path: &str) -> FsResult<&mut VfsNode> {
        let path = self.resolve_path(path);
        
        if path == "/" {
            return self.nodes.get_mut(&1).ok_or(FsError::NotFound);
        }
        
        let mut current_inode: InodeNumber = 1;
        
        for component in path.split('/').filter(|s| !s.is_empty()) {
            let current = self.nodes.get(&current_inode).ok_or(FsError::NotFound)?;
            
            if !current.is_dir() {
                return Err(FsError::NotDirectory);
            }
            
            let entry = current.lookup(component)?;
            current_inode = entry.inode;
        }
        
        self.nodes.get_mut(&current_inode).ok_or(FsError::NotFound)
    }
    
    pub fn get_node(&self, inode: InodeNumber) -> FsResult<&VfsNode> {
        self.nodes.get(&inode).ok_or(FsError::NotFound)
    }
    
    pub fn get_node_mut(&mut self, inode: InodeNumber) -> FsResult<&mut VfsNode> {
        self.nodes.get_mut(&inode).ok_or(FsError::NotFound)
    }
    
    fn get_parent_and_name(&self, path: &str) -> FsResult<(String, String)> {
        let path = self.resolve_path(path);
        
        if path == "/" {
            return Err(FsError::InvalidArgument);
        }
        
        let path = path.trim_end_matches('/');
        if let Some(pos) = path.rfind('/') {
            let parent = if pos == 0 { "/".to_string() } else { path[..pos].to_string() };
            let name = path[pos + 1..].to_string();
            Ok((parent, name))
        } else {
            Err(FsError::InvalidPath)
        }
    }
    
    pub fn create_file(&mut self, path: &str, mode: FileMode) -> FsResult<VfsNode> {
        let (parent_path, name) = self.get_parent_and_name(path)?;
        
        let parent_inode = {
            let parent = self.lookup_path(&parent_path)?;
            if !parent.is_dir() {
                return Err(FsError::NotDirectory);
            }
            parent.inode
        };
        
        let inode = self.alloc_inode();
        let node = VfsNode::new_file(name.clone(), inode, mode.0 & 0o7777);
        
        self.nodes.insert(inode, node.clone());
        
        let parent = self.nodes.get_mut(&parent_inode).ok_or(FsError::NotFound)?;
        parent.add_entry(DirEntry::new(name, inode, FileType::Regular))?;
        
        Ok(node)
    }

    pub fn create_device(&mut self, path: &str, device: super::node::DeviceId, mode: FileMode) -> FsResult<VfsNode> {
        let (parent_path, name) = self.get_parent_and_name(path)?;

        let parent_inode = {
            let parent = self.lookup_path(&parent_path)?;
            if !parent.is_dir() {
                return Err(FsError::NotDirectory);
            }
            parent.inode
        };

        let inode = self.alloc_inode();
        let node = VfsNode::new_char_device(name.clone(), inode, device, mode.0 & 0o7777);

        self.nodes.insert(inode, node.clone());

        let parent = self.nodes.get_mut(&parent_inode).ok_or(FsError::NotFound)?;
        parent.add_entry(DirEntry::new(name, inode, FileType::CharDevice))?;

        Ok(node)
    }
    
    pub fn create_directory(&mut self, path: &str, mode: FileMode) -> FsResult<VfsNode> {
        let (parent_path, name) = self.get_parent_and_name(path)?;
        
        let parent_inode = {
            let parent = self.lookup_path(&parent_path)?;
            if !parent.is_dir() {
                return Err(FsError::NotDirectory);
            }
            parent.inode
        };
        
        let inode = self.alloc_inode();
        let mut node = VfsNode::new_directory(name.clone(), inode, mode.0 & 0o7777);
        
        if let VfsNodeData::Directory(ref mut entries) = node.data {
            entries.push(DirEntry::new("..".into(), parent_inode, FileType::Directory));
        }
        
        self.nodes.insert(inode, node.clone());
        
        let parent = self.nodes.get_mut(&parent_inode).ok_or(FsError::NotFound)?;
        parent.add_entry(DirEntry::new(name, inode, FileType::Directory))?;
        parent.nlink += 1;
        
        Ok(node)
    }
    
    pub fn create_symlink(&mut self, path: &str, target: &str) -> FsResult<VfsNode> {
        let (parent_path, name) = self.get_parent_and_name(path)?;
        
        let parent_inode = {
            let parent = self.lookup_path(&parent_path)?;
            if !parent.is_dir() {
                return Err(FsError::NotDirectory);
            }
            parent.inode
        };
        
        let inode = self.alloc_inode();
        let node = VfsNode::new_symlink(name.clone(), inode, target.to_string());
        
        self.nodes.insert(inode, node.clone());
        
        let parent = self.nodes.get_mut(&parent_inode).ok_or(FsError::NotFound)?;
        parent.add_entry(DirEntry::new(name, inode, FileType::Symlink))?;
        
        Ok(node)
    }
    
    pub fn remove_file(&mut self, path: &str) -> FsResult<()> {
        let (parent_path, name) = self.get_parent_and_name(path)?;
        
        let (parent_inode, file_inode) = {
            let parent = self.lookup_path(&parent_path)?;
            let entry = parent.lookup(&name)?;
            
            if entry.file_type == FileType::Directory {
                return Err(FsError::IsDirectory);
            }
            
            (parent.inode, entry.inode)
        };
        
        let parent = self.nodes.get_mut(&parent_inode).ok_or(FsError::NotFound)?;
        parent.remove_entry(&name)?;
        
        self.nodes.remove(&file_inode);
        
        Ok(())
    }
    
    pub fn remove_directory(&mut self, path: &str) -> FsResult<()> {
        let (parent_path, name) = self.get_parent_and_name(path)?;
        
        let (parent_inode, dir_inode) = {
            let dir = self.lookup_path(path)?;
            
            if !dir.is_dir() {
                return Err(FsError::NotDirectory);
            }
            
            if let VfsNodeData::Directory(entries) = &dir.data {
                if entries.iter().any(|e| e.name != "." && e.name != "..") {
                    return Err(FsError::NotEmpty);
                }
            }
            
            let parent = self.lookup_path(&parent_path)?;
            (parent.inode, dir.inode)
        };
        
        let parent = self.nodes.get_mut(&parent_inode).ok_or(FsError::NotFound)?;
        parent.remove_entry(&name)?;
        parent.nlink -= 1;
        
        self.nodes.remove(&dir_inode);
        
        Ok(())
    }
    
    pub fn rename(&mut self, old_path: &str, new_path: &str) -> FsResult<()> {
        let (old_parent_path, old_name) = self.get_parent_and_name(old_path)?;
        let (new_parent_path, new_name) = self.get_parent_and_name(new_path)?;
        
        let (old_parent_inode, entry_inode, file_type) = {
            let old_parent = self.lookup_path(&old_parent_path)?;
            let entry = old_parent.lookup(&old_name)?;
            (old_parent.inode, entry.inode, entry.file_type)
        };
        
        let new_parent_inode = {
            let new_parent = self.lookup_path(&new_parent_path)?;
            new_parent.inode
        };
        
        let old_parent = self.nodes.get_mut(&old_parent_inode).ok_or(FsError::NotFound)?;
        old_parent.remove_entry(&old_name)?;
        
        let node = self.nodes.get_mut(&entry_inode).ok_or(FsError::NotFound)?;
        node.name = new_name.clone();
        
        let new_parent = self.nodes.get_mut(&new_parent_inode).ok_or(FsError::NotFound)?;
        new_parent.add_entry(DirEntry::new(new_name, entry_inode, file_type))?;
        
        Ok(())
    }
    
    pub fn read_symlink(&self, path: &str) -> FsResult<String> {
        let node = self.lookup_path(path)?;
        
        match &node.data {
            VfsNodeData::Symlink(target) => Ok(target.clone()),
            _ => Err(FsError::InvalidArgument),
        }
    }
    
    pub fn write_node(&mut self, inode: InodeNumber, offset: u64, buf: &[u8]) -> FsResult<usize> {
        let node = self.nodes.get_mut(&inode).ok_or(FsError::NotFound)?;
        node.write(offset, buf)
    }
    
    pub fn chmod(&mut self, path: &str, mode: u16) -> FsResult<()> {
        let node = self.lookup_path_mut(path)?;
        let current = node.mode.0 & FileMode::S_IFMT;
        node.mode = FileMode::new(current | (mode & 0o7777));
        Ok(())
    }
    
    pub fn chown(&mut self, path: &str, uid: u32, gid: u32) -> FsResult<()> {
        let node = self.lookup_path_mut(path)?;
        node.uid = uid;
        node.gid = gid;
        Ok(())
    }
    
    pub fn truncate(&mut self, path: &str, length: u64) -> FsResult<()> {
        let node = self.lookup_path_mut(path)?;
        node.truncate(length)
    }
    
    pub fn sync(&mut self) -> FsResult<()> {
        Ok(())
    }
    
    pub fn set_cwd(&mut self, path: &str) -> FsResult<()> {
        let resolved = self.resolve_path(path);
        let node = self.lookup_path(&resolved)?;
        
        if !node.is_dir() {
            return Err(FsError::NotDirectory);
        }
        
        self.cwd = resolved;
        Ok(())
    }
    
    pub fn get_cwd(&self) -> &str {
        &self.cwd
    }
}

pub fn init_vfs() {
    let mut vfs = VFS.lock();
    
    vfs.create_directory("/bin", FileMode::new(0o755)).ok();
    vfs.create_directory("/sbin", FileMode::new(0o755)).ok();
    vfs.create_directory("/etc", FileMode::new(0o755)).ok();
    vfs.create_directory("/dev", FileMode::new(0o755)).ok();
    // Create standard device nodes
    vfs.create_device("/dev/stdin", super::node::DeviceId::new(1, 0), FileMode::new(FileMode::S_IFCHR | 0o666)).ok();
    vfs.create_device("/dev/stdout", super::node::DeviceId::new(1, 1), FileMode::new(FileMode::S_IFCHR | 0o666)).ok();
    vfs.create_device("/dev/stderr", super::node::DeviceId::new(1, 2), FileMode::new(FileMode::S_IFCHR | 0o666)).ok();
    vfs.create_directory("/proc", FileMode::new(0o555)).ok();
    vfs.create_directory("/sys", FileMode::new(0o555)).ok();
    vfs.create_directory("/tmp", FileMode::new(0o1777)).ok();
    vfs.create_directory("/var", FileMode::new(0o755)).ok();
    vfs.create_directory("/var/log", FileMode::new(0o755)).ok();
    vfs.create_directory("/home", FileMode::new(0o755)).ok();
    vfs.create_directory("/root", FileMode::new(0o700)).ok();
    vfs.create_directory("/usr", FileMode::new(0o755)).ok();
    vfs.create_directory("/usr/bin", FileMode::new(0o755)).ok();
    vfs.create_directory("/usr/lib", FileMode::new(0o755)).ok();
    vfs.create_directory("/lib", FileMode::new(0o755)).ok();
    vfs.create_directory("/mnt", FileMode::new(0o755)).ok();
}

fn normalize_path(path: &str) -> String {
    let mut components: Vec<&str> = Vec::new();
    
    for component in path.split('/') {
        match component {
            "" | "." => continue,
            ".." => { components.pop(); }
            c => components.push(c),
        }
    }
    
    if components.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", components.join("/"))
    }
}
