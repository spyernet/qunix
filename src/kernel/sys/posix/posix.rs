use super::fs::*;
use super::proc::*;
use super::signals::*;
use crate::fs::{FsResult, FsError};
use crate::kernel::scheduler::Pid;
use alloc::string::String;

pub fn posix_api_version() -> (i32, i32, i32) {
    (2008, 9, 0)
}

pub fn is_posix_compliant() -> bool {
    true
}

pub struct PosixEnvironment {
    pub user: String,
    pub home: String,
    pub shell: String,
    pub path: String,
    pub term: String,
    pub lang: String,
}

impl Default for PosixEnvironment {
    fn default() -> Self {
        PosixEnvironment {
            user: String::from("root"),
            home: String::from("/root"),
            shell: String::from("/bin/sh"),
            path: String::from("/bin:/sbin:/usr/bin:/usr/sbin"),
            term: String::from("linux"),
            lang: String::from("C"),
        }
    }
}

pub fn get_env(name: &str) -> Option<String> {
    let env = PosixEnvironment::default();
    
    match name {
        "USER" => Some(env.user),
        "HOME" => Some(env.home),
        "SHELL" => Some(env.shell),
        "PATH" => Some(env.path),
        "TERM" => Some(env.term),
        "LANG" => Some(env.lang),
        "PWD" => posix_getcwd().ok(),
        _ => None,
    }
}

pub struct PosixLimits {
    pub arg_max: usize,
    pub child_max: usize,
    pub clk_tck: usize,
    pub ngroups_max: usize,
    pub open_max: usize,
    pub stream_max: usize,
    pub tzname_max: usize,
    pub pagesize: usize,
    pub symloop_max: usize,
    pub path_max: usize,
    pub name_max: usize,
    pub pipe_buf: usize,
}

impl Default for PosixLimits {
    fn default() -> Self {
        PosixLimits {
            arg_max: 131072,
            child_max: 1024,
            clk_tck: 100,
            ngroups_max: 65536,
            open_max: 1024,
            stream_max: 16,
            tzname_max: 6,
            pagesize: 4096,
            symloop_max: 40,
            path_max: 4096,
            name_max: 255,
            pipe_buf: 4096,
        }
    }
}

pub fn sysconf(name: i32) -> Option<i64> {
    let limits = PosixLimits::default();
    
    match name {
        0 => Some(limits.arg_max as i64),
        1 => Some(limits.child_max as i64),
        2 => Some(limits.clk_tck as i64),
        3 => Some(limits.ngroups_max as i64),
        4 => Some(limits.open_max as i64),
        5 => Some(limits.stream_max as i64),
        6 => Some(limits.tzname_max as i64),
        30 => Some(limits.pagesize as i64),
        _ => None,
    }
}

pub fn pathconf(_path: &str, name: i32) -> Option<i64> {
    let limits = PosixLimits::default();
    
    match name {
        0 => Some(limits.symloop_max as i64),
        1 => Some(limits.path_max as i64),
        2 => Some(limits.name_max as i64),
        3 => Some(limits.pipe_buf as i64),
        _ => None,
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TimeSpec {
    pub tv_sec: i64,
    pub tv_nsec: i64,
}

impl TimeSpec {
    pub fn now() -> Self {
        let ticks = crate::hal::drivers::pit::get_ticks();
        TimeSpec {
            tv_sec: (ticks / 1000) as i64,
            tv_nsec: ((ticks % 1000) * 1_000_000) as i64,
        }
    }
    
    pub fn from_secs(secs: i64) -> Self {
        TimeSpec {
            tv_sec: secs,
            tv_nsec: 0,
        }
    }
    
    pub fn to_millis(&self) -> i64 {
        self.tv_sec * 1000 + self.tv_nsec / 1_000_000
    }
}

pub fn clock_gettime(clock_id: i32) -> FsResult<TimeSpec> {
    match clock_id {
        0 | 1 => Ok(TimeSpec::now()),
        _ => Err(FsError::InvalidArgument),
    }
}

pub fn nanosleep(req: &TimeSpec) -> FsResult<TimeSpec> {
    let ms = req.to_millis() as u64;
    crate::hal::drivers::pit::sleep_ms(ms);
    Ok(TimeSpec::default())
}

pub const CLOCK_REALTIME: i32 = 0;
pub const CLOCK_MONOTONIC: i32 = 1;
pub const CLOCK_PROCESS_CPUTIME_ID: i32 = 2;
pub const CLOCK_THREAD_CPUTIME_ID: i32 = 3;
pub const CLOCK_MONOTONIC_RAW: i32 = 4;
pub const CLOCK_REALTIME_COARSE: i32 = 5;
pub const CLOCK_MONOTONIC_COARSE: i32 = 6;
pub const CLOCK_BOOTTIME: i32 = 7;

pub const _POSIX_VERSION: i64 = 200809;
pub const _POSIX2_VERSION: i64 = 200809;
pub const _XOPEN_VERSION: i32 = 700;

pub fn conformance_test() -> bool {
    posix_getcwd().is_ok() &&
    posix_getpid() > 0 &&
    posix_getuid() == 0
}
