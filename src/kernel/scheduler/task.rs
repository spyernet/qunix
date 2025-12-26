use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use super::context::Context;

pub type Pid = u32;
pub type Tid = u32;

const KERNEL_STACK_SIZE: usize = 16 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Blocked,
    Sleeping,
    Zombie,
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskPriority {
    Idle = 0,
    Low = 1,
    Normal = 2,
    High = 3,
    RealTime = 4,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Normal
    }
}

#[derive(Debug, Clone)]
pub struct FileDescriptor {
    pub fd: i32,
    pub path: String,
    pub offset: u64,
    pub flags: u32, // O_CLOEXEC, etc.
}

/// POSIX-like Process Control Block
#[derive(Debug, Clone)]
pub struct Task {
    // Identity
    pub pid: Pid,
    pub ppid: Option<Pid>,          // Parent PID (POSIX)
    pub pgid: Pid,                  // Process group ID (for job control)
    pub sid: Pid,                   // Session ID
    
    // Process info
    pub name: String,
    pub state: TaskState,
    pub priority: TaskPriority,
    pub exit_code: Option<i32>,     // POSIX: set when exiting
    pub children: Vec<Pid>,         // POSIX: track child PIDs
    
    // Execution context
    pub context: Context,
    pub kernel_stack: usize,
    pub kernel_stack_size: usize,
    pub user_stack: usize,
    pub entry_point: usize,
    pub is_kernel_task: bool,
    
    // Credentials (POSIX)
    pub uid: u32,                   // Real UID
    pub gid: u32,                   // Real GID
    pub euid: u32,                  // Effective UID
    pub egid: u32,                  // Effective GID
    pub umask: u32,                 // File creation mask
    
    // File descriptor table
    pub cwd: String,                // Current working directory
    pub fds: BTreeMap<i32, FileDescriptor>,
    pub next_fd: i32,
    
    // Signals (POSIX)
    pub signal_mask: u64,           // Blocked signals
    pub pending_signals: u64,       // Signals to deliver
    pub signal_handlers: [u64; 64], // Signal handlers (future: function pointers)
    
    // Timing
    pub cpu_time: u64,              // CPU ticks consumed
    pub start_time: u64,            // Boot time when created
    pub last_schedule: u64,         // Last scheduled time
}

impl Task {
    /// Create a new task (POSIX-compatible PCB)
    pub fn new(pid: Pid, name: String, entry_point: usize, is_kernel: bool) -> Result<Self, &'static str> {
        // Allocate kernel stack for kernel tasks
        let (kernel_stack_ptr, kernel_stack_top) = if is_kernel {
            let boxed = Box::new([0u8; KERNEL_STACK_SIZE]);
            let ptr = Box::into_raw(boxed) as usize;
            let top = ptr + KERNEL_STACK_SIZE;
            (ptr, top)
        } else {
            (0, 0)
        };

        // Build execution context
        let context = if is_kernel {
            Context::new_kernel(entry_point, kernel_stack_top)
        } else {
            Context::new_user(entry_point, 0)
        };

        Ok(Task {
            // Identity (POSIX)
            pid,
            ppid: None,                 // Will be set on fork
            pgid: pid,                  // Process is own group initially
            sid: pid,                   // Process is own session initially
            
            // Process info
            name,
            state: TaskState::Ready,
            priority: TaskPriority::Normal,
            exit_code: None,            // Not exited yet
            children: Vec::new(),       // No children yet
            
            // Execution context
            context,
            kernel_stack: kernel_stack_ptr,
            kernel_stack_size: if is_kernel { KERNEL_STACK_SIZE } else { 0 },
            user_stack: 0,
            entry_point,
            is_kernel_task: is_kernel,
            
            // Credentials (POSIX)
            uid: if is_kernel { 0 } else { 1000 },
            gid: if is_kernel { 0 } else { 1000 },
            euid: if is_kernel { 0 } else { 1000 },
            egid: if is_kernel { 0 } else { 1000 },
            umask: 0o022,               // Standard umask
            
            // File descriptors
            cwd: String::from("/"),
            fds: BTreeMap::new(),
            next_fd: 3,                 // 0=stdin, 1=stdout, 2=stderr
            
            // Signals
            signal_mask: 0,
            pending_signals: 0,
            signal_handlers: [0; 64],
            
            // Timing
            cpu_time: 0,
            start_time: crate::hal::drivers::pit::get_ticks(),
            last_schedule: 0,
        })
    }

    /// Initialize standard file descriptors (stdin, stdout, stderr)
    pub fn init_fds(&mut self) {
        self.fds.insert(0, FileDescriptor {
            fd: 0,
            path: String::from("/dev/stdin"),
            offset: 0,
            flags: 0,
        });
        self.fds.insert(1, FileDescriptor {
            fd: 1,
            path: String::from("/dev/stdout"),
            offset: 0,
            flags: 1,
        });
        self.fds.insert(2, FileDescriptor {
            fd: 2,
            path: String::from("/dev/stderr"),
            offset: 0,
            flags: 1,
        });
    }

    /// POSIX fork: duplicate this task as a child
    pub fn fork(&self, child_pid: Pid) -> Result<Task, &'static str> {
        let mut child = self.clone();
        child.pid = child_pid;
        child.ppid = Some(self.pid);           // Set parent PID
        child.pgid = child_pid;                 // New process group
        child.children.clear();                 // Child has no children
        child.exit_code = None;                 // Not exited
        child.cpu_time = 0;
        child.start_time = crate::hal::drivers::pit::get_ticks();
        Ok(child)
    }

    /// POSIX exit: mark as zombie with exit code
    pub fn exit(&mut self, code: i32) {
        self.exit_code = Some(code);
        self.state = TaskState::Zombie;
    }

    /// Allocate a new file descriptor
    pub fn allocate_fd(&mut self) -> i32 {
        let fd = self.next_fd;
        self.next_fd += 1;
        fd
    }

    /// Close a file descriptor
    pub fn close_fd(&mut self, fd: i32) -> bool {
        self.fds.remove(&fd).is_some()
    }

    /// Get file descriptor (immutable)
    pub fn get_fd(&self, fd: i32) -> Option<&FileDescriptor> {
        self.fds.get(&fd)
    }

    /// Get file descriptor (mutable)
    pub fn get_fd_mut(&mut self, fd: i32) -> Option<&mut FileDescriptor> {
        self.fds.get_mut(&fd)
    }

    /// Check if process has root permissions
    pub fn is_root(&self) -> bool {
        self.euid == 0
    }

    /// Set process state
    pub fn set_state(&mut self, state: TaskState) {
        self.state = state;
    }

    /// Check if process is runnable
    pub fn is_runnable(&self) -> bool {
        matches!(self.state, TaskState::Ready | TaskState::Running)
    }

    // ========== Signal handling (POSIX) ==========

    /// Send a signal to this process
    pub fn send_signal(&mut self, signal: u8) {
        if signal < 64 {
            self.pending_signals |= 1 << signal;
        }
    }

    /// Check if signal is pending and not masked
    pub fn has_pending_signal(&self, signal: u8) -> bool {
        if signal < 64 {
            (self.pending_signals & (1 << signal)) != 0
                && (self.signal_mask & (1 << signal)) == 0
        } else {
            false
        }
    }

    /// Clear a pending signal
    pub fn clear_signal(&mut self, signal: u8) {
        if signal < 64 {
            self.pending_signals &= !(1 << signal);
        }
    }

    /// Block (mask) a signal
    pub fn block_signal(&mut self, signal: u8) {
        if signal < 64 {
            self.signal_mask |= 1 << signal;
        }
    }

    /// Unblock (unmask) a signal
    pub fn unblock_signal(&mut self, signal: u8) {
        if signal < 64 {
            self.signal_mask &= !(1 << signal);
        }
    }

    // ========== Process hierarchy (POSIX) ==========

    /// Add a child PID to this process
    pub fn add_child(&mut self, child_pid: Pid) {
        if !self.children.contains(&child_pid) {
            self.children.push(child_pid);
        }
    }

    /// Remove a child PID
    pub fn remove_child(&mut self, child_pid: Pid) {
        self.children.retain(|&pid| pid != child_pid);
    }

    /// Check if this process has any children
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}

/// POSIX signal definitions (standard)
pub const SIGHUP: u8 = 1;
pub const SIGINT: u8 = 2;
pub const SIGQUIT: u8 = 3;
pub const SIGABRT: u8 = 6;
pub const SIGKILL: u8 = 9;      // Cannot be caught/blocked
pub const SIGTERM: u8 = 15;
pub const SIGCHLD: u8 = 17;     // Child process exited
pub const SIGSTOP: u8 = 19;     // Cannot be caught/blocked
pub const SIGTSTP: u8 = 20;     // Terminal stop signal

impl Drop for Task {
    fn drop(&mut self) {
        // Deallocate kernel stack if it was allocated
        if self.is_kernel_task && self.kernel_stack != 0 {
            unsafe {
                let boxed = Box::from_raw(self.kernel_stack as *mut [u8; KERNEL_STACK_SIZE]);
                drop(boxed);
            }
        }
    }
}
