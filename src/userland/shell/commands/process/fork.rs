// fork - Test fork syscall

pub fn run() {
    // Use inline assembly to call fork syscall
    let pid: i32 = unsafe {
        let result: i64;
        core::arch::asm!(
            "mov rax, 57; syscall",
            out("rax") result,
            options(nostack, preserves_flags)
        );
        result as i32
    };
    
    if pid == 0 {
        crate::serial_println!("[CHILD] This is the child process");
    } else if pid > 0 {
        crate::serial_println!("[PARENT] Forked child process: {}", pid);
    } else {
        crate::serial_println!("fork() failed with error code: {}", pid);
    }
}
