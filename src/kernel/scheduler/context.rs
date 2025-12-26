

#[derive(Debug, Clone, Copy)]
pub struct Context {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
    pub rflags: u64,
    pub cs: u64,
    pub ss: u64,
    pub ds: u64,
    pub es: u64,
    pub fs: u64,
    pub gs: u64,
    pub cr3: u64,
}

impl Context {
    pub fn new() -> Self {
        Context {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rdi: 0,
            rbp: 0,
            rsp: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip: 0,
            rflags: 0x202,
            cs: 0,
            ss: 0,
            ds: 0,
            es: 0,
            fs: 0,
            gs: 0,
            cr3: 0,
        }
    }

    /// Create a kernel-mode context (ring 0)
    pub fn new_kernel(entry_point: usize, stack_top: usize) -> Self {
        let mut ctx = Self::new();
        ctx.rip = entry_point as u64;
        ctx.rsp = stack_top as u64;
        ctx.rbp = stack_top as u64;
        ctx.cs = 0x08;
        ctx.ss = 0x10;
        ctx.ds = 0x10;
        ctx.es = 0x10;
        ctx.rflags = 0x202;
        ctx
    }

    /// Create a user-mode context (ring 3) - for future use
    pub fn new_user(entry_point: usize, stack_top: usize) -> Self {
        let mut ctx = Self::new();
        ctx.rip = entry_point as u64;
        ctx.rsp = stack_top as u64;
        ctx.rbp = stack_top as u64;
        ctx.cs = 0x23;
        ctx.ss = 0x1b;
        ctx.ds = 0x1b;
        ctx.es = 0x1b;
        ctx.rflags = 0x202;
        ctx
    }

    /// Set up registers for syscall return (rax = return value)
    pub fn set_return(&mut self, value: i64) {
        self.rax = value as u64;
    }

    /// Get first 6 syscall arguments (Linux x86_64 ABI)
    pub fn syscall_args(&self) -> (u64, u64, u64, u64, u64, u64) {
        (self.rdi, self.rsi, self.rdx, self.r10, self.r8, self.r9)
    }

    /// Set syscall argument for retval (rax)
    pub fn syscall_retval(&mut self, val: i64) {
        self.rax = val as u64;
    }
}