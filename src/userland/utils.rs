// Core userland utilities for Qunix
//
// Basic POSIX command-line tools

extern crate alloc;

use core::ffi::c_char;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use alloc::string::ToString;
use crate::userland::libc;

// ============== Cat - concatenate and print files ==============

pub fn cat_main(argc: i32, argv: *const *const c_char) -> i32 {
    if argc < 2 {
        let msg = "Usage: cat [FILE...]\n";
        libc::write(libc::STDOUT_FILENO, msg.as_ptr() as *const u8, msg.len());
        return 1;
    }

    let mut buf = [0u8; 4096];
    let mut total_ret = 0;

    for i in 1..argc {
        let cstr = unsafe { *argv.add(i as usize) };
        
        let path = unsafe {
            let mut path_bytes = Vec::new();
            let mut ptr = cstr as *const u8;
            while *ptr != 0 {
                path_bytes.push(*ptr);
                ptr = ptr.add(1);
            }
            String::from_utf8_lossy(&path_bytes).to_string()
        };

        let fd = libc::open(cstr, libc::O_RDONLY, 0);
        if fd < 0 {
            let msg = format!("cat: cannot open '{}'\n", path);
            libc::write(libc::STDERR_FILENO, msg.as_ptr() as *const u8, msg.len());
            total_ret = 1;
            continue;
        }

        loop {
            let n = libc::read(fd, buf.as_mut_ptr(), buf.len());
            if n <= 0 {
                break;
            }
            libc::write(libc::STDOUT_FILENO, buf.as_ptr(), n as usize);
        }

        libc::close(fd);
    }

    total_ret
}

// ============== Echo - output text ==============

pub fn echo_main(argc: i32, argv: *const *const c_char) -> i32 {
    for i in 1..argc {
        if i > 1 {
            libc::write(libc::STDOUT_FILENO, b" ".as_ptr() as *const u8, 1);
        }

        let cstr = unsafe { *argv.add(i as usize) };
        let len = libc::strlen(cstr);
        libc::write(libc::STDOUT_FILENO, cstr as *const u8, len);
    }

    libc::write(libc::STDOUT_FILENO, b"\n".as_ptr() as *const u8, 1);
    0
}

// ============== Pwd - print working directory ==============

pub fn pwd_main() -> i32 {
    let mut buf = [0u8; 256];
    let result = libc::getcwd(buf.as_mut_ptr() as *mut c_char, buf.len());
    
    if result.is_null() {
        let msg = "pwd: error\n";
        libc::write(libc::STDERR_FILENO, msg.as_ptr() as *const u8, msg.len());
        return 1;
    }

    let len = libc::strlen(buf.as_ptr() as *const c_char);
    libc::write(libc::STDOUT_FILENO, buf.as_ptr(), len);
    libc::write(libc::STDOUT_FILENO, b"\n".as_ptr() as *const u8, 1);
    0
}

// ============== Ls - list directory contents ==============

pub fn ls_main(_argc: i32, _argv: *const *const c_char) -> i32 {
    let msg = "Listing directory (filesystem read not fully implemented)\n";
    libc::write(libc::STDOUT_FILENO, msg.as_ptr() as *const u8, msg.len());
    0
}

// ============== Mkdir - create directory ==============

pub fn mkdir_main(argc: i32, argv: *const *const c_char) -> i32 {
    if argc < 2 {
        let msg = "Usage: mkdir [DIR...]\n";
        libc::write(libc::STDERR_FILENO, msg.as_ptr() as *const u8, msg.len());
        return 1;
    }

    let mut ret = 0;
    for i in 1..argc {
        let cstr = unsafe { *argv.add(i as usize) };
        let result = libc::mkdir(cstr, 0o755);
        if result != 0 {
            let msg = "mkdir: cannot create directory\n";
            libc::write(libc::STDERR_FILENO, msg.as_ptr() as *const u8, msg.len());
            ret = 1;
        }
    }

    ret
}

// ============== Rm - remove files ==============

pub fn rm_main(argc: i32, argv: *const *const c_char) -> i32 {
    if argc < 2 {
        let msg = "Usage: rm [FILE...]\n";
        libc::write(libc::STDERR_FILENO, msg.as_ptr() as *const u8, msg.len());
        return 1;
    }

    let mut ret = 0;
    for i in 1..argc {
        let cstr = unsafe { *argv.add(i as usize) };
        let result = libc::unlink(cstr);
        if result != 0 {
            let msg = "rm: cannot remove file\n";
            libc::write(libc::STDERR_FILENO, msg.as_ptr() as *const u8, msg.len());
            ret = 1;
        }
    }

    ret
}

// ============== Touch - create empty file ==============

pub fn touch_main(argc: i32, argv: *const *const c_char) -> i32 {
    if argc < 2 {
        let msg = "Usage: touch [FILE...]\n";
        libc::write(libc::STDERR_FILENO, msg.as_ptr() as *const u8, msg.len());
        return 1;
    }

    let mut ret = 0;
    for i in 1..argc {
        let cstr = unsafe { *argv.add(i as usize) };

        let fd = libc::open(
            cstr,
            libc::O_CREAT | libc::O_WRONLY,
            0o644,
        );
        
        if fd < 0 {
            let msg = "touch: cannot create file\n";
            libc::write(libc::STDERR_FILENO, msg.as_ptr() as *const u8, msg.len());
            ret = 1;
        } else {
            libc::close(fd);
        }
    }

    ret
}

// ============== Uname - system information ==============

pub fn uname_main() -> i32 {
    let msg = "Qunix 0.1 (x86_64)\n";
    libc::write(libc::STDOUT_FILENO, msg.as_ptr() as *const u8, msg.len());
    0
}

// ============== Id - print user/group IDs ==============

pub fn id_main() -> i32 {
    let uid = libc::getuid();
    let gid = libc::getgid();
    
    let msg = format!("uid={} gid={}\n", uid, gid);
    libc::write(libc::STDOUT_FILENO, msg.as_ptr() as *const u8, msg.len());
    0
}

