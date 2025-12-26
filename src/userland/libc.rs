// Minimal libc for Qunix userland
//
// Provides basic C-compatible syscall wrappers and standard library functions

use core::ffi::c_char;
use core::ptr;

// ============== Syscall numbers (x86_64) ==============
pub const SYS_READ: u64 = 0;
pub const SYS_WRITE: u64 = 1;
pub const SYS_OPEN: u64 = 2;
pub const SYS_CLOSE: u64 = 3;
pub const SYS_STAT: u64 = 4;
pub const SYS_FSTAT: u64 = 5;
pub const SYS_LSEEK: u64 = 8;
pub const SYS_PIPE: u64 = 22;
pub const SYS_GETPID: u64 = 39;
pub const SYS_FORK: u64 = 57;
pub const SYS_EXECVE: u64 = 59;
pub const SYS_EXIT: u64 = 60;
pub const SYS_WAIT4: u64 = 61;
pub const SYS_KILL: u64 = 62;
pub const SYS_CHMOD: u64 = 90;
pub const SYS_CHOWN: u64 = 92;
pub const SYS_GETUID: u64 = 102;
pub const SYS_GETGID: u64 = 104;
pub const SYS_SETUID: u64 = 105;
pub const SYS_SETGID: u64 = 106;
pub const SYS_GETPPID: u64 = 110;
pub const SYS_DUP: u64 = 32;
pub const SYS_DUP2: u64 = 33;
pub const SYS_GETCWD: u64 = 79;
pub const SYS_CHDIR: u64 = 80;
pub const SYS_MKDIR: u64 = 83;
pub const SYS_RMDIR: u64 = 84;
pub const SYS_UNLINK: u64 = 87;

// File descriptor constants
pub const STDIN_FILENO: i32 = 0;
pub const STDOUT_FILENO: i32 = 1;
pub const STDERR_FILENO: i32 = 2;

// Open flags (from x86_64 ABI)
pub const O_RDONLY: i32 = 0;
pub const O_WRONLY: i32 = 1;
pub const O_RDWR: i32 = 2;
pub const O_CREAT: i32 = 0o100;
pub const O_EXCL: i32 = 0o200;
pub const O_TRUNC: i32 = 0o1000;
pub const O_APPEND: i32 = 0o2000;

// Error constants (POSIX errno values)
pub const EPERM: i32 = 1;
pub const ENOENT: i32 = 2;
pub const ESRCH: i32 = 3;
pub const EINTR: i32 = 4;
pub const EIO: i32 = 5;
pub const EBADF: i32 = 9;
pub const EAGAIN: i32 = 11;
pub const ENOMEM: i32 = 12;
pub const EACCES: i32 = 13;
pub const EFAULT: i32 = 14;
pub const EBUSY: i32 = 16;
pub const EEXIST: i32 = 17;
pub const EINVAL: i32 = 22;
pub const ENFILE: i32 = 23;
pub const EMFILE: i32 = 24;
pub const ENOSPC: i32 = 28;
pub const ENOSYS: i32 = 38;
pub const ECHILD: i32 = 10;

// Exit codes
pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_FAILURE: i32 = 1;

// Signal numbers (POSIX)
pub const SIGHUP: u8 = 1;
pub const SIGINT: u8 = 2;
pub const SIGQUIT: u8 = 3;
pub const SIGKILL: u8 = 9;
pub const SIGSEGV: u8 = 11;
pub const SIGTERM: u8 = 15;
pub const SIGCHLD: u8 = 17;

// Wait flags
pub const WNOHANG: i32 = 1;
pub const WUNTRACED: i32 = 2;

// ============== Inline syscall helpers ==============

#[inline(always)]
pub unsafe fn syscall0(num: u64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "syscall",
        in("rax") num,
        lateout("rax") ret,
        options(nostack, preserves_flags)
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall1(num: u64, arg1: u64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "syscall",
        in("rax") num,
        in("rdi") arg1,
        lateout("rax") ret,
        options(nostack, preserves_flags)
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall2(num: u64, arg1: u64, arg2: u64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "syscall",
        in("rax") num,
        in("rdi") arg1,
        in("rsi") arg2,
        lateout("rax") ret,
        options(nostack, preserves_flags)
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall3(num: u64, arg1: u64, arg2: u64, arg3: u64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "syscall",
        in("rax") num,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        lateout("rax") ret,
        options(nostack, preserves_flags)
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall4(num: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "syscall",
        in("rax") num,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        in("r10") arg4,
        lateout("rax") ret,
        options(nostack, preserves_flags)
    );
    ret
}

// ============== POSIX syscall wrappers ==============

pub fn read(fd: i32, buf: *mut u8, count: usize) -> i64 {
    unsafe { syscall3(SYS_READ, fd as u64, buf as u64, count as u64) }
}

pub fn write(fd: i32, buf: *const u8, count: usize) -> i64 {
    unsafe { syscall3(SYS_WRITE, fd as u64, buf as u64, count as u64) }
}

pub fn open(pathname: *const c_char, flags: i32, mode: u32) -> i32 {
    unsafe { syscall3(SYS_OPEN, pathname as u64, flags as u64, mode as u64) as i32 }
}

pub fn close(fd: i32) -> i32 {
    unsafe { syscall1(SYS_CLOSE, fd as u64) as i32 }
}

pub fn fork() -> i32 {
    unsafe { syscall0(SYS_FORK) as i32 }
}

pub fn execve(filename: *const c_char, argv: *const *const c_char, envp: *const *const c_char) -> i32 {
    unsafe { syscall3(SYS_EXECVE, filename as u64, argv as u64, envp as u64) as i32 }
}

pub fn exit(status: i32) -> ! {
    unsafe {
        syscall1(SYS_EXIT, status as u64);
    }
    loop {}
}

pub fn waitpid(pid: i32, status: *mut i32, options: i32) -> i32 {
    unsafe { syscall4(SYS_WAIT4, pid as u64, status as u64, options as u64, 0) as i32 }
}

pub fn getpid() -> i32 {
    unsafe { syscall0(SYS_GETPID) as i32 }
}

pub fn getppid() -> i32 {
    unsafe { syscall0(SYS_GETPPID) as i32 }
}

pub fn getuid() -> u32 {
    unsafe { syscall0(SYS_GETUID) as u32 }
}

pub fn getgid() -> u32 {
    unsafe { syscall0(SYS_GETGID) as u32 }
}

pub fn getcwd(buf: *mut c_char, size: usize) -> *mut c_char {
    let result = unsafe { syscall2(SYS_GETCWD, buf as u64, size as u64) };
    if result < 0 {
        ptr::null_mut()
    } else {
        buf
    }
}

pub fn chdir(path: *const c_char) -> i32 {
    unsafe { syscall1(SYS_CHDIR, path as u64) as i32 }
}

pub fn mkdir(pathname: *const c_char, mode: u32) -> i32 {
    unsafe { syscall2(SYS_MKDIR, pathname as u64, mode as u64) as i32 }
}

pub fn unlink(pathname: *const c_char) -> i32 {
    unsafe { syscall1(SYS_UNLINK, pathname as u64) as i32 }
}

pub fn rmdir(pathname: *const c_char) -> i32 {
    unsafe { syscall1(SYS_RMDIR, pathname as u64) as i32 }
}

pub fn kill(pid: i32, sig: i32) -> i32 {
    unsafe { syscall2(SYS_KILL, pid as u64, sig as u64) as i32 }
}

pub fn chmod(path: *const c_char, mode: u32) -> i32 {
    unsafe { syscall2(SYS_CHMOD, path as u64, mode as u64) as i32 }
}

pub fn dup(oldfd: i32) -> i32 {
    unsafe { syscall1(SYS_DUP, oldfd as u64) as i32 }
}

pub fn dup2(oldfd: i32, newfd: i32) -> i32 {
    unsafe { syscall2(SYS_DUP2, oldfd as u64, newfd as u64) as i32 }
}

pub fn pipe(pipefd: *mut i32) -> i32 {
    unsafe { syscall1(SYS_PIPE, pipefd as u64) as i32 }
}

// ============== Standard string/memory functions ==============

pub fn strlen(s: *const c_char) -> usize {
    unsafe {
        let mut i = 0;
        while *s.add(i) != 0 {
            i += 1;
        }
        i
    }
}

pub fn strcmp(s1: *const c_char, s2: *const c_char) -> i32 {
    unsafe {
        let mut i = 0;
        loop {
            let c1 = *s1.add(i) as i32;
            let c2 = *s2.add(i) as i32;
            if c1 != c2 {
                return c1 - c2;
            }
            if c1 == 0 {
                return 0;
            }
            i += 1;
        }
    }
}

pub unsafe fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    for i in 0..n {
        *dest.add(i) = *src.add(i);
    }
    dest
}

pub unsafe fn memset(s: *mut u8, c: u8, n: usize) -> *mut u8 {
    for i in 0..n {
        *s.add(i) = c;
    }
    s
}

// ============== Printf-like output ==============

pub fn puts(s: *const c_char) -> i32 {
    let len = strlen(s);
    let result = write(STDOUT_FILENO, s as *const u8, len) as i32;
    write(STDOUT_FILENO, b"\n" as *const u8, 1);
    result
}

pub fn printf(format: *const c_char) -> i32 {
    // Simplified version: just write raw format string
    let len = strlen(format);
    write(STDOUT_FILENO, format as *const u8, len) as i32
}

