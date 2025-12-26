use crate::kernel::scheduler::Scheduler;
use crate::kernel::scheduler::SCHEDULER;
use crate::kernel::scheduler::Pid;
use crate::fs::FsError;
use crate::fs::FsResult;
use crate::kernel::scheduler::TaskState;


pub fn posix_fork() -> FsResult<Pid> {
    Err(FsError::NotSupported)
}

pub fn posix_vfork() -> FsResult<Pid> {
    Err(FsError::NotSupported)
}

pub fn posix_execve(_pathname: &str, _argv: &[&str], _envp: &[&str]) -> FsResult<()> {
    Err(FsError::NotSupported)
}

pub fn posix_exit(status: i32) -> ! {
    crate::kernel::scheduler::exit(status);
    crate::hlt_loop()
}

pub fn posix_wait(status: &mut i32) -> FsResult<Pid> {
    posix_waitpid(-1, status, 0)
}

pub fn posix_waitpid(pid: i32, status: &mut i32, _options: i32) -> FsResult<Pid> {
    let scheduler = SCHEDULER.lock();
    
    let current = scheduler.current().ok_or(FsError::InvalidArgument)?;
    
    for child_pid in &current.children {
        if pid == -1 || pid as Pid == *child_pid {
            if let Some(child) = scheduler.get_task(*child_pid) {
                if child.state == TaskState::Zombie {
                    if let Some(code) = child.exit_code {
                        *status = code << 8;
                        return Ok(*child_pid);
                    }
                }
            }
        }
    }
    
    Err(FsError::InvalidArgument)
}

pub fn posix_getpid() -> Pid {
    SCHEDULER.lock().current_pid().unwrap_or(0)
}

pub fn posix_getppid() -> Pid {
    let scheduler = SCHEDULER.lock();
    scheduler.current()
        .and_then(|t| t.ppid)
        .unwrap_or(1)
}

pub fn posix_getuid() -> u32 {
    SCHEDULER.lock().current().map_or(0, |t| t.uid)
}

pub fn posix_geteuid() -> u32 {
    SCHEDULER.lock().current().map_or(0, |t| t.euid)
}

pub fn posix_getgid() -> u32 {
    SCHEDULER.lock().current().map_or(0, |t| t.gid)
}

pub fn posix_getegid() -> u32 {
    SCHEDULER.lock().current().map_or(0, |t| t.egid)
}

pub fn posix_setuid(uid: u32) -> FsResult<()> {
    let mut scheduler = SCHEDULER.lock();
    if let Some(task) = scheduler.current_mut() {
        if task.euid == 0 {
            task.uid = uid;
            task.euid = uid;
            Ok(())
        } else if uid == task.uid || uid == task.euid {
            task.euid = uid;
            Ok(())
        } else {
            Err(FsError::PermissionDenied)
        }
    } else {
        Err(FsError::InvalidArgument)
    }
}

pub fn posix_setgid(gid: u32) -> FsResult<()> {
    let mut scheduler = SCHEDULER.lock();
    if let Some(task) = scheduler.current_mut() {
        if task.euid == 0 {
            task.gid = gid;
            task.egid = gid;
            Ok(())
        } else if gid == task.gid || gid == task.egid {
            task.egid = gid;
            Ok(())
        } else {
            Err(FsError::PermissionDenied)
        }
    } else {
        Err(FsError::InvalidArgument)
    }
}

pub fn posix_setsid() -> FsResult<Pid> {
    Err(FsError::NotSupported)
}

pub fn posix_getpgid(_pid: Pid) -> FsResult<Pid> {
    Err(FsError::NotSupported)
}

pub fn posix_setpgid(_pid: Pid, _pgid: Pid) -> FsResult<()> {
    Err(FsError::NotSupported)
}

pub fn posix_getpgrp() -> Pid {
    posix_getpid()
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RUsage {
    pub ru_utime: TimeVal,
    pub ru_stime: TimeVal,
    pub ru_maxrss: i64,
    pub ru_ixrss: i64,
    pub ru_idrss: i64,
    pub ru_isrss: i64,
    pub ru_minflt: i64,
    pub ru_majflt: i64,
    pub ru_nswap: i64,
    pub ru_inblock: i64,
    pub ru_oublock: i64,
    pub ru_msgsnd: i64,
    pub ru_msgrcv: i64,
    pub ru_nsignals: i64,
    pub ru_nvcsw: i64,
    pub ru_nivcsw: i64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TimeVal {
    pub tv_sec: i64,
    pub tv_usec: i64,
}

impl Default for RUsage {
    fn default() -> Self {
        RUsage {
            ru_utime: TimeVal::default(),
            ru_stime: TimeVal::default(),
            ru_maxrss: 0,
            ru_ixrss: 0,
            ru_idrss: 0,
            ru_isrss: 0,
            ru_minflt: 0,
            ru_majflt: 0,
            ru_nswap: 0,
            ru_inblock: 0,
            ru_oublock: 0,
            ru_msgsnd: 0,
            ru_msgrcv: 0,
            ru_nsignals: 0,
            ru_nvcsw: 0,
            ru_nivcsw: 0,
        }
    }
}

pub fn posix_getrusage(_who: i32) -> RUsage {
    let mut rusage = RUsage::default();
    
    if let Some(task) = SCHEDULER.lock().current() {
        let ms = task.cpu_time;
        rusage.ru_utime.tv_sec = (ms / 1000) as i64;
        rusage.ru_utime.tv_usec = ((ms % 1000) * 1000) as i64;
    }
    
    rusage
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Utsname {
    pub sysname: [u8; 65],
    pub nodename: [u8; 65],
    pub release: [u8; 65],
    pub version: [u8; 65],
    pub machine: [u8; 65],
    pub domainname: [u8; 65],
}

impl Default for Utsname {
    fn default() -> Self {
        let mut uname = Utsname {
            sysname: [0; 65],
            nodename: [0; 65],
            release: [0; 65],
            version: [0; 65],
            machine: [0; 65],
            domainname: [0; 65],
        };
        
        copy_str(&mut uname.sysname, "Qunix");
        copy_str(&mut uname.nodename, "qunix");
        copy_str(&mut uname.release, crate::QUNIX_VERSION);
        copy_str(&mut uname.version, "#1 SMP Qunix");
        copy_str(&mut uname.machine, "x86_64");
        copy_str(&mut uname.domainname, "(none)");
        
        uname
    }
}

fn copy_str(dest: &mut [u8], src: &str) {
    let bytes = src.as_bytes();
    let len = core::cmp::min(bytes.len(), dest.len() - 1);
    dest[..len].copy_from_slice(&bytes[..len]);
    dest[len] = 0;
}

pub fn posix_uname() -> Utsname {
    Utsname::default()
}

pub const WNOHANG: i32 = 1;
pub const WUNTRACED: i32 = 2;
pub const WCONTINUED: i32 = 8;

pub const RUSAGE_SELF: i32 = 0;
pub const RUSAGE_CHILDREN: i32 = -1;
pub const RUSAGE_THREAD: i32 = 1;
