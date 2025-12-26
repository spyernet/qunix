use crate::fs::{FsError};
use crate::kernel::scheduler::{SCHEDULER, Pid};
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use crate::fs::vfs::api as vfs_api;

pub const SYS_READ: u64 = 0;
pub const SYS_WRITE: u64 = 1;
pub const SYS_OPEN: u64 = 2;
pub const SYS_CLOSE: u64 = 3;
pub const SYS_STAT: u64 = 4;
pub const SYS_FSTAT: u64 = 5;
pub const SYS_LSTAT: u64 = 6;
pub const SYS_POLL: u64 = 7;
pub const SYS_LSEEK: u64 = 8;
pub const SYS_MMAP: u64 = 9;
pub const SYS_MPROTECT: u64 = 10;
pub const SYS_MUNMAP: u64 = 11;
pub const SYS_BRK: u64 = 12;
pub const SYS_IOCTL: u64 = 16;
pub const SYS_ACCESS: u64 = 21;
pub const SYS_PIPE: u64 = 22;
pub const SYS_DUP: u64 = 32;
pub const SYS_DUP2: u64 = 33;
pub const SYS_GETPID: u64 = 39;
pub const SYS_FORK: u64 = 57;
pub const SYS_VFORK: u64 = 58;
pub const SYS_EXECVE: u64 = 59;
pub const SYS_EXIT: u64 = 60;
pub const SYS_WAIT4: u64 = 61;
pub const SYS_KILL: u64 = 62;
pub const SYS_UNAME: u64 = 63;
pub const SYS_FCNTL: u64 = 72;
pub const SYS_FLOCK: u64 = 73;
pub const SYS_FSYNC: u64 = 74;
pub const SYS_GETCWD: u64 = 79;
pub const SYS_CHDIR: u64 = 80;
pub const SYS_FCHDIR: u64 = 81;
pub const SYS_RENAME: u64 = 82;
pub const SYS_MKDIR: u64 = 83;
pub const SYS_RMDIR: u64 = 84;
pub const SYS_CREAT: u64 = 85;
pub const SYS_LINK: u64 = 86;
pub const SYS_UNLINK: u64 = 87;
pub const SYS_SYMLINK: u64 = 88;
pub const SYS_READLINK: u64 = 89;
pub const SYS_CHMOD: u64 = 90;
pub const SYS_FCHMOD: u64 = 91;
pub const SYS_CHOWN: u64 = 92;
pub const SYS_FCHOWN: u64 = 93;
pub const SYS_UMASK: u64 = 95;
pub const SYS_GETUID: u64 = 102;
pub const SYS_GETGID: u64 = 104;
pub const SYS_SETUID: u64 = 105;
pub const SYS_SETGID: u64 = 106;
pub const SYS_GETEUID: u64 = 107;
pub const SYS_GETEGID: u64 = 108;
pub const SYS_GETPPID: u64 = 110;
pub const SYS_GETPGRP: u64 = 111;
pub const SYS_SETSID: u64 = 112;
pub const SYS_GETGROUPS: u64 = 115;
pub const SYS_SETGROUPS: u64 = 116;
pub const SYS_SIGACTION: u64 = 13;
pub const SYS_SIGPROCMASK: u64 = 14;
pub const SYS_SIGRETURN: u64 = 15;

#[derive(Debug)]
pub struct SyscallArgs {
    pub num: u64,
    pub arg1: u64,
    pub arg2: u64,
    pub arg3: u64,
    pub arg4: u64,
    pub arg5: u64,
    pub arg6: u64,
}

pub fn dispatch_syscall(args: &SyscallArgs) -> i64 {
    match args.num {
        SYS_READ => sys_read(args.arg1 as i32, args.arg2 as *mut u8, args.arg3 as usize),
        SYS_WRITE => sys_write(args.arg1 as i32, args.arg2 as *const u8, args.arg3 as usize),
        SYS_OPEN => sys_open(args.arg1 as *const u8, args.arg2 as i32, args.arg3 as u32),
        SYS_CLOSE => sys_close(args.arg1 as i32),
        SYS_LSEEK => sys_lseek(args.arg1 as i32, args.arg2 as i64, args.arg3 as i32),
        SYS_GETPID => sys_getpid(),
        SYS_GETPPID => sys_getppid(),
        SYS_GETUID => sys_getuid(),
        SYS_GETEUID => sys_geteuid(),
        SYS_GETGID => sys_getgid(),
        SYS_GETEGID => sys_getegid(),
        SYS_FORK => sys_fork(),
        SYS_EXIT => sys_exit(args.arg1 as i32),
        SYS_EXECVE => sys_execve(args.arg1 as *const u8, args.arg2 as *const *const u8, args.arg3 as *const *const u8),
        SYS_WAIT4 => sys_wait4(args.arg1 as i32, args.arg2 as *mut i32, args.arg3 as i32, args.arg4 as *const u8),
        SYS_KILL => sys_kill(args.arg1 as i32, args.arg2 as i32),
        SYS_GETCWD => sys_getcwd(args.arg1 as *mut u8, args.arg2 as usize),
        SYS_CHDIR => sys_chdir(args.arg1 as *const u8),
        SYS_MKDIR => sys_mkdir(args.arg1 as *const u8, args.arg2 as u32),
        SYS_RMDIR => sys_rmdir(args.arg1 as *const u8),
        SYS_UNLINK => sys_unlink(args.arg1 as *const u8),
        SYS_STAT => sys_stat(args.arg1 as *const u8, args.arg2 as *mut u8),
        SYS_FSTAT => sys_fstat(args.arg1 as i32, args.arg2 as *mut u8),
        SYS_CHMOD => sys_chmod(args.arg1 as *const u8, args.arg2 as u32),
        SYS_FCHMOD => sys_fchmod(args.arg1 as i32, args.arg2 as u32),
        SYS_CHOWN => sys_chown(args.arg1 as *const u8, args.arg2 as u32, args.arg3 as u32),
        SYS_FCHOWN => sys_fchown(args.arg1 as i32, args.arg2 as u32, args.arg3 as u32),
        SYS_UMASK => sys_umask(args.arg1 as u32),
        SYS_PIPE => sys_pipe(args.arg1 as *mut i32),
        SYS_DUP => sys_dup(args.arg1 as i32),
        SYS_DUP2 => sys_dup2(args.arg1 as i32, args.arg2 as i32),
        _ => -38,  // ENOSYS
    }
}

fn sys_read(fd: i32, buf: *mut u8, count: usize) -> i64 {
    if buf.is_null() {
        return -14;
    }
    
    let mut scheduler = SCHEDULER.lock();
    if let Some(task) = scheduler.current_mut() {
        if let Some(fd_entry) = task.get_fd_mut(fd) {
            // Build buffer
            let slice = unsafe { core::slice::from_raw_parts_mut(buf, count) };
            // Lookup node
            let vfs = crate::fs::vfs::vfs::VFS.lock();
            match vfs.lookup_path(&fd_entry.path) {
                Ok(node) => {
                    match node.read(fd_entry.offset, slice) {
                        Ok(bytes_read) => {
                            fd_entry.offset += bytes_read as u64;
                            return bytes_read as i64;
                        }
                        Err(e) => return fs_error_to_errno(e),
                    }
                }
                Err(e) => return fs_error_to_errno(e),
            }
        }
    }

    -9
}

fn sys_write(fd: i32, buf: *const u8, count: usize) -> i64 {
    if buf.is_null() {
        return -14;  // EFAULT
    }
    
    // stdout/stderr: write directly
    if fd == 1 || fd == 2 {
        let slice = unsafe { core::slice::from_raw_parts(buf, count) };
        if let Ok(s) = core::str::from_utf8(slice) {
            crate::print!("{}", s);
        } else {
            // Write binary data
            for &byte in slice {
                crate::print!("{}", byte as char);
            }
        }
        return count as i64;
    }
    
    // For other fds: check if exists in task's fd table and write via VFS or device
    let mut scheduler = SCHEDULER.lock();
    if let Some(task) = scheduler.current_mut() {
        if let Some(fd_entry) = task.get_fd_mut(fd) {
            let path = fd_entry.path.clone();
            let offset = fd_entry.offset;
            let slice = unsafe { core::slice::from_raw_parts(buf, count) };
            
            let mut vfs = crate::fs::vfs::vfs::VFS.lock();
            // Get inode first, then drop the immutable borrow
            let inode = match vfs.lookup_path(&path) {
                Ok(node) => node.inode,
                Err(e) => return fs_error_to_errno(e),
            };
            
            match vfs.write_node(inode, offset, slice) {
                Ok(written) => {
                    fd_entry.offset += written as u64;
                    return written as i64;
                }
                Err(e) => return fs_error_to_errno(e),
            }
        }
    }

    -9  // EBADF
}

fn sys_open(_pathname: *const u8, _flags: i32, _mode: u32) -> i64 {
    if _pathname.is_null() {
        return -14; // EFAULT
    }

    // Extract path string
    let path_vec = unsafe {
        let mut bytes = Vec::new();
        let mut ptr = _pathname;
        while *ptr != 0 {
            bytes.push(*ptr);
            ptr = ptr.add(1);
            if bytes.len() > 4096 { break; }
        }
        bytes
    };

    let path = match core::str::from_utf8(&path_vec) {
        Ok(s) => s.to_string(),
        Err(_) => return -14,
    };

    // Validate via VFS open
    let open_flags = vfs_api::OpenFlags::from_bits_truncate(_flags as u32);
    match crate::fs::vfs::api::open(&path, open_flags, _mode as u16) {
        Ok(_) => {
            let mut scheduler = SCHEDULER.lock();
            if let Some(task) = scheduler.current_mut() {
                let newfd = task.allocate_fd();
                task.fds.insert(newfd, crate::kernel::scheduler::task::FileDescriptor {
                    fd: newfd,
                    path: path.clone(),
                    offset: 0,
                    flags: _flags as u32,
                });
                return newfd as i64;
            }
            -3
        }
        Err(e) => fs_error_to_errno(e),
    }
}

fn sys_close(fd: i32) -> i64 {
    let mut scheduler = SCHEDULER.lock();
    if let Some(task) = scheduler.current_mut() {
        if task.close_fd(fd) {
            return 0;
        }
    }
    -9
}

fn sys_lseek(fd: i32, offset: i64, whence: i32) -> i64 {
    let mut scheduler = SCHEDULER.lock();
    if let Some(task) = scheduler.current_mut() {
        if let Some(fd_entry) = task.get_fd_mut(fd) {
            match whence {
                0 => fd_entry.offset = offset as u64,  // SEEK_SET
                1 => fd_entry.offset = (fd_entry.offset as i64 + offset) as u64,  // SEEK_CUR
                2 => fd_entry.offset = (10000 + offset) as u64,  // SEEK_END (stub file size)
                _ => return -22,  // EINVAL
            }
            return fd_entry.offset as i64;
        }
    }
    -9  // EBADF
}

fn sys_getpid() -> i64 {
    SCHEDULER.lock().current_pid().map_or(-1, |pid| pid as i64)
}

fn sys_getppid() -> i64 {
    let scheduler = SCHEDULER.lock();
    if let Some(task) = scheduler.current() {
        task.ppid.map_or(1, |pid| pid as i64)
    } else {
        1
    }
}

fn sys_getuid() -> i64 {
    let scheduler = SCHEDULER.lock();
    if let Some(task) = scheduler.current() {
        task.uid as i64
    } else {
        0
    }
}

fn sys_geteuid() -> i64 {
    let scheduler = SCHEDULER.lock();
    if let Some(task) = scheduler.current() {
        task.euid as i64
    } else {
        0
    }
}

fn sys_getgid() -> i64 {
    let scheduler = SCHEDULER.lock();
    if let Some(task) = scheduler.current() {
        task.gid as i64
    } else {
        0
    }
}

fn sys_getegid() -> i64 {
    let scheduler = SCHEDULER.lock();
    if let Some(task) = scheduler.current() {
        task.egid as i64
    } else {
        0
    }
}

fn sys_fork() -> i64 {
    let mut scheduler = SCHEDULER.lock();
    
    // Get the current task and clone it BEFORE calling allocate_pid
    let cloned_parent = if let Some(parent_task) = scheduler.current() {
        parent_task.clone()
    } else {
        return -3;  // ESRCH (no such process)
    };
    
    // Now allocate PID (this doesn't conflict with the clone)
    let child_pid = scheduler.allocate_pid();
    
    // Clone the parent task as child
    match cloned_parent.fork(child_pid) {
        Ok(child_task) => {
            // Add child to scheduler
            scheduler.add_task(child_task);
            
            // Update parent's children list
            if let Some(parent) = scheduler.current_mut() {
                parent.children.push(child_pid);
            }
            
            // Parent returns child PID
            child_pid as i64
        }
        Err(_) => -12,  // ENOMEM
    }
}

fn sys_exit(code: i32) -> i64 {
    crate::kernel::scheduler::exit(code);
    0
}

fn sys_kill(pid: i32, sig: i32) -> i64 {
    if crate::kernel::scheduler::kill(pid as Pid, sig as u8) {
        0
    } else {
        -3
    }
}

fn sys_getcwd(buf: *mut u8, size: usize) -> i64 {
    if buf.is_null() || size == 0 {
        return -14;
    }
    
    let vfs = crate::fs::vfs::vfs::VFS.lock();
    let cwd = vfs.get_cwd();
    
    if cwd.len() + 1 > size {
        return -34;
    }
    
    unsafe {
        core::ptr::copy_nonoverlapping(cwd.as_ptr(), buf, cwd.len());
        *buf.add(cwd.len()) = 0;
    }
    
    cwd.len() as i64
}

fn sys_chdir(pathname: *const u8) -> i64 {
    if pathname.is_null() {
        return -14;  // EFAULT
    }
    
    let mut scheduler = SCHEDULER.lock();
    if let Some(task) = scheduler.current_mut() {
        // Extract path string
        let path_bytes = unsafe {
            let mut bytes = Vec::new();
            let mut ptr = pathname;
            while *ptr != 0 && bytes.len() < 256 {
                bytes.push(*ptr);
                ptr = ptr.add(1);
            }
            bytes
        };
        
        if let Ok(path_str) = core::str::from_utf8(&path_bytes) {
            task.cwd = path_str.to_string();
            return 0;
        }
    }
    
    -3  // ESRCH
}

fn sys_mkdir(pathname: *const u8, _mode: u32) -> i64 {
    if pathname.is_null() {
        return -14;
    }
    // Extract path
    let path_vec = unsafe {
        let mut bytes = Vec::new();
        let mut ptr = pathname;
        while *ptr != 0 {
            bytes.push(*ptr);
            ptr = ptr.add(1);
            if bytes.len() > 4096 { break; }
        }
        bytes
    };

    let path = match core::str::from_utf8(&path_vec) {
        Ok(s) => s,
        Err(_) => return -14,
    };

    match crate::fs::vfs::api::mkdir(path, _mode as u16) {
        Ok(()) => 0,
        Err(e) => fs_error_to_errno(e),
    }
}

fn sys_rmdir(pathname: *const u8) -> i64 {
    if pathname.is_null() {
        return -14;
    }
    let path_vec = unsafe {
        let mut bytes = Vec::new();
        let mut ptr = pathname;
        while *ptr != 0 {
            bytes.push(*ptr);
            ptr = ptr.add(1);
            if bytes.len() > 4096 { break; }
        }
        bytes
    };

    let path = match core::str::from_utf8(&path_vec) {
        Ok(s) => s,
        Err(_) => return -14,
    };

    match crate::fs::vfs::api::rmdir(path) {
        Ok(()) => 0,
        Err(e) => fs_error_to_errno(e),
    }
}

fn sys_unlink(pathname: *const u8) -> i64 {
    if pathname.is_null() {
        return -14;
    }
    let path_vec = unsafe {
        let mut bytes = Vec::new();
        let mut ptr = pathname;
        while *ptr != 0 {
            bytes.push(*ptr);
            ptr = ptr.add(1);
            if bytes.len() > 4096 { break; }
        }
        bytes
    };

    let path = match core::str::from_utf8(&path_vec) {
        Ok(s) => s,
        Err(_) => return -14,
    };

    match crate::fs::vfs::api::unlink(path) {
        Ok(()) => 0,
        Err(e) => fs_error_to_errno(e),
    }
}

fn sys_execve(pathname: *const u8, argv: *const *const u8, envp: *const *const u8) -> i64 {
    if pathname.is_null() {
        return -14;  // EFAULT
    }
    
    // Extract program name from pathname
    let prog_name_vec = unsafe {
        let mut bytes = Vec::new();
        let mut ptr = pathname as *const u8;
        while *ptr != 0 {
            bytes.push(*ptr);
            ptr = ptr.add(1);
        }
        bytes
    };
    
    if prog_name_vec.is_empty() {
        return -2;  // ENOENT
    }
    
    let prog_name = String::from_utf8_lossy(&prog_name_vec).to_string();
    
    // Update current task's name and entry point
    let mut scheduler = SCHEDULER.lock();
    if let Some(task) = scheduler.current_mut() {
        task.name = prog_name;
        // In a real implementation, we'd load the ELF binary, set up memory, and jump to entry point
        // For now, this is a stub
        return 0;
    }
    
    -3  // ESRCH
}

fn sys_wait4(pid: i32, status: *mut i32, flags: i32, _rusage: *const u8) -> i64 {
    let mut scheduler = SCHEDULER.lock();
    
    let target_pid = if pid == -1 {
        // Wait for any child
        if let Some(task) = scheduler.current() {
            task.children.first().copied()
        } else {
            None
        }
    } else if pid > 0 {
        Some(pid as Pid)
    } else {
        return -22;  // EINVAL
    };
    
    if let Some(tpid) = target_pid {
        // Check if child exists and is a zombie
        if let Some(child) = scheduler.get_task(tpid) {
            if child.state == crate::kernel::scheduler::task::TaskState::Zombie {
                let exit_code = child.exit_code.unwrap_or(0);
                
                // Store exit status if pointer provided
                if !status.is_null() {
                    unsafe {
                        *status = exit_code;
                    }
                }
                
                // Remove zombie task
                scheduler.tasks.retain(|t| t.pid != tpid);
                
                return tpid as i64;
            }
        }
    }
    
    -10  // ECHILD (no child process)
}

fn sys_stat(_pathname: *const u8, _stat_buf: *mut u8) -> i64 {
    if _pathname.is_null() || _stat_buf.is_null() {
        return -14;
    }

    // extract path
    let path_vec = unsafe {
        let mut bytes = Vec::new();
        let mut ptr = _pathname;
        while *ptr != 0 {
            bytes.push(*ptr);
            ptr = ptr.add(1);
            if bytes.len() > 4096 { break; }
        }
        bytes
    };

    let path = match core::str::from_utf8(&path_vec) {
        Ok(s) => s,
        Err(_) => return -14,
    };

    match crate::kernel::sys::posix::posix_stat(path) {
        Ok(posix_stat) => {
            // copy PosixStat bytes into buffer (caller expects struct)
            let src = &posix_stat as *const crate::kernel::sys::posix::PosixStat as *const u8;
            let size = core::mem::size_of::<crate::kernel::sys::posix::PosixStat>();
            unsafe { core::ptr::copy_nonoverlapping(src, _stat_buf, size); }
            0
        }
        Err(e) => fs_error_to_errno(e),
    }
}

fn sys_fstat(_fd: i32, _stat_buf: *mut u8) -> i64 {
    if _stat_buf.is_null() {
        return -14;
    }

    let mut scheduler = SCHEDULER.lock();
    if let Some(task) = scheduler.current_mut() {
        if let Some(fd_entry) = task.get_fd(_fd) {
            match crate::fs::vfs::api::stat(&fd_entry.path) {
                Ok(stat) => {
                    let pos = crate::kernel::sys::posix::PosixStat::from(stat);
                    let src = &pos as *const crate::kernel::sys::posix::PosixStat as *const u8;
                    let size = core::mem::size_of::<crate::kernel::sys::posix::PosixStat>();
                    unsafe { core::ptr::copy_nonoverlapping(src, _stat_buf, size); }
                    return 0;
                }
                Err(e) => return fs_error_to_errno(e),
            }
        }
    }

    -9
}

fn sys_chmod(_pathname: *const u8, _mode: u32) -> i64 {
    -38
}

fn sys_fchmod(_fd: i32, _mode: u32) -> i64 {
    -38
}

fn sys_chown(_pathname: *const u8, _uid: u32, _gid: u32) -> i64 {
    -38
}

fn sys_fchown(_fd: i32, _uid: u32, _gid: u32) -> i64 {
    -38
}

fn sys_umask(mask: u32) -> i64 {
    let mut scheduler = SCHEDULER.lock();
    if let Some(task) = scheduler.current_mut() {
        let old_mask = task.umask;
        task.umask = mask;
        old_mask as i64
    } else {
        -3
    }
}

fn sys_pipe(_pipefd: *mut i32) -> i64 {
    -38
}

fn sys_dup(oldfd: i32) -> i64 {
    let mut scheduler = SCHEDULER.lock();
    if let Some(task) = scheduler.current_mut() {
        if let Some(fd) = task.get_fd(oldfd) {
            let descriptor = fd.clone();
            let newfd = task.allocate_fd();
            task.fds.insert(newfd, descriptor);
            return newfd as i64;
        }
    }
    -9  // EBADF
}

fn sys_dup2(oldfd: i32, newfd: i32) -> i64 {
    let mut scheduler = SCHEDULER.lock();
    if let Some(task) = scheduler.current_mut() {
        if let Some(fd) = task.get_fd(oldfd) {
            let descriptor = fd.clone();
            task.fds.insert(newfd, descriptor);
            return newfd as i64;
        }
    }
    -9  // EBADF
}

pub const EPERM: i32 = 1;
pub const ENOENT: i32 = 2;
pub const ESRCH: i32 = 3;
pub const EINTR: i32 = 4;
pub const EIO: i32 = 5;
pub const ENXIO: i32 = 6;
pub const E2BIG: i32 = 7;
pub const ENOEXEC: i32 = 8;
pub const EBADF: i32 = 9;
pub const ECHILD: i32 = 10;
pub const EAGAIN: i32 = 11;
pub const ENOMEM: i32 = 12;
pub const EACCES: i32 = 13;
pub const EFAULT: i32 = 14;
pub const EBUSY: i32 = 16;
pub const EEXIST: i32 = 17;
pub const EXDEV: i32 = 18;
pub const ENODEV: i32 = 19;
pub const ENOTDIR: i32 = 20;
pub const EISDIR: i32 = 21;
pub const EINVAL: i32 = 22;
pub const ENFILE: i32 = 23;
pub const EMFILE: i32 = 24;
pub const ENOTTY: i32 = 25;
pub const ETXTBSY: i32 = 26;
pub const EFBIG: i32 = 27;
pub const ENOSPC: i32 = 28;
pub const ESPIPE: i32 = 29;
pub const EROFS: i32 = 30;
pub const EMLINK: i32 = 31;
pub const EPIPE: i32 = 32;
pub const ERANGE: i32 = 34;
pub const ENOSYS: i32 = 38;
pub const ENOTEMPTY: i32 = 39;
pub const ELOOP: i32 = 40;
fn fs_error_to_errno(e: FsError) -> i64 {
    match e {
        FsError::NotFound => -2,
        FsError::PermissionDenied => -13,
        FsError::AlreadyExists => -17,
        FsError::NotDirectory => -20,
        FsError::IsDirectory => -21,
        FsError::NotEmpty => -39,
        FsError::InvalidPath => -22,
        FsError::InvalidArgument => -22,
        FsError::IoError => -5,
        FsError::NoSpace => -28,
        FsError::ReadOnly => -30,
        FsError::TooManyLinks => -31,
        FsError::NameTooLong => -36,
        _ => -38,
    }
}
