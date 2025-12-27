// cd - Change directory

pub fn run(args: &[&str]) {
    let target = if args.is_empty() { "/root" } else { args[0] };
    
    let mut vfs = crate::fs::vfs::VFS.lock();
    match vfs.set_cwd(target) {
        Ok(_) => {
            // Silent on success like real cd
        }
        Err(e) => {
            crate::serial_println!("cd: error changing to '{}': {:?}", target, e);
        }
    }
}
