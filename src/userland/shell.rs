// Minimal shell implementation for Qunix
//
// Provides basic command parsing, execution, and shell built-ins

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use alloc::string::ToString;
use core::ffi::c_char;
use crate::userland::libc;

#[derive(Debug, Clone)]
pub struct Command {
    pub prog: String,
    pub args: Vec<String>,
    pub input_fd: i32,
    pub output_fd: i32,
    pub background: bool,
}

pub struct Shell {
    pub cwd: String,
    pub running: bool,
    pub last_exit_code: i32,
}

impl Shell {
    pub fn new() -> Self {
        Shell {
            cwd: "/".to_string(),
            running: true,
            last_exit_code: 0,
        }
    }

    /// Parse a command line into tokens
    pub fn parse_command(&self, line: &str) -> Option<Command> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }

        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        if tokens.is_empty() {
            return None;
        }

        let prog = tokens[0].to_string();
        let args = tokens[1..].iter().map(|s| s.to_string()).collect();
        
        let background = trimmed.ends_with('&');

        Some(Command {
            prog,
            args,
            input_fd: libc::STDIN_FILENO,
            output_fd: libc::STDOUT_FILENO,
            background,
        })
    }

    /// Execute a single command (builtin or fork+exec)
    pub fn execute_command(&mut self, cmd: &Command) -> i32 {
        // Check for builtins
        match cmd.prog.as_str() {
            "cd" => self.builtin_cd(&cmd.args),
            "pwd" => self.builtin_pwd(),
            "exit" => self.builtin_exit(&cmd.args),
            "echo" => self.builtin_echo(&cmd.args),
            "ls" => self.builtin_ls(&cmd.args),
            "help" => self.builtin_help(),
            "clear" => self.builtin_clear(),
            _ => self.execute_external(&cmd),
        }
    }

    // =========== Built-in Commands ===========

    fn builtin_cd(&mut self, args: &[String]) -> i32 {
        let path = if args.is_empty() {
            "/".to_string()
        } else {
            args[0].clone()
        };

        // For now, just update internal state
        // In a real implementation, would call chdir syscall
        self.cwd = path;
        0
    }

    fn builtin_pwd(&self) -> i32 {
        let msg = format!("{}\n", self.cwd);
        libc::write(libc::STDOUT_FILENO, msg.as_ptr() as *const u8, msg.len());
        0
    }

    fn builtin_exit(&mut self, args: &[String]) -> i32 {
        let code = if args.is_empty() {
            0
        } else {
            args[0].parse::<i32>().unwrap_or(0)
        };
        self.running = false;
        code
    }

    fn builtin_echo(&self, args: &[String]) -> i32 {
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                libc::write(libc::STDOUT_FILENO, b" ".as_ptr() as *const u8, 1);
            }
            libc::write(libc::STDOUT_FILENO, arg.as_ptr() as *const u8, arg.len());
        }
        libc::write(libc::STDOUT_FILENO, b"\n".as_ptr() as *const u8, 1);
        0
    }

    fn builtin_ls(&self, _args: &[String]) -> i32 {
        let msg = "Filesystem entries (stub)\n";
        libc::write(libc::STDOUT_FILENO, msg.as_ptr() as *const u8, msg.len());
        0
    }

    fn builtin_help(&self) -> i32 {
        let help_text = r#"Qunix Shell - Built-in Commands:
  cd [DIR]       - Change directory
  pwd            - Print working directory
  echo [ARGS]    - Echo arguments
  ls [DIR]       - List directory contents
  exit [CODE]    - Exit shell
  help           - Show this help
  clear          - Clear screen
"#;
        libc::write(libc::STDOUT_FILENO, help_text.as_ptr() as *const u8, help_text.len());
        0
    }

    fn builtin_clear(&self) -> i32 {
        // ANSI clear screen escape code
        let clear = "\x1b[2J\x1b[H";
        libc::write(libc::STDOUT_FILENO, clear.as_ptr() as *const u8, clear.len());
        0
    }

    /// Execute external command via fork+exec
    fn execute_external(&mut self, cmd: &Command) -> i32 {
        let pid = libc::fork();
        
        match pid {
            0 => {
                // Child process: execute the program
                // In a real implementation, would call execve
                0
            }
            n if n > 0 => {
                // Parent process: wait for child if not background
                if !cmd.background {
                    let mut status: i32 = 0;
                    libc::waitpid(pid, &mut status as *mut i32, 0);
                    self.last_exit_code = status;
                    status
                } else {
                    // Background job
                    self.last_exit_code = pid;
                    pid
                }
            }
            _ => {
                // Fork failed
                -1
            }
        }
    }
}

/// Main shell REPL loop
pub fn shell_main() {
    let mut shell = Shell::new();
    let prompt = "qunix# ";
    let mut line = String::new();

    while shell.running {
        // Print prompt
        libc::write(libc::STDOUT_FILENO, prompt.as_ptr() as *const u8, prompt.len());

        // Read a line (simplified: just read until newline)
        line.clear();
        let mut buf = [0u8; 256];
        let n = libc::read(libc::STDIN_FILENO, buf.as_mut_ptr(), 256);
        
        if n <= 0 {
            break;
        }

        // Parse and execute
        let input = core::str::from_utf8(&buf[..n as usize]).unwrap_or("");
        if let Some(cmd) = shell.parse_command(input) {
            let _exit_code = shell.execute_command(&cmd);
        }
    }
}

