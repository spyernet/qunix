// mkdir - Create directory

pub fn run(args: &[&str]) {
    if args.is_empty() {
        crate::serial_println!("Usage: mkdir <directory>");
        return;
    }
    
    let mut vfs = crate::fs::vfs::VFS.lock();
    
    for dirname in args {
        match vfs.create_directory(dirname, crate::fs::FileMode::new(0o755)) {
            Ok(_) => {
                crate::serial_println!("Created directory: {}", dirname);
            }
            Err(e) => {
                crate::serial_println!("mkdir: error creating '{}': {:?}", dirname, e);
            }
        }
    }
}
