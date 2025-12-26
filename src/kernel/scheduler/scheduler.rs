extern crate alloc;

use alloc::vec::Vec;
use alloc::collections::VecDeque;
use spin::Mutex;
use lazy_static::lazy_static;

use super::task::{Task, TaskState, TaskPriority, Pid};

lazy_static! {
    pub static ref SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
}

pub struct Scheduler {
    pub tasks: Vec<Task>,
    pub ready_queue: [VecDeque<Pid>; 5],
    pub current_pid: Option<Pid>,
    pub next_pid: Pid,
    pub idle_pid: Option<Pid>,
    pub ticks: u64,
    pub time_slice: u64,
    pub preemption_enabled: bool,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            tasks: Vec::new(),
            ready_queue: [
                VecDeque::new(),
                VecDeque::new(),
                VecDeque::new(),
                VecDeque::new(),
                VecDeque::new(),
            ],
            current_pid: None,
            next_pid: 1,
            idle_pid: None,
            ticks: 0,
            time_slice: 10,
            preemption_enabled: true,
        }
    }

    pub fn add_task(&mut self, mut task: Task) {
        let pid = task.pid;
        let priority = task.priority as usize;
        task.init_fds();
        self.tasks.push(task);
        self.ready_queue[priority].push_back(pid);
        if self.next_pid <= pid {
            self.next_pid = pid + 1;
        }
    }

    pub fn allocate_pid(&mut self) -> Pid {
        let pid = self.next_pid;
        self.next_pid += 1;
        pid
    }

    pub fn get_task(&self, pid: Pid) -> Option<&Task> {
        self.tasks.iter().find(|t| t.pid == pid)
    }

    pub fn get_task_mut(&mut self, pid: Pid) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.pid == pid)
    }

    pub fn current(&self) -> Option<&Task> {
        self.current_pid.and_then(|pid| self.get_task(pid))
    }

    pub fn current_mut(&mut self) -> Option<&mut Task> {
        if let Some(pid) = self.current_pid {
            self.tasks.iter_mut().find(|t| t.pid == pid)
        } else {
            None
        }
    }

    pub fn current_pid(&self) -> Option<Pid> {
        self.current_pid
    }

    pub fn get_tasks(&self) -> &[Task] {
        &self.tasks
    }

    fn select_next(&mut self) -> Option<Pid> {
        for priority in (0..5).rev() {
            if let Some(pid) = self.ready_queue[priority].pop_front() {
                return Some(pid);
            }
        }
        self.idle_pid
    }

    pub fn schedule(&mut self) {
        if !self.preemption_enabled {
            return;
        }

        self.ticks += 1;

        if self.ticks % self.time_slice != 0 {
            return;
        }

        if let Some(current_pid) = self.current_pid {
            if let Some(task) = self.get_task_mut(current_pid) {
                if task.state == TaskState::Running {
                    task.state = TaskState::Ready;
                    let priority = task.priority as usize;
                    self.ready_queue[priority].push_back(current_pid);
                }
            }
        }

        if let Some(next_pid) = self.select_next() {
            if Some(next_pid) != self.current_pid {
                self.switch_to(next_pid);
            }
        }
    }

    fn switch_to(&mut self, next_pid: Pid) {
        let _old_pid = self.current_pid;
        self.current_pid = Some(next_pid);
        if let Some(task) = self.get_task_mut(next_pid) {
            task.state = TaskState::Running;
            task.cpu_time += 1;
        }
    }

    pub fn block_current(&mut self) {
        if let Some(pid) = self.current_pid {
            if let Some(task) = self.get_task_mut(pid) {
                task.state = TaskState::Blocked;
            }
        }

        self.schedule();
    }

    pub fn unblock(&mut self, pid: Pid) {
        if let Some(task) = self.get_task_mut(pid) {
            if task.state == TaskState::Blocked {
                task.state = TaskState::Ready;
                let priority = task.priority as usize;
                self.ready_queue[priority].push_back(pid);
            }
        }
    }

    pub fn sleep(&mut self, _ticks: u64) {
        if let Some(task) = self.current_mut() {
            task.state = TaskState::Sleeping;
        }

        self.schedule();
    }

    pub fn exit(&mut self, code: i32) {
        if let Some(task) = self.current_mut() {
            task.exit(code);
        }

        self.current_pid = None;
        self.schedule();
    }

    pub fn kill(&mut self, pid: Pid, signal: u8) -> bool {
        if let Some(task) = self.get_task_mut(pid) {
            task.send_signal(signal);
            true
        } else {
            false
        }
    }

    pub fn remove_zombie(&mut self, pid: Pid) -> Option<i32> {
        if let Some(pos) = self.tasks.iter().position(|t| t.pid == pid && t.state == TaskState::Zombie) {
            let task = self.tasks.remove(pos);
            task.exit_code
        } else {
            None
        }
    }

    pub fn set_priority(&mut self, pid: Pid, priority: TaskPriority) {
        if let Some(task) = self.get_task_mut(pid) {
            task.priority = priority;
        }
    }

    pub fn disable_preemption(&mut self) {
        self.preemption_enabled = false;
    }

    pub fn enable_preemption(&mut self) {
        self.preemption_enabled = true;
    }

    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    pub fn ready_count(&self) -> usize {
        self.ready_queue.iter().map(|q| q.len()).sum()
    }
}

pub fn init() {
    crate::println!("[SCHED] Scheduler initialized");
}

pub fn schedule() {
    let mut scheduler = SCHEDULER.lock();
    scheduler.schedule();
}

pub fn yield_now() {
    let mut scheduler = SCHEDULER.lock();
    scheduler.schedule();
}

pub fn current_pid() -> Option<Pid> {
    SCHEDULER.lock().current_pid()
}

pub fn spawn(name: alloc::string::String, entry: usize) -> Pid {
    let mut scheduler = SCHEDULER.lock();
    let pid = scheduler.allocate_pid();
    match Task::new(pid, name, entry, false) {
        Ok(task) => {
            scheduler.add_task(task);
            pid
        }
        Err(e) => {
            crate::println!("[SCHED] ERROR: spawn failed: {}", e);
            0
        }
    }
}

pub fn run_first_task(pid: Pid) {
    crate::println!("[SCHED] run_first_task({}) called", pid);
    
    let mut sched = SCHEDULER.lock();
    if let Some(task) = sched.get_task_mut(pid) {
        crate::println!("[SCHED] Task {} found: {}", pid, task.name);
        task.state = TaskState::Running;
        sched.current_pid = Some(pid);
    } else {
        panic!("[SCHED] PANIC: run_first_task: PID {} not found", pid);
    }
    
    crate::println!("[SCHED] run_first_task: Task marked running, continuing...");
}

pub fn exit(code: i32) {
    SCHEDULER.lock().exit(code);
}

pub fn kill(pid: Pid, signal: u8) -> bool {
    SCHEDULER.lock().kill(pid, signal)
}

pub fn get_task(pid: Pid) -> Option<Task> {
    SCHEDULER.lock().get_task(pid).cloned()
}