use crate::fs::{FsResult, FsError};
use crate::kernel::scheduler::{SCHEDULER, Pid};
use alloc::string::String;
use alloc::vec::Vec;

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
        SYS_KILL => sys_kill(args.arg1 as i32, args.arg2 as i32),
        SYS_GETCWD => sys_getcwd(args.arg1 as *mut u8, args.arg2 as usize),
        SYS_CHDIR => sys_chdir(args.arg1 as *const u8),
        SYS_MKDIR => sys_mkdir(args.arg1 as *const u8, args.arg2 as u32),
        SYS_RMDIR => sys_rmdir(args.arg1 as *const u8),
        SYS_UNLINK => sys_unlink(args.arg1 as *const u8),
        _ => -38,
    }
}

fn sys_read(fd: i32, buf: *mut u8, count: usize) -> i64 {
    if buf.is_null() {
        return -14;
    }
    
    let scheduler = SCHEDULER.lock();
    if let Some(task) = scheduler.current() {
        if task.fds.contains_key(&fd) {
            return count as i64;
        }
    }
    
    -9
}

fn sys_write(fd: i32, buf: *const u8, count: usize) -> i64 {
    if buf.is_null() {
        return -14;
    }
    
    if fd == 1 || fd == 2 {
        let slice = unsafe { core::slice::from_raw_parts(buf, count) };
        if let Ok(s) = core::str::from_utf8(slice) {
            crate::print!("{}", s);
        }
        return count as i64;
    }
    
    let scheduler = SCHEDULER.lock();
    if let Some(task) = scheduler.current() {
        if task.fds.contains_key(&fd) {
            return count as i64;
        }
    }
    
    -9
}

fn sys_open(_pathname: *const u8, _flags: i32, _mode: u32) -> i64 {
    -38
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

fn sys_lseek(_fd: i32, _offset: i64, _whence: i32) -> i64 {
    -38
}

fn sys_getpid() -> i64 {
    SCHEDULER.lock().current_pid().map_or(-1, |pid| pid as i64)
}

fn sys_getppid() -> i64 {
    let scheduler = SCHEDULER.lock();
    if let Some(task) = scheduler.current() {
        task.parent_pid.map_or(1, |pid| pid as i64)
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
    -38
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

fn sys_chdir(_pathname: *const u8) -> i64 {
    -38
}

fn sys_mkdir(_pathname: *const u8, _mode: u32) -> i64 {
    -38
}

fn sys_rmdir(_pathname: *const u8) -> i64 {
    -38
}

fn sys_unlink(_pathname: *const u8) -> i64 {
    -38
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
