// pwd - Print working directory

pub fn run() {
    let vfs = crate::fs::vfs::VFS.lock();
    let cwd = vfs.get_cwd();
    crate::serial_println!("{}", cwd);
}
