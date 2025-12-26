
// Minimal qutils: echo, cat, ls

extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
use core::ffi::c_char;
use crate::userland::libc;

pub fn echo(args: &[String]) -> i32 {
    for (i, a) in args.iter().enumerate() {
        if i > 0 { libc::write(libc::STDOUT_FILENO, b" ".as_ptr(), 1); }
        libc::write(libc::STDOUT_FILENO, a.as_ptr() as *const u8, a.len());
    }
    libc::write(libc::STDOUT_FILENO, b"\n".as_ptr(), 1);
    0
}

pub fn cat(args: &[String]) -> i32 {
    if args.is_empty() {
        // read stdin to stdout
        let mut buf = [0u8; 512];
        loop {
            let n = libc::read(libc::STDIN_FILENO, buf.as_mut_ptr(), 512);
            if n <= 0 { break; }
            libc::write(libc::STDOUT_FILENO, buf.as_ptr(), n as usize);
        }
        return 0;
    }

    for path in args {
        // open file via libc open
        let cpath = path.as_ptr() as *const c_char;
        let fd = libc::open(cpath, libc::O_RDONLY, 0);
        if fd < 0 {
            let msg = "cat: cannot open file\n";
            libc::write(libc::STDERR_FILENO, msg.as_ptr() as *const u8, msg.len());
            continue;
        }
        let mut buf = [0u8; 512];
        loop {
            let n = libc::read(fd, buf.as_mut_ptr(), 512);
            if n <= 0 { break; }
            libc::write(libc::STDOUT_FILENO, buf.as_ptr(), n as usize);
        }
        libc::close(fd);
    }
    0
}

pub fn ls(args: &[String]) -> i32 {
    let path = if args.is_empty() { 
        "/".to_string()
    } else {
        args[0].clone()
    };

    match crate::fs::vfs::api::readdir(&path) {
        Ok(entries) => {
            for e in entries {
                let s = alloc::format!("{}\n", e.name);
                libc::write(libc::STDOUT_FILENO, s.as_ptr() as *const u8, s.len());
            }
            0
        }
        Err(_) => {
            let msg = "ls: cannot access directory\n";
            libc::write(libc::STDERR_FILENO, msg.as_ptr() as *const u8, msg.len());
            -1
        }
    }
}
