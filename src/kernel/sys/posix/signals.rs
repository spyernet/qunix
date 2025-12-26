use crate::kernel::scheduler::{SCHEDULER, Pid};
use crate::fs::{FsResult, FsError};
use alloc::collections::BTreeMap;

pub const SIGHUP: i32 = 1;
pub const SIGINT: i32 = 2;
pub const SIGQUIT: i32 = 3;
pub const SIGILL: i32 = 4;
pub const SIGTRAP: i32 = 5;
pub const SIGABRT: i32 = 6;
pub const SIGBUS: i32 = 7;
pub const SIGFPE: i32 = 8;
pub const SIGKILL: i32 = 9;
pub const SIGUSR1: i32 = 10;
pub const SIGSEGV: i32 = 11;
pub const SIGUSR2: i32 = 12;
pub const SIGPIPE: i32 = 13;
pub const SIGALRM: i32 = 14;
pub const SIGTERM: i32 = 15;
pub const SIGSTKFLT: i32 = 16;
pub const SIGCHLD: i32 = 17;
pub const SIGCONT: i32 = 18;
pub const SIGSTOP: i32 = 19;
pub const SIGTSTP: i32 = 20;
pub const SIGTTIN: i32 = 21;
pub const SIGTTOU: i32 = 22;
pub const SIGURG: i32 = 23;
pub const SIGXCPU: i32 = 24;
pub const SIGXFSZ: i32 = 25;
pub const SIGVTALRM: i32 = 26;
pub const SIGPROF: i32 = 27;
pub const SIGWINCH: i32 = 28;
pub const SIGIO: i32 = 29;
pub const SIGPWR: i32 = 30;
pub const SIGSYS: i32 = 31;

pub const NSIG: i32 = 64;

pub const SIG_DFL: usize = 0;
pub const SIG_IGN: usize = 1;
pub const SIG_ERR: usize = usize::MAX;

pub const SA_NOCLDSTOP: u32 = 1;
pub const SA_NOCLDWAIT: u32 = 2;
pub const SA_SIGINFO: u32 = 4;
pub const SA_ONSTACK: u32 = 0x08000000;
pub const SA_RESTART: u32 = 0x10000000;
pub const SA_NODEFER: u32 = 0x40000000;
pub const SA_RESETHAND: u32 = 0x80000000;

pub const SIG_BLOCK: i32 = 0;
pub const SIG_UNBLOCK: i32 = 1;
pub const SIG_SETMASK: i32 = 2;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SigSet {
    pub bits: [u64; 1],
}

impl SigSet {
    pub fn new() -> Self {
        SigSet { bits: [0] }
    }
    
    pub fn full() -> Self {
        SigSet { bits: [!0] }
    }
    
    pub fn add(&mut self, sig: i32) {
        if sig > 0 && sig <= NSIG {
            self.bits[0] |= 1 << (sig - 1);
        }
    }
    
    pub fn del(&mut self, sig: i32) {
        if sig > 0 && sig <= NSIG {
            self.bits[0] &= !(1 << (sig - 1));
        }
    }
    
    pub fn is_member(&self, sig: i32) -> bool {
        if sig > 0 && sig <= NSIG {
            (self.bits[0] & (1 << (sig - 1))) != 0
        } else {
            false
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.bits[0] == 0
    }
    
    pub fn or(&mut self, other: &SigSet) {
        self.bits[0] |= other.bits[0];
    }
    
    pub fn and(&mut self, other: &SigSet) {
        self.bits[0] &= other.bits[0];
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SigAction {
    pub sa_handler: usize,
    pub sa_flags: u32,
    pub sa_restorer: usize,
    pub sa_mask: SigSet,
}

impl Default for SigAction {
    fn default() -> Self {
        SigAction {
            sa_handler: SIG_DFL,
            sa_flags: 0,
            sa_restorer: 0,
            sa_mask: SigSet::new(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SigInfo {
    pub si_signo: i32,
    pub si_errno: i32,
    pub si_code: i32,
    pub si_pid: i32,
    pub si_uid: u32,
    pub si_status: i32,
    pub si_addr: usize,
    pub si_value: usize,
}

impl Default for SigInfo {
    fn default() -> Self {
        SigInfo {
            si_signo: 0,
            si_errno: 0,
            si_code: 0,
            si_pid: 0,
            si_uid: 0,
            si_status: 0,
            si_addr: 0,
            si_value: 0,
        }
    }
}

pub fn posix_kill(pid: i32, sig: i32) -> FsResult<()> {
    if sig < 0 || sig > NSIG {
        return Err(FsError::InvalidArgument);
    }
    
    if sig == 0 {
        let scheduler = SCHEDULER.lock();
        if scheduler.get_task(pid as Pid).is_some() {
            return Ok(());
        } else {
            return Err(FsError::NotFound);
        }
    }
    
    if crate::kernel::scheduler::kill(pid as Pid, sig as u8) {
        Ok(())
    } else {
        Err(FsError::NotFound)
    }
}

pub fn posix_sigaction(sig: i32, act: Option<&SigAction>, oldact: Option<&mut SigAction>) -> FsResult<()> {
    if sig <= 0 || sig > NSIG {
        return Err(FsError::InvalidArgument);
    }
    
    if sig == SIGKILL || sig == SIGSTOP {
        return Err(FsError::InvalidArgument);
    }
    
    Ok(())
}

pub fn posix_sigprocmask(how: i32, set: Option<&SigSet>, oldset: Option<&mut SigSet>) -> FsResult<()> {
    let mut scheduler = SCHEDULER.lock();
    
    if let Some(task) = scheduler.current_mut() {
        if let Some(old) = oldset {
            old.bits[0] = task.signal_mask;
        }
        
        if let Some(s) = set {
            match how {
                SIG_BLOCK => task.signal_mask |= s.bits[0],
                SIG_UNBLOCK => task.signal_mask &= !s.bits[0],
                SIG_SETMASK => task.signal_mask = s.bits[0],
                _ => return Err(FsError::InvalidArgument),
            }
            
            task.signal_mask &= !((1 << (SIGKILL - 1)) | (1 << (SIGSTOP - 1)));
        }
        
        Ok(())
    } else {
        Err(FsError::InvalidArgument)
    }
}

pub fn posix_sigpending(set: &mut SigSet) -> FsResult<()> {
    let scheduler = SCHEDULER.lock();
    
    if let Some(task) = scheduler.current() {
        set.bits[0] = task.pending_signals & task.signal_mask;
        Ok(())
    } else {
        Err(FsError::InvalidArgument)
    }
}

pub fn posix_sigsuspend(_mask: &SigSet) -> FsResult<()> {
    Err(FsError::NotSupported)
}

pub fn posix_raise(sig: i32) -> FsResult<()> {
    let pid = crate::kernel::scheduler::current_pid().ok_or(FsError::InvalidArgument)?;
    posix_kill(pid as i32, sig)
}

pub fn posix_alarm(_seconds: u32) -> u32 {
    0
}

pub fn posix_pause() -> FsResult<()> {
    Err(FsError::NotSupported)
}

pub fn is_fatal_signal(sig: i32) -> bool {
    matches!(sig,
        SIGKILL | SIGTERM | SIGINT | SIGQUIT |
        SIGILL | SIGABRT | SIGFPE | SIGSEGV | SIGBUS
    )
}

pub fn is_stop_signal(sig: i32) -> bool {
    matches!(sig, SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU)
}

pub fn is_cont_signal(sig: i32) -> bool {
    sig == SIGCONT
}

pub fn default_action(sig: i32) -> DefaultAction {
    match sig {
        SIGCHLD | SIGURG | SIGWINCH => DefaultAction::Ignore,
        SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU => DefaultAction::Stop,
        SIGCONT => DefaultAction::Continue,
        SIGABRT | SIGBUS | SIGFPE | SIGILL | SIGQUIT | SIGSEGV |
        SIGSYS | SIGTRAP | SIGXCPU | SIGXFSZ => DefaultAction::CoreDump,
        _ => DefaultAction::Terminate,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultAction {
    Terminate,
    Ignore,
    CoreDump,
    Stop,
    Continue,
}
